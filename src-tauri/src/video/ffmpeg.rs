//! Locating and running the FFmpeg sidecar.

use std::collections::VecDeque;
use std::ffi::OsString;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output, Stdio};
use std::sync::atomic::Ordering;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use crate::error::{AppError, AppResult};
use crate::state::{lock, VideoState};

/// `bundle.externalBin` name; deliberately not `ffmpeg` so the .deb does not
/// install a second /usr/bin/ffmpeg.
pub const SIDECAR: &str = "sharpr-ffmpeg";
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const STDERR_TAIL: usize = 40;
const MESSAGE_MAX: usize = 300;
const PROGRESS_INTERVAL: Duration = Duration::from_millis(250);
const PROGRESS_STEP: f64 = 0.005;

fn binary_name() -> String {
    format!("{SIDECAR}{}", std::env::consts::EXE_SUFFIX)
}

/// The bundled binary next to the app executable, where tauri-build (dev)
/// and the bundlers (release) put `externalBin`.
pub fn resolve() -> AppResult<PathBuf> {
    static RESOLVED: OnceLock<Option<PathBuf>> = OnceLock::new();
    RESOLVED
        .get_or_init(find)
        .clone()
        .ok_or_else(|| AppError::FfmpegMissing(binary_name()))
}

fn find() -> Option<PathBuf> {
    #[cfg(any(test, debug_assertions))]
    if let Some(path) = std::env::var_os("SHARPR_FFMPEG").map(PathBuf::from) {
        if path.is_file() {
            return Some(path);
        }
    }
    let exe = tauri::utils::platform::current_exe().ok()?;
    let dir = exe.parent()?;
    let mut candidates = vec![dir.join(binary_name())];
    // Test binaries live in target/<profile>/deps; the sidecar is one level up.
    if cfg!(test) {
        candidates.extend(dir.parent().map(|parent| parent.join(binary_name())));
    }
    candidates.into_iter().find(|path| path.is_file())
}

/// A command with stdin closed and, on Windows, no console window.
pub fn command(ffmpeg: &Path, args: &[OsString]) -> Command {
    let mut command = Command::new(ffmpeg);
    command.args(args).stdin(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    command
}

/// Run to completion and capture stdout/stderr (probe, thumbnails). The exit
/// status is left to the caller: `ffmpeg -i` alone always "fails".
pub fn run_capture(ffmpeg: &Path, args: &[OsString]) -> AppResult<Output> {
    Ok(command(ffmpeg, args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?)
}

/// Run a long job that can be cancelled. The child is parked in
/// `state.child` so `cancel_compress` can kill it; `-progress pipe:1` output
/// is turned into completion fractions (`None` when the duration is unknown).
pub fn run_tracked(
    state: &VideoState,
    ffmpeg: &Path,
    args: &[OsString],
    duration: Option<f64>,
    mut on_fraction: impl FnMut(Option<f64>),
) -> AppResult<()> {
    let (stdout, stderr) = {
        let mut slot = lock(&state.child);
        // Checked under the lock so a cancel can't slip in before the spawn.
        if state.cancel.load(Ordering::SeqCst) {
            return Err(AppError::Cancelled);
        }
        let mut child = command(ffmpeg, args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let pipes = (child.stdout.take(), child.stderr.take());
        *slot = Some(child);
        pipes
    };
    let tail = std::thread::scope(|scope| {
        let reader = scope.spawn(move || stderr.map(stderr_tail).unwrap_or_default());
        if let Some(stdout) = stdout {
            read_progress(stdout, duration, &mut on_fraction);
        }
        reader.join().unwrap_or_default()
    });
    let Some(mut child) = lock(&state.child).take() else {
        return Err(AppError::Cancelled); // shutdown() already reaped it
    };
    let status = child.wait()?;
    if state.cancel.load(Ordering::SeqCst) {
        return Err(AppError::Cancelled);
    }
    if status.success() {
        Ok(())
    } else {
        Err(AppError::Ffmpeg(failure_message(&tail, status)))
    }
}

fn read_progress(
    stdout: impl Read,
    duration: Option<f64>,
    on_fraction: &mut impl FnMut(Option<f64>),
) {
    let duration = duration.filter(|d| *d > 0.0);
    if duration.is_none() {
        on_fraction(None);
    }
    let mut last: Option<(Instant, f64)> = None;
    // Keep draining even without a duration so FFmpeg never blocks on a full pipe.
    for line in BufReader::new(stdout).lines().map_while(Result::ok) {
        let (Some(duration), Some(value)) = (duration, line.strip_prefix("out_time_us=")) else {
            continue;
        };
        let Ok(micros) = value.trim().parse::<f64>() else {
            continue; // "N/A" before the first frame
        };
        let fraction = (micros / 1_000_000.0 / duration).clamp(0.0, 1.0);
        let due = last.is_none_or(|(at, previous)| {
            at.elapsed() >= PROGRESS_INTERVAL && fraction - previous >= PROGRESS_STEP
        });
        if due {
            on_fraction(Some(fraction));
            last = Some((Instant::now(), fraction));
        }
    }
}

fn stderr_tail(stderr: impl Read) -> VecDeque<String> {
    let mut tail = VecDeque::with_capacity(STDERR_TAIL);
    for line in BufReader::new(stderr).lines().map_while(Result::ok) {
        if tail.len() == STDERR_TAIL {
            tail.pop_front();
        }
        tail.push_back(line);
    }
    tail
}

fn failure_message(tail: &VecDeque<String>, status: ExitStatus) -> String {
    let line = tail
        .iter()
        .rev()
        .map(|line| line.trim())
        .find(|line| !line.is_empty());
    match line {
        Some(line) => line.chars().take(MESSAGE_MAX).collect(),
        None => format!("exited with {status}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::video::testing;

    #[test]
    fn progress_is_reported_as_fractions() {
        let input = "frame=1\nout_time_us=N/A\nout_time_us=1500000\nprogress=continue\nout_time_us=3000000\nprogress=end\n";
        let mut seen = Vec::new();
        read_progress(input.as_bytes(), Some(3.0), &mut |f| seen.push(f));
        // The second update is throttled (it arrives within 250 ms).
        assert_eq!(seen, vec![Some(0.5)]);
    }

    #[test]
    fn unknown_duration_reports_indeterminate_once() {
        let input = "out_time_us=1000000\nout_time_us=2000000\n";
        let mut seen = Vec::new();
        read_progress(input.as_bytes(), None, &mut |f| seen.push(f));
        assert_eq!(seen, vec![None]);
    }

    #[test]
    fn failure_message_uses_the_last_line() {
        let tail: VecDeque<String> = ["first", "Conversion failed!", "  "]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let status = Command::new(if cfg!(windows) { "cmd" } else { "false" })
            .args(if cfg!(windows) { &["/C", "exit 1"][..] } else { &[][..] })
            .status()
            .unwrap();
        assert_eq!(failure_message(&tail, status), "Conversion failed!");
        assert!(failure_message(&VecDeque::new(), status).starts_with("exited with"));
    }

    #[test]
    fn sidecar_runs() {
        let Some(ffmpeg) = testing::ffmpeg() else {
            return;
        };
        let output = run_capture(&ffmpeg, &["-hide_banner".into(), "-version".into()]).unwrap();
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).starts_with("ffmpeg version"));
    }
}
