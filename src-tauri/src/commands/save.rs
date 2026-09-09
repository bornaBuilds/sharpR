//! Writing converted results to disk: single file, ZIP archive, or a folder.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::{AppHandle, Manager};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::error::{AppError, AppResult};
use crate::naming::OutputNamer;
use crate::state::{lock, AppState};

#[derive(Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct SaveSummary {
    pub saved: u32,
    pub missing: u32,
}

#[tauri::command]
pub async fn save_item(app: AppHandle, id: String, dest_path: String) -> AppResult<()> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let bytes = lock(&state.results)
            .get(&id)
            .map(|r| r.bytes.clone())
            .ok_or_else(|| AppError::NotFound(id.clone()))?;
        std::fs::write(&dest_path, bytes)?;
        Ok(())
    })
    .await
    .map_err(|_| AppError::Panic)?
}

#[tauri::command]
pub async fn save_zip(app: AppHandle, ids: Vec<String>, dest_path: String) -> AppResult<SaveSummary> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        write_zip(&state, &ids, Path::new(&dest_path))
    })
    .await
    .map_err(|_| AppError::Panic)?
}

#[tauri::command]
pub async fn save_to_folder(app: AppHandle, ids: Vec<String>, dir: String) -> AppResult<SaveSummary> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        write_folder(&state, &ids, Path::new(&dir))
    })
    .await
    .map_err(|_| AppError::Panic)?
}

pub(crate) fn write_zip(state: &AppState, ids: &[String], dest: &Path) -> AppResult<SaveSummary> {
    let file = File::create(dest)?;
    let mut zip = ZipWriter::new(BufWriter::new(file));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let mut namer = OutputNamer::new();
    let mut summary = SaveSummary::default();

    let items = lock(&state.items);
    let results = lock(&state.results);
    for id in ids {
        let Some(result) = results.get(id) else {
            summary.missing += 1;
            continue;
        };
        let source_name = items.get(id).map(|i| i.name.as_str()).unwrap_or(id);
        zip.start_file(namer.unique(source_name, result.ext), options)?;
        zip.write_all(&result.bytes)?;
        summary.saved += 1;
    }
    zip.finish()?.flush()?;
    Ok(summary)
}

pub(crate) fn write_folder(state: &AppState, ids: &[String], dir: &Path) -> AppResult<SaveSummary> {
    if !dir.is_dir() {
        return Err(AppError::Invalid(format!("Not a folder: {}", dir.display())));
    }
    let mut namer = OutputNamer::with_existing_dir(dir);
    let mut summary = SaveSummary::default();

    let items = lock(&state.items);
    let results = lock(&state.results);
    for id in ids {
        let Some(result) = results.get(id) else {
            summary.missing += 1;
            continue;
        };
        let source_name = items.get(id).map(|i| i.name.as_str()).unwrap_or(id);
        let target: PathBuf = dir.join(namer.unique(source_name, result.ext));
        std::fs::write(target, &result.bytes)?;
        summary.saved += 1;
    }
    Ok(summary)
}
