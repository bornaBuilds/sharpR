//! The compression queue: one video at a time (x264 and VP9 already use every
//! core), with progress, cancellation, the keep-original fallback and poster
//! extraction.

use std::ops::RangeInclusive;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;

use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, State};

use crate::error::{AppError, AppResult};
use crate::state::{lock, PosterFrame, VideoEntry, VideoResult, VideoState};
use crate::video::args::{self, EncodePlan};
use crate::video::formats::{self, VideoFormat, VideoQuality};
use crate::video::{ffmpeg, posters};

/// Keep the source when the re-encode isn't at least this much smaller.
const MIN_SAVING: f64 = 0.05;
/// Accepted short-side caps (144p .. 8K).
const RESOLUTION_RANGE: RangeInclusive<u32> = 144..=4320;

#[derive(Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VideoOptions {
    pub format: VideoFormat,
    pub quality: VideoQuality,
    /// Short-side cap in pixels; None keeps the original size.
    pub max_resolution: Option<u32>,
    pub remove_audio: bool,
}

#[derive(Serialize, Clone)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum VideoProgress {
    Started {
        id: String,
    },
    /// Encode progress 0..1, or None when the duration is unknown.
    Progress {
        id: String,
        fraction: Option<f64>,
    },
    /// Encode finished; fallback check and poster extraction running.
    Finishing {
        id: String,
    },
    Done {
        id: String,
        size_bytes: u64,
        ext: String,
        width: u32,
        height: u32,
        kept_original: bool,
        poster_count: u32,
    },
    Error {
        id: String,
        message: String,
    },
    Cancelled {
        id: String,
    },
}

#[derive(Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct CompressSummary {
    pub done: u32,
    pub failed: u32,
    pub cancelled: u32,
    pub kept_original: u32,
}

#[tauri::command]
pub async fn compress_videos(
    app: AppHandle,
    ids: Vec<String>,
    options: VideoOptions,
    on_progress: Channel<VideoProgress>,
) -> AppResult<CompressSummary> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<VideoState>();
        let ffmpeg = ffmpeg::resolve()?;
        compress_blocking(&state, &ffmpeg, &ids, options, &on_progress)
    })
    .await
    .map_err(|_| AppError::Panic)?
}

/// Stop the queue and kill the running encode (its output is discarded).
#[tauri::command]
pub fn cancel_compress(state: State<'_, VideoState>) {
    state.cancel.store(true, Ordering::SeqCst);
    state.kill_child();
}

/// Clears the `running` flag when the queue ends, even on panic.
struct RunningGuard<'a>(&'a VideoState);

impl Drop for RunningGuard<'_> {
    fn drop(&mut self) {
        self.0.running.store(false, Ordering::SeqCst);
    }
}

pub(crate) fn compress_blocking(
    state: &VideoState,
    ffmpeg: &Path,
    ids: &[String],
    options: VideoOptions,
    on_progress: &Channel<VideoProgress>,
) -> AppResult<CompressSummary> {
    if let Some(cap) = options.max_resolution {
        if !RESOLUTION_RANGE.contains(&cap) {
            return Err(AppError::Invalid(format!("max resolution {cap}")));
        }
    }
    if state.running.swap(true, Ordering::SeqCst) {
        return Err(AppError::VideoBusy);
    }
    let _guard = RunningGuard(state);
    state.cancel.store(false, Ordering::SeqCst);

    let mut summary = CompressSummary::default();
    for id in ids {
        lock(&state.results).remove(id);
        let send = |progress: VideoProgress| {
            let _ = on_progress.send(progress);
        };
        if state.cancel.load(Ordering::SeqCst) {
            send(VideoProgress::Cancelled { id: id.clone() });
            summary.cancelled += 1;
            continue;
        }
        let Some(entry) = lock(&state.items).get(id).cloned() else {
            let message = "Item is no longer in the list".to_string();
            send(VideoProgress::Error { id: id.clone(), message });
            summary.failed += 1;
            continue;
        };
        send(VideoProgress::Started { id: id.clone() });
        match compress_one(state, ffmpeg, &entry, options, &send) {
            Ok(result) => {
                summary.done += 1;
                summary.kept_original += u32::from(result.kept_original);
                send(VideoProgress::Done {
                    id: id.clone(),
                    size_bytes: result.size_bytes,
                    ext: result.ext.to_string(),
                    width: result.width,
                    height: result.height,
                    kept_original: result.kept_original,
                    poster_count: result.posters.len() as u32,
                });
                lock(&state.results).insert(id.clone(), result);
            }
            Err(error) => {
                let _ = std::fs::remove_dir_all(state.item_dir(id));
                if matches!(error, AppError::Cancelled) {
                    summary.cancelled += 1;
                    send(VideoProgress::Cancelled { id: id.clone() });
                } else {
                    summary.failed += 1;
                    send(VideoProgress::Error { id: id.clone(), message: error.to_string() });
                }
            }
        }
    }
    Ok(summary)
}

