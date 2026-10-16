//! Test helpers: find the FFmpeg sidecar and synthesize clips with lavfi so
//! no binary fixtures live in the repository.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The sidecar (or `SHARPR_FFMPEG`); None makes FFmpeg-backed tests skip
/// locally. CI always fetches it before running tests.
pub fn ffmpeg() -> Option<PathBuf> {
    match super::ffmpeg::resolve() {
        Ok(path) => Some(path),
        Err(_) => {
            eprintln!("skipping: FFmpeg sidecar not found (run `pnpm ffmpeg:fetch`)");
            None
        }
    }
}

/// A synthetic test clip.
pub struct Clip {
    pub size: (u32, u32),
    pub seconds: f64,
    pub audio: bool,
    /// Extra output arguments (codec, quality, color tags).
    pub video_args: &'static [&'static str],
}

impl Default for Clip {
    fn default() -> Self {
        Clip {
            size: (640, 360),
            seconds: 2.0,
            audio: true,
            // Nearly lossless, so a re-encode is always much smaller.
            video_args: &["-c:v", "libx264", "-crf", "8", "-pix_fmt", "yuv420p"],
        }
    }
}

pub fn make_clip(ffmpeg: &Path, path: &Path, clip: &Clip) {
    let source = format!("testsrc2=size={}x{}:rate=30", clip.size.0, clip.size.1);
    let mut command = Command::new(ffmpeg);
    command.args(["-hide_banner", "-v", "error", "-y", "-f", "lavfi", "-i", &source]);
    if clip.audio {
        command.args(["-f", "lavfi", "-i", "sine=frequency=440:sample_rate=48000"]);
    }
    command.args(["-t", &clip.seconds.to_string()]).args(clip.video_args);
    if clip.audio {
        command.args(["-c:a", "aac", "-b:a", "96k"]);
    }
    let status = command.arg(path).status().expect("spawn ffmpeg");
    assert!(status.success(), "could not create {}", path.display());
}

/// Copy `input` to `output` with a display-rotation tag (like phone videos).
pub fn rotate_tag(ffmpeg: &Path, input: &Path, output: &Path, degrees: i32) {
    let status = Command::new(ffmpeg)
        .args(["-hide_banner", "-v", "error", "-y", "-display_rotation"])
        .arg(degrees.to_string())
        .arg("-i")
        .arg(input)
        .args(["-c", "copy"])
        .arg(output)
        .status()
        .expect("spawn ffmpeg");
    assert!(status.success());
}

/// A fresh scratch folder under the system temp dir.
pub fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sharpr-video-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
