//! In-memory application state shared by all commands.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use image::RgbaImage;

/// Where an item's original bytes come from.
#[derive(Clone)]
pub enum SourceKind {
    /// A file on disk (selected, picked from a folder, or dropped).
    Path(PathBuf),
    /// Encoded bytes held in memory (e.g. a crop result sent to the converter).
    Memory(Arc<Vec<u8>>),
}

/// A source image registered in the converter list.
pub struct StoredItem {
    pub id: String,
    pub name: String,
    pub source: SourceKind,
    /// Canonical path used to detect duplicates (None for in-memory items).
    pub key: Option<PathBuf>,
}

/// An encoded output waiting to be saved.
#[derive(Clone)]
pub struct ConvertedImage {
    pub bytes: Vec<u8>,
    pub ext: &'static str,
    pub width: u32,
    pub height: u32,
}

/// The single full-resolution image currently open in the crop tab.
pub struct CropSourceState {
    pub id: String,
    pub name: String,
    pub image: RgbaImage,
}

/// A rendered crop: the encoded output plus the pixels it came from (kept so
/// the result can be handed to the converter regardless of output format).
pub struct CropRendered {
    pub encoded: ConvertedImage,
    pub pixels: RgbaImage,
}

#[derive(Default)]
pub struct AppState {
    pub items: Mutex<HashMap<String, StoredItem>>,
    pub results: Mutex<HashMap<String, ConvertedImage>>,
    pub crop_source: Mutex<Option<CropSourceState>>,
    pub crop_results: Mutex<HashMap<String, CropRendered>>,
    pub running: AtomicBool,
    pub cancel: AtomicBool,
    next_id: AtomicU64,
}

impl AppState {
    pub fn next_id(&self, prefix: &str) -> String {
        let n = self.next_id.fetch_add(1, Ordering::Relaxed) + 1;
        format!("{prefix}-{n}")
    }
}

/// Lock a mutex, recovering the data if a previous holder panicked.
pub fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}
