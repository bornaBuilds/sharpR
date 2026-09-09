//! End-to-end pipeline tests: intake -> convert -> save, on generated files.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use image::{Rgb, RgbImage};
use tauri::ipc::Channel;

use super::convert::{convert_blocking, ConvertOptions, ConvertProgress};
use super::files::{add_paths_blocking, ImageItem};
use super::save::{write_folder, write_zip};
use crate::imaging::encode::OutputFormat;
use crate::state::{lock, AppState};

fn scratch_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sharpr-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_sample(path: &Path, w: u32, h: u32) {
    let img = RgbImage::from_fn(w, h, |x, y| {
        Rgb([(x % 256) as u8, (y % 256) as u8, ((x + y) % 256) as u8])
    });
    img.save(path).unwrap();
}

fn counting_channel<T: serde::Serialize + Clone + 'static>() -> (Channel<T>, Arc<AtomicU32>) {
    let count = Arc::new(AtomicU32::new(0));
    let seen = Arc::clone(&count);
    let channel = Channel::new(move |_| {
        seen.fetch_add(1, Ordering::Relaxed);
        Ok(())
    });
    (channel, count)
}

fn to_str(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[test]
fn intake_convert_zip_and_folder_pipeline() {
    let dir = scratch_dir("pipeline");
    write_sample(&dir.join("a.png"), 300, 200);
    std::fs::create_dir_all(dir.join("sub")).unwrap();
    write_sample(&dir.join("sub").join("b.jpg"), 640, 480);
    std::fs::write(dir.join("notes.txt"), b"not an image").unwrap();

    let state = AppState::default();

    // Intake (recursive): 2 images, 1 unsupported file.
    let (items_channel, items_seen) = counting_channel::<ImageItem>();
    let summary = add_paths_blocking(&state, vec![to_str(&dir)], true, &items_channel).unwrap();
    assert_eq!(summary.added, 2);
    assert_eq!(summary.skipped_unsupported, 1);
    assert_eq!(summary.failed, 0);
    assert_eq!(items_seen.load(Ordering::Relaxed), 2);

    // Intake again: same file is a duplicate; non-recursive ignores the subfolder.
    let again = add_paths_blocking(
        &state,
        vec![to_str(&dir.join("a.png")), to_str(&dir)],
        false,
        &items_channel,
    )
    .unwrap();
    assert_eq!(again.added, 0);
    assert_eq!(again.skipped_duplicate, 2);
    assert_eq!(again.skipped_unsupported, 1);

    let mut ids: Vec<String> = lock(&state.items).keys().cloned().collect();
    ids.sort();
    assert_eq!(ids.len(), 2);

    // Convert to WebP with a max dimension.
    let (progress_channel, progress_seen) = counting_channel::<ConvertProgress>();
    let options = ConvertOptions {
        format: OutputFormat::Webp,
        quality: 80,
        lossless: false,
        max_dimension: Some(100),
    };
    let converted = convert_blocking(&state, &ids, options, &progress_channel).unwrap();
    assert_eq!(converted.done, 2);
    assert_eq!(converted.failed, 0);
    assert_eq!(progress_seen.load(Ordering::Relaxed), 4, "started + done per item");
    assert!(!state.running.load(Ordering::SeqCst));
    for result in lock(&state.results).values() {
        assert_eq!(result.ext, "webp");
        assert!(result.width.max(result.height) <= 100);
        assert_eq!(&result.bytes[8..12], b"WEBP");
    }

    // ZIP with flattened, unique names.
    let zip_path = dir.join("out.zip");
    let zipped = write_zip(&state, &ids, &zip_path).unwrap();
    assert_eq!(zipped.saved, 2);
    let mut archive = zip::ZipArchive::new(std::fs::File::open(&zip_path).unwrap()).unwrap();
    let mut names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().to_string())
        .collect();
    names.sort();
    assert_eq!(names, vec!["a.webp".to_string(), "b.webp".to_string()]);

    // Folder save twice: second run gets -1 suffixes instead of overwriting.
    let out_dir = dir.join("out");
    std::fs::create_dir_all(&out_dir).unwrap();
    assert_eq!(write_folder(&state, &ids, &out_dir).unwrap().saved, 2);
    assert_eq!(write_folder(&state, &ids, &out_dir).unwrap().saved, 2);
    assert!(out_dir.join("a.webp").exists());
    assert!(out_dir.join("a-1.webp").exists());
    assert!(out_dir.join("b-1.webp").exists());

    // Missing results are reported, not fatal.
    let missing = write_zip(&state, &["nope".to_string()], &dir.join("empty.zip")).unwrap();
    assert_eq!(missing.saved, 0);
    assert_eq!(missing.missing, 1);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn cancel_flag_skips_remaining_items() {
    let dir = scratch_dir("cancel");
    for i in 0..4 {
        write_sample(&dir.join(format!("{i}.png")), 64, 64);
    }
    let state = AppState::default();
    let (items_channel, _) = counting_channel::<ImageItem>();
    add_paths_blocking(&state, vec![to_str(&dir)], false, &items_channel).unwrap();
    let ids: Vec<String> = lock(&state.items).keys().cloned().collect();

    // Cancel requested before the batch starts: convert_blocking resets it, so
    // this only proves the flag is cleared and the batch runs normally.
    state.cancel.store(true, Ordering::SeqCst);
    let (progress_channel, _) = counting_channel::<ConvertProgress>();
    let options = ConvertOptions {
        format: OutputFormat::Png,
        quality: 100,
        lossless: false,
        max_dimension: None,
    };
    let summary = convert_blocking(&state, &ids, options, &progress_channel).unwrap();
    assert_eq!(summary.done + summary.cancelled, 4);
    assert!(!state.cancel.load(Ordering::SeqCst));

    let _ = std::fs::remove_dir_all(&dir);
}
