//! Batch conversion with per-item progress streamed over a channel.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicU32, Ordering};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, State};

use crate::error::{AppError, AppResult};
use crate::imaging::encode::{encode, EncodeOptions, OutputFormat};
use crate::imaging::{decode, resize};
use crate::state::{lock, AppState, ConvertedImage, SourceKind};

#[derive(Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConvertOptions {
    pub format: OutputFormat,
    pub quality: u8,
    pub lossless: bool,
    pub max_dimension: Option<u32>,
}

#[derive(Serialize, Clone)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ConvertProgress {
    Started {
        id: String,
    },
    Done {
        id: String,
        size_bytes: u64,
        ext: String,
        width: u32,
        height: u32,
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
pub struct ConvertSummary {
    pub done: u32,
    pub failed: u32,
    pub cancelled: u32,
}

/// Convert the given items in parallel. Progress for each item is streamed
/// through `on_progress`; encoded bytes stay in memory until saved.
#[tauri::command]
pub async fn convert(
    app: AppHandle,
    ids: Vec<String>,
    options: ConvertOptions,
    on_progress: Channel<ConvertProgress>,
) -> AppResult<ConvertSummary> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        convert_blocking(&state, &ids, options, &on_progress)
    })
    .await
    .map_err(|_| AppError::Panic)?
}

/// Stop scheduling new items; items already encoding finish normally.
#[tauri::command]
pub fn cancel_convert(state: State<'_, AppState>) {
    state.cancel.store(true, Ordering::SeqCst);
}

/// Clears the `running` flag when the batch ends, even on panic.
struct RunningGuard<'a>(&'a AppState);

impl Drop for RunningGuard<'_> {
    fn drop(&mut self) {
        self.0.running.store(false, Ordering::SeqCst);
    }
}

pub(crate) fn convert_blocking(
    state: &AppState,
    ids: &[String],
    options: ConvertOptions,
    on_progress: &Channel<ConvertProgress>,
) -> AppResult<ConvertSummary> {
    if state.running.swap(true, Ordering::SeqCst) {
        return Err(AppError::Busy);
    }
    let _guard = RunningGuard(state);
    state.cancel.store(false, Ordering::SeqCst);

    {
        let mut results = lock(&state.results);
        for id in ids {
            results.remove(id);
        }
    }
    let sources: Vec<(String, Option<SourceKind>)> = {
        let items = lock(&state.items);
        ids.iter()
            .map(|id| (id.clone(), items.get(id).map(|item| item.source.clone())))
            .collect()
    };

    let encode_opts = EncodeOptions {
        format: options.format,
        quality: options.quality,
        lossless: options.lossless,
    };
    let cores = rayon::current_num_threads().max(1);
    let threads_per_item = (cores / sources.len().clamp(1, cores)).max(1);

    let done = AtomicU32::new(0);
    let failed = AtomicU32::new(0);
    let cancelled = AtomicU32::new(0);

    sources.par_iter().for_each(|(id, source)| {
        if state.cancel.load(Ordering::SeqCst) {
            cancelled.fetch_add(1, Ordering::Relaxed);
            let _ = on_progress.send(ConvertProgress::Cancelled { id: id.clone() });
            return;
        }
        let Some(source) = source else {
            failed.fetch_add(1, Ordering::Relaxed);
            let _ = on_progress.send(ConvertProgress::Error {
                id: id.clone(),
                message: "Item is no longer in the list".into(),
            });
            return;
        };
        let _ = on_progress.send(ConvertProgress::Started { id: id.clone() });
        match convert_one(source, &encode_opts, options.max_dimension, threads_per_item) {
            Ok(converted) => {
                let message = ConvertProgress::Done {
                    id: id.clone(),
                    size_bytes: converted.bytes.len() as u64,
                    ext: converted.ext.to_string(),
                    width: converted.width,
                    height: converted.height,
                };
                lock(&state.results).insert(id.clone(), converted);
                done.fetch_add(1, Ordering::Relaxed);
                let _ = on_progress.send(message);
            }
            Err(error) => {
                failed.fetch_add(1, Ordering::Relaxed);
                let _ = on_progress.send(ConvertProgress::Error {
                    id: id.clone(),
                    message: error.to_string(),
                });
            }
        }
    });

    Ok(ConvertSummary {
        done: done.into_inner(),
        failed: failed.into_inner(),
        cancelled: cancelled.into_inner(),
    })
}

fn convert_one(
    source: &SourceKind,
    opts: &EncodeOptions,
    max_dimension: Option<u32>,
    threads: usize,
) -> AppResult<ConvertedImage> {
    catch_unwind(AssertUnwindSafe(|| {
        let img = decode::load_oriented(source)?;
        let img = resize::cap_max_dimension(img, max_dimension);
        let encoded = encode(&img, opts, threads)?;
        Ok(ConvertedImage {
            bytes: encoded.bytes,
            ext: encoded.ext,
            width: encoded.width,
            height: encoded.height,
        })
    }))
    .unwrap_or(Err(AppError::Panic))
}