/// The finished file for one item, before posters are attached.
struct Output {
    path: PathBuf,
    size_bytes: u64,
    dims: (u32, u32),
    kept_original: bool,
}

fn compress_one(
    state: &VideoState,
    ffmpeg: &Path,
    entry: &VideoEntry,
    options: VideoOptions,
    send: &dyn Fn(VideoProgress),
) -> AppResult<VideoResult> {
    let dir = state.item_dir(&entry.id);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir)?;

    let display = entry.probe.display_dims();
    let target = args::scaled_dims(display, options.max_resolution);
    let audio_channels = entry
        .probe
        .audio
        .as_ref()
        .filter(|_| !options.remove_audio)
        .map(|audio| audio.channels);
    let plan = EncodePlan {
        format: options.format,
        preset: formats::preset(options.format, options.quality),
        target,
        hdr: entry.probe.hdr.is_some(),
        audio_channels,
    };
    let encoded = dir.join(format!("encoded.{}", options.format.ext()));
    let id = entry.id.clone();
    ffmpeg::run_tracked(
        state,
        ffmpeg,
        &args::encode_args(&entry.path, &encoded, &plan),
        entry.probe.duration,
        |fraction| send(VideoProgress::Progress { id: id.clone(), fraction }),
    )?;
    send(VideoProgress::Finishing { id: entry.id.clone() });

    let mut output = Output {
        size_bytes: std::fs::metadata(&encoded)?.len(),
        path: encoded,
        dims: target,
        kept_original: false,
    };
    if should_keep_original(entry, options, output.size_bytes) {
        keep_original(state, ffmpeg, entry, options, &dir, &mut output)?;
    }
    let posters = extract_posters(state, ffmpeg, entry, target, &dir)?;
    Ok(VideoResult {
        path: output.path,
        ext: options.format.ext(),
        size_bytes: output.size_bytes,
        width: output.dims.0,
        height: output.dims.1,
        kept_original: output.kept_original,
        posters,
    })
}

/// Re-encoding saved under 5% of an already web-friendly source, without a
/// downscale: the generation loss isn't worth it.
fn should_keep_original(entry: &VideoEntry, options: VideoOptions, encoded_size: u64) -> bool {
    let downscaled = args::is_downscale(entry.probe.display_dims(), options.max_resolution);
    let saved_enough = encoded_size as f64 <= entry.size_bytes as f64 * (1.0 - MIN_SAVING);
    !downscaled
        && !saved_enough
        && formats::fallback_allowed(&entry.path, &entry.probe, options.format, options.remove_audio)
}

/// Swap the re-encode for a stream copy of the source. A failed remux keeps
/// the re-encode; only cancellation is passed on.
fn keep_original(
    state: &VideoState,
    ffmpeg: &Path,
    entry: &VideoEntry,
    options: VideoOptions,
    dir: &Path,
    output: &mut Output,
) -> AppResult<()> {
    let remuxed = dir.join(format!("remux.{}", options.format.ext()));
    let keep_audio = !options.remove_audio && entry.probe.audio.is_some();
    let args = args::remux_args(&entry.path, &remuxed, options.format, keep_audio);
    match ffmpeg::run_tracked(state, ffmpeg, &args, None, |_| {}) {
        Ok(()) => {
            let _ = std::fs::remove_file(&output.path);
            output.size_bytes = std::fs::metadata(&remuxed)?.len();
            output.path = remuxed;
            output.dims = entry.probe.display_dims();
            output.kept_original = true;
            Ok(())
        }
        Err(AppError::Cancelled) => Err(AppError::Cancelled),
        Err(_) => {
            let _ = std::fs::remove_file(&remuxed);
            Ok(())
        }
    }
}

/// Poster frames are a bonus: failures leave the list empty, but a cancel
/// still stops the item.
fn extract_posters(
    state: &VideoState,
    ffmpeg: &Path,
    entry: &VideoEntry,
    target: (u32, u32),
    dir: &Path,
) -> AppResult<Vec<PosterFrame>> {
    let plan = args::poster_plan(entry.probe.duration, entry.probe.fps);
    let posters_dir = dir.join("posters");
    std::fs::create_dir_all(&posters_dir)?;
    let hdr = entry.probe.hdr.is_some();
    let args = args::poster_args(&entry.path, &posters_dir, plan, target, hdr);
    match ffmpeg::run_tracked(state, ffmpeg, &args, None, |_| {}) {
        Ok(()) => Ok(posters::collect(&posters_dir, plan).unwrap_or_default()),
        Err(AppError::Cancelled) => Err(AppError::Cancelled),
        Err(_) => Ok(Vec::new()),
    }
}
