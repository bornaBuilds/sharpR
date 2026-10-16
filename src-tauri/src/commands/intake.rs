//! Turning picked/dropped paths into a deduplicated list of files. Shared by
//! the image converter and the video list; each passes its own filter.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::Serialize;
use walkdir::WalkDir;

#[derive(Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct AddSummary {
    pub added: u32,
    pub skipped_unsupported: u32,
    pub skipped_duplicate: u32,
    pub failed: u32,
}

/// Expand folders (optionally recursively, in name order, skipping hidden
/// files) and keep the files `is_supported` accepts.
pub fn expand_paths(
    paths: &[String],
    recursive: bool,
    is_supported: fn(&Path) -> bool,
    summary: &mut AddSummary,
) -> Vec<PathBuf> {
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
                if is_supported(entry.path()) {
                    files.push(entry.into_path());
                } else {
                    summary.skipped_unsupported += 1;
                }
            }
        } else if path.is_file() {
            if is_supported(&path) {
                files.push(path);
            } else {
                summary.skipped_unsupported += 1;
            }
        }
    }
    files
}

/// Drop files already in the list (`seen` holds their canonical keys) or
/// repeated within this batch.
pub fn dedupe(mut seen: HashSet<PathBuf>, files: Vec<PathBuf>, summary: &mut AddSummary) -> Vec<PathBuf> {
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

pub fn canonical_key(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

/// File name for display, falling back to `fallback` (the item id).
pub fn display_name(path: &Path, fallback: &str) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| fallback.to_string())
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with('.'))
        .unwrap_or(false)
}
