//! File intake for the converter list: expand folders, filter, dedupe,
//! decode in parallel and stream list items back to the webview.

use std::collections::HashSet;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};

use image::GenericImageView;
use rayon::prelude::*;
use serde::Serialize;
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, State};
use walkdir::WalkDir;

use crate::error::{AppError, AppResult};
use crate::imaging::{decode, thumbnail};
use crate::state::{lock, AppState, SourceKind, StoredItem};

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConvertedInfo {
    pub size_bytes: u64,
    pub ext: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImageItem {
    pub id: String,
    pub name: String,
    pub path: Option<String>,
    pub size_bytes: u64,
    pub width: u32,
    pub height: u32,
    pub thumb_data_url: String,
    pub preconverted: Option<ConvertedInfo>,
}

#[derive(Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct AddSummary {
    pub added: u32,
    pub skipped_unsupported: u32,
    pub skipped_duplicate: u32,
    pub failed: u32,
}

/// Register files and/or folders. Items are streamed through `on_item` as
/// soon as their thumbnail is ready (in selection order, chunked).
#[tauri::command]
pub async fn add_paths(
    app: AppHandle,
    paths: Vec<String>,
    recursive: bool,
    on_item: Channel<ImageItem>,
) -> AppResult<AddSummary> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        add_paths_blocking(&state, paths, recursive, &on_item)
    })
    .await
    .map_err(|_| AppError::Panic)?
}

#[tauri::command]
pub fn remove_item(state: State<'_, AppState>, id: String) -> AppResult<()> {
    lock(&state.items).remove(&id);
    lock(&state.results).remove(&id);
    Ok(())
}

pub(crate) fn add_paths_blocking(
    state: &AppState,
    paths: Vec<String>,
    recursive: bool,
    on_item: &Channel<ImageItem>,
) -> AppResult<AddSummary> {
    let mut summary = AddSummary::default();
    let files = expand_paths(&paths, recursive, &mut summary);
    let unique = dedupe(state, files, &mut summary);

    // Decode in parallel per chunk, but emit chunks in order so the list
    // follows the user's selection order while still appearing progressively.
    let chunk_size = rayon::current_num_threads().max(2) * 2;
    for chunk in unique.chunks(chunk_size) {
        let built: Vec<AppResult<(StoredItem, ImageItem)>> =
            chunk.par_iter().map(|path| build_item(state, path)).collect();
        for result in built {
            match result {
                Ok((stored, item)) => {
                    lock(&state.items).insert(stored.id.clone(), stored);
                    summary.added += 1;
                    let _ = on_item.send(item);
                }
                Err(_) => summary.failed += 1,
            }
        }
    }
    Ok(summary)
}

fn expand_paths(paths: &[String], recursive: bool, summary: &mut AddSummary) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for raw in paths {
        let path = PathBuf::from(raw);
        if path.is_dir() {
            let max_depth = if recursive { usize::MAX } else { 1 };
            let walker = WalkDir::new(&path)
                .max_depth(max_depth)
                .follow_links(false)
                .sort_by_file_name();
            for entry in walker.into_iter().filter_map(Result::ok) {
                if !entry.file_type().is_file() || is_hidden(entry.path()) {
                    continue;
                }
                if decode::is_supported(entry.path()) {
                    files.push(entry.into_path());
                } else {
                    summary.skipped_unsupported += 1;
                }
            }
        } else if path.is_file() {
            if decode::is_supported(&path) {
                files.push(path);
            } else {
                summary.skipped_unsupported += 1;
            }
        }
    }
    files
}

fn dedupe(state: &AppState, files: Vec<PathBuf>, summary: &mut AddSummary) -> Vec<PathBuf> {
    let mut seen: HashSet<PathBuf> = lock(&state.items)
        .values()
        .filter_map(|item| item.key.clone())
        .collect();
    let mut unique = Vec::with_capacity(files.len());
    for file in files {
        let key = canonical_key(&file);
        if seen.insert(key) {
            unique.push(file);
        } else {
            summary.skipped_duplicate += 1;
        }
    }
    unique
}

fn canonical_key(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with('.'))
        .unwrap_or(false)
}

fn build_item(state: &AppState, path: &Path) -> AppResult<(StoredItem, ImageItem)> {
    catch_unwind(AssertUnwindSafe(|| build_item_inner(state, path))).unwrap_or(Err(AppError::Panic))
}

fn build_item_inner(state: &AppState, path: &Path) -> AppResult<(StoredItem, ImageItem)> {
    let size_bytes = std::fs::metadata(path)?.len();
    let source = SourceKind::Path(path.to_path_buf());
    let img = decode::load_oriented(&source)?;
    let (width, height) = img.dimensions();
    let thumb_data_url = thumbnail::thumbnail_data_url(&img)?;
    let id = state.next_id("img");
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| id.clone());

    let stored = StoredItem {
        id: id.clone(),
        name: name.clone(),
        source,
        key: Some(canonical_key(path)),
    };
    let item = ImageItem {
        id,
        name,
        path: Some(path.to_string_lossy().into_owned()),
        size_bytes,
        width,
        height,
        thumb_data_url,
        preconverted: None,
    };
    Ok((stored, item))
}
