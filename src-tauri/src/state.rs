//! In-memory application state shared by all commands.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Child;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use image::RgbaImage;

use crate::video::probe::ProbeInfo;

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

/// A source video registered in the video list.
#[derive(Clone)]
pub struct VideoEntry {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    /// Canonical path used to detect duplicates.
    pub key: PathBuf,
    pub size_bytes: u64,
    pub probe: ProbeInfo,
}

/// A poster candidate extracted next to the compressed video.
#[derive(Clone)]
pub struct PosterFrame {
    pub path: PathBuf,
    pub time_sec: f64,
    pub dark: bool,
    pub width: u32,
    pub height: u32,
}

/// A compressed video waiting in the scratch folder to be saved.
pub struct VideoResult {
    pub path: PathBuf,
    pub ext: &'static str,
    pub size_bytes: u64,
    pub width: u32,
    pub height: u32,
    /// The re-encode saved too little, so the source was stream-copied.
    pub kept_original: bool,
    pub posters: Vec<PosterFrame>,
}

/// State of the video tab. Separate from [`AppState`] so image conversions
/// and video compressions never block each other.
#[derive(Default)]
pub struct VideoState {
    pub items: Mutex<HashMap<String, VideoEntry>>,
    pub results: Mutex<HashMap<String, VideoResult>>,
    pub running: AtomicBool,
    pub cancel: AtomicBool,
    /// The FFmpeg process currently encoding, so it can be killed.
    pub child: Mutex<Option<Child>>,
    work_root: OnceLock<PathBuf>,
    next_id: AtomicU64,
}

impl VideoState {
    pub fn next_id(&self) -> String {
        let n = self.next_id.fetch_add(1, Ordering::Relaxed) + 1;
        format!("vid-{n}")
    }

    pub fn set_work_root(&self, dir: PathBuf) {
        let _ = self.work_root.set(dir);
    }

    /// This session's scratch folder (see `video::workdir`).
    pub fn work_root(&self) -> PathBuf {
        self.work_root.get().cloned().unwrap_or_else(|| {
            std::env::temp_dir().join(format!("sharpr-video-{}", std::process::id()))
        })
    }

    pub fn item_dir(&self, id: &str) -> PathBuf {
        self.work_root().join(id)
    }

    /// Kill the running FFmpeg process, if any (it is reaped by its job).
    pub fn kill_child(&self) {
        if let Some(child) = lock(&self.child).as_mut() {
            let _ = child.kill();
        }
    }

    /// App exit: stop the encode and delete this session's scratch files.
    /// The child is reaped here because Windows won't delete open files.
    pub fn shutdown(&self) {
        self.cancel.store(true, Ordering::SeqCst);
        if let Some(mut child) = lock(&self.child).take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        let _ = std::fs::remove_dir_all(self.work_root());
    }
}

/// Lock a mutex, recovering the data if a previous holder panicked.
pub fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}
