//! Saving compressed videos (copied from the scratch folder, never loaded
//! into memory) and poster frames (re-encoded with the image pipeline).

use std::path::{Path, PathBuf};

use rayon::prelude::*;
use serde::Serialize;
use tauri::{AppHandle, Manager};

use super::save::SaveSummary;
use crate::error::{AppError, AppResult};
use crate::imaging::encode::{self, EncodeOptions};
use crate::naming::OutputNamer;
use crate::state::{lock, PosterFrame, VideoState};
use crate::video::posters;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PosterPreview {
    pub index: u32,
    pub time_sec: f64,
    pub dark: bool,
    pub width: u32,
    pub height: u32,
    pub preview_data_url: String,
}

/// Previews of an item's poster frames, built when the sidebar opens.
#[tauri::command]
pub async fn video_posters(app: AppHandle, id: String) -> AppResult<Vec<PosterPreview>> {
    run_blocking(app, move |state| posters_blocking(state, &id)).await
}

#[tauri::command]
pub async fn save_video(app: AppHandle, id: String, dest_path: String) -> AppResult<()> {
    run_blocking(app, move |state| save_video_blocking(state, &id, Path::new(&dest_path))).await
}

#[tauri::command]
pub async fn save_videos_to_folder(app: AppHandle, ids: Vec<String>, dir: String) -> AppResult<SaveSummary> {
    run_blocking(app, move |state| write_videos_to_folder(state, &ids, Path::new(&dir))).await
}

#[tauri::command]
pub async fn save_poster(
    app: AppHandle,
    id: String,
    index: u32,
    options: EncodeOptions,
    dest_path: String,
) -> AppResult<()> {
    run_blocking(app, move |state| {
        save_poster_blocking(state, &id, index, &options, Path::new(&dest_path))
    })
    .await
}

async fn run_blocking<T: Send + 'static>(
    app: AppHandle,
    job: impl FnOnce(&VideoState) -> AppResult<T> + Send + 'static,
) -> AppResult<T> {
    tauri::async_runtime::spawn_blocking(move || job(&app.state::<VideoState>()))
        .await
        .map_err(|_| AppError::Panic)?
}

pub(crate) fn posters_blocking(state: &VideoState, id: &str) -> AppResult<Vec<PosterPreview>> {
    let frames = lock(&state.results)
        .get(id)
        .map(|result| result.posters.clone())
        .ok_or_else(|| AppError::NotFound(id.to_string()))?;
    frames
        .par_iter()
        .enumerate()
        .map(|(index, frame)| {
            Ok(PosterPreview {
                index: index as u32,
                time_sec: frame.time_sec,
                dark: frame.dark,
                width: frame.width,
                height: frame.height,
                preview_data_url: posters::preview_data_url(frame)?,
            })
        })
        .collect()
}

pub(crate) fn save_video_blocking(state: &VideoState, id: &str, dest: &Path) -> AppResult<()> {
    let source = lock(&state.results)
        .get(id)
        .map(|result| result.path.clone())
        .ok_or_else(|| AppError::NotFound(id.to_string()))?;
    std::fs::copy(source, dest)?;
    Ok(())
}

pub(crate) fn write_videos_to_folder(state: &VideoState, ids: &[String], dir: &Path) -> AppResult<SaveSummary> {
    if !dir.is_dir() {
        return Err(AppError::Invalid(format!("Not a folder: {}", dir.display())));
    }
    let mut summary = SaveSummary::default();
    let mut namer = OutputNamer::with_existing_dir(dir);
    // Pick names under the locks, copy (slow, large files) outside them.
    let jobs: Vec<(PathBuf, String)> = {
        let items = lock(&state.items);
        let results = lock(&state.results);
        ids.iter()
            .filter_map(|id| {
                let Some(result) = results.get(id) else {
                    summary.missing += 1;
                    return None;
                };
                let name = items.get(id).map_or(id.as_str(), |item| item.name.as_str());
                Some((result.path.clone(), namer.unique(name, result.ext)))
            })
            .collect()
    };
    for (source, name) in jobs {
        std::fs::copy(source, dir.join(name))?;
        summary.saved += 1;
    }
    Ok(summary)
}

pub(crate) fn save_poster_blocking(
    state: &VideoState,
    id: &str,
    index: u32,
    options: &EncodeOptions,
    dest: &Path,
) -> AppResult<()> {
    let frame: PosterFrame = lock(&state.results)
        .get(id)
        .and_then(|result| result.posters.get(index as usize).cloned())
        .ok_or_else(|| AppError::NotFound(format!("{id} poster {index}")))?;
    let img = image::open(&frame.path)?;
    let encoded = encode::encode(&img, options, rayon::current_num_threads())?;
    std::fs::write(dest, encoded.bytes)?;
    Ok(())
}
