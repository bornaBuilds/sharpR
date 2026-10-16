//! Video list intake: probe each file with FFmpeg, grab a thumbnail frame and
//! stream list items back to the webview.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;

use image::ImageFormat;
use serde::Serialize;
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, State};

use super::intake::{canonical_key, dedupe, display_name, expand_paths, AddSummary};
use crate::error::{AppError, AppResult};
use crate::imaging::thumbnail;
use crate::state::{lock, VideoEntry, VideoState};
use crate::video::probe::{parse_probe, ProbeInfo};
use crate::video::{args, ffmpeg, formats};

/// FFmpeg processes probing at once. Probing is mostly I/O and process
/// startup, so a few in parallel keeps big drops snappy without thrashing.
const PROBE_PARALLELISM: usize = 4;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VideoItem {
    pub id: String,
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    /// Display size (rotation and pixel aspect applied).
    pub width: u32,
    pub height: u32,
    pub duration_sec: Option<f64>,
    pub video_codec: String,
    pub has_audio: bool,
    pub hdr: bool,
    pub thumb_data_url: String,
}

/// Register video files and/or folders. Items are streamed through `on_item`
/// in selection order as soon as they are probed.
#[tauri::command]
pub async fn add_videos(
    app: AppHandle,
    paths: Vec<String>,
    recursive: bool,
    on_item: Channel<VideoItem>,
) -> AppResult<AddSummary> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<VideoState>();
        let ffmpeg = ffmpeg::resolve()?;
        add_videos_blocking(&state, &ffmpeg, paths, recursive, &on_item)
    })
    .await
    .map_err(|_| AppError::Panic)?
}

#[tauri::command]
pub fn remove_video(state: State<'_, VideoState>, id: String) -> AppResult<()> {
    remove_entries(&state, &[id])
}

#[tauri::command]
pub fn clear_videos(state: State<'_, VideoState>) -> AppResult<()> {
    let ids: Vec<String> = lock(&state.items).keys().cloned().collect();
    remove_entries(&state, &ids)
}

pub(crate) fn add_videos_blocking(
    state: &VideoState,
    ffmpeg: &Path,
    paths: Vec<String>,
    recursive: bool,
    on_item: &Channel<VideoItem>,
) -> AppResult<AddSummary> {
    let mut summary = AddSummary::default();
    let files = expand_paths(&paths, recursive, formats::is_supported, &mut summary);
    let seen: HashSet<PathBuf> = lock(&state.items).values().map(|e| e.key.clone()).collect();
    let unique = dedupe(seen, files, &mut summary);
    for chunk in unique.chunks(PROBE_PARALLELISM) {
        let built: Vec<AppResult<(VideoEntry, VideoItem)>> = std::thread::scope(|scope| {
            let handles: Vec<_> = chunk
                .iter()
                .map(|path| scope.spawn(|| build_video(state, ffmpeg, path)))
                .collect();
            handles
                .into_iter()
                .map(|handle| handle.join().unwrap_or(Err(AppError::Panic)))
                .collect()
        });
        for result in built {
            match result {
                Ok((entry, item)) => {
                    lock(&state.items).insert(entry.id.clone(), entry);
                    summary.added += 1;
                    let _ = on_item.send(item);
                }
                Err(_) => summary.failed += 1,
            }
        }
    }
    Ok(summary)
}

pub(crate) fn remove_entries(state: &VideoState, ids: &[String]) -> AppResult<()> {
    if state.running.load(Ordering::SeqCst) {
        return Err(AppError::VideoBusy);
    }
    for id in ids {
        lock(&state.items).remove(id);
        lock(&state.results).remove(id);
        let _ = std::fs::remove_dir_all(state.item_dir(id));
    }
    Ok(())
}

fn build_video(state: &VideoState, ffmpeg: &Path, path: &Path) -> AppResult<(VideoEntry, VideoItem)> {
    let size_bytes = std::fs::metadata(path)?.len();
    let output = ffmpeg::run_capture(ffmpeg, &args::probe_args(path))?;
    let probe = parse_probe(&String::from_utf8_lossy(&output.stderr))?;
    let thumb_data_url = thumbnail_data_url(ffmpeg, path, &probe)?;
    let (width, height) = probe.display_dims();
    let id = state.next_id();
    let name = display_name(path, &id);
    let item = VideoItem {
        id: id.clone(),
        name: name.clone(),
        path: path.to_string_lossy().into_owned(),
        size_bytes,
        width,
        height,
        duration_sec: probe.duration,
        video_codec: probe.video_codec.clone(),
        has_audio: probe.audio.is_some(),
        hdr: probe.hdr.is_some(),
        thumb_data_url,
    };
    let entry = VideoEntry {
        id,
        name,
        path: path.to_path_buf(),
        key: canonical_key(path),
        size_bytes,
        probe,
    };
    Ok((entry, item))
}

/// A frame shortly after the start, falling back to the very first frame
/// when the seek lands past the end (wrong or missing duration).
fn thumbnail_data_url(ffmpeg: &Path, path: &Path, probe: &ProbeInfo) -> AppResult<String> {
    let hdr = probe.hdr.is_some();
    let seek = args::thumbnail_seek(probe.duration);
    let mut png = grab_frame(ffmpeg, path, seek, hdr)?;
    if png.is_empty() && seek > 0.0 {
        png = grab_frame(ffmpeg, path, 0.0, hdr)?;
    }
    if png.is_empty() {
        return Err(AppError::Ffmpeg("could not read a video frame".to_string()));
    }
    let frame = image::load_from_memory_with_format(&png, ImageFormat::Png)?;
    thumbnail::thumbnail_data_url(&frame)
}

fn grab_frame(ffmpeg: &Path, path: &Path, seek: f64, hdr: bool) -> AppResult<Vec<u8>> {
    let output = ffmpeg::run_capture(ffmpeg, &args::thumbnail_args(path, seek, hdr))?;
    Ok(if output.status.success() { output.stdout } else { Vec::new() })
}
