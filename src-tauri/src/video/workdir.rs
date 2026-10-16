//! Scratch space for encoded videos and poster frames. Videos are far too
//! large to keep in memory like converted images, so every item gets a folder
//! under `<app cache>/video-work/<pid>-<unix time>/<item id>/`.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, Manager};

use crate::state::VideoState;

pub const WORK_DIR: &str = "video-work";
/// Sessions older than this were left behind by a crash or a kill.
const STALE_AFTER: Duration = Duration::from_secs(24 * 60 * 60);

/// Point `VideoState` at this session's folder and clear out old sessions in
/// the background (age-based, because several instances may be running).
pub fn init(app: &AppHandle) {
    let base = app
        .path()
        .app_cache_dir()
        .unwrap_or_else(|_| std::env::temp_dir().join("sharpr"));
    let root = base.join(WORK_DIR);
    let session = session_dir(&root);
    app.state::<VideoState>().set_work_root(session.clone());
    std::thread::spawn(move || purge_stale(&root, &session, STALE_AFTER));
}

pub fn session_dir(root: &Path) -> PathBuf {
    let started = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    root.join(format!("{}-{started}", std::process::id()))
}

/// Remove session folders in `root` older than `max_age`, except `keep`.
/// Returns how many were removed.
pub fn purge_stale(root: &Path, keep: &Path, max_age: Duration) -> usize {
    let Ok(entries) = std::fs::read_dir(root) else {
        return 0;
    };
    let now = SystemTime::now();
    let mut removed = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        let age = entry
            .metadata()
            .and_then(|meta| meta.modified())
            .ok()
            .and_then(|modified| now.duration_since(modified).ok())
            .unwrap_or_default();
        if path != keep && path.is_dir() && age >= max_age && std::fs::remove_dir_all(&path).is_ok() {
            removed += 1;
        }
    }
    removed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn purge_keeps_the_current_session() {
        let root = std::env::temp_dir().join(format!("sharpr-workdir-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let current = session_dir(&root);
        let old = root.join("1-1");
        std::fs::create_dir_all(current.join("vid-1")).unwrap();
        std::fs::create_dir_all(old.join("vid-9")).unwrap();
        // Nothing is a day old yet.
        assert_eq!(purge_stale(&root, &current, STALE_AFTER), 0);
        assert_eq!(purge_stale(&root, &current, Duration::ZERO), 1);
        assert!(current.is_dir());
        assert!(!old.exists());
        std::fs::remove_dir_all(&root).unwrap();
    }
}
