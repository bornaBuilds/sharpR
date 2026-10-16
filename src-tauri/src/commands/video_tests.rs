//! Video pipeline tests against the real FFmpeg sidecar, on clips generated
//! with lavfi. They skip (with a message) when the sidecar isn't installed.

use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::Value;
use tauri::ipc::Channel;

use super::video_compress::{compress_blocking, CompressSummary, VideoOptions, VideoProgress};
use super::video_intake::{add_videos_blocking, remove_entries, VideoItem};
use super::video_save::{posters_blocking, save_poster_blocking, write_videos_to_folder};
use crate::imaging::encode::{EncodeOptions, OutputFormat};
use crate::state::{lock, VideoState};
use crate::video::args;
use crate::video::ffmpeg::run_capture;
use crate::video::formats::{VideoFormat, VideoQuality};
use crate::video::probe::{parse_probe, ProbeInfo};
use crate::video::testing::{self, make_clip, rotate_tag, Clip};

fn to_str(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn options(format: VideoFormat) -> VideoOptions {
    VideoOptions {
        format,
        quality: VideoQuality::Balanced,
        max_resolution: None,
        remove_audio: false,
    }
}

/// Probe a file; also returns the raw summary for checks the parser skips.
fn probe(ffmpeg: &Path, path: &Path) -> (ProbeInfo, String) {
    let output = run_capture(ffmpeg, &args::probe_args(path)).unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    (parse_probe(&stderr).unwrap(), stderr)
}

/// Add `files` to a fresh state, compress them and return everything needed
/// to inspect the outcome.
struct Run {
    state: VideoState,
    summary: CompressSummary,
    events: Vec<Value>,
    ids: Vec<String>,
}

fn add(state: &VideoState, ffmpeg: &Path, files: &[PathBuf]) -> Vec<String> {
    let (channel, seen) = collecting_channel::<VideoItem>();
    let paths = files.iter().map(|f| to_str(f)).collect();
    add_videos_blocking(state, ffmpeg, paths, false, &channel).unwrap();
    let items = lock(&seen).clone();
    items.iter().map(|i| i["id"].as_str().unwrap().to_string()).collect()
}

fn compress(ffmpeg: &Path, dir: &Path, files: &[PathBuf], options: VideoOptions) -> Run {
    let state = state_in(dir);
    let ids = add(&state, ffmpeg, files);
    let (channel, seen) = collecting_channel::<VideoProgress>();
    let summary = compress_blocking(&state, ffmpeg, &ids, options, &channel).unwrap();
    let events = lock(&seen).clone();
    Run { state, summary, events, ids }
}

fn result_path(state: &VideoState, id: &str) -> PathBuf {
    lock(&state.results)[id].path.clone()
}

fn done_event<'a>(events: &'a [Value], id: &str) -> &'a Value {
    events
        .iter()
        .find(|e| e["type"] == "done" && e["id"] == id)
        .unwrap_or_else(|| panic!("no done event for {id}: {events:?}"))
}

/// A channel that keeps every message it receives.
fn collecting_channel<T: serde::Serialize + Clone + Send + 'static>() -> (Channel<T>, Arc<Mutex<Vec<serde_json::Value>>>) {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&seen);
    let channel = Channel::new(move |body| {
        if let tauri::ipc::InvokeResponseBody::Json(json) = body {
            lock(&sink).push(serde_json::from_str(&json).unwrap());
        }
        Ok(())
    });
    (channel, seen)
}

fn state_in(dir: &Path) -> VideoState {
    let state = VideoState::default();
    state.set_work_root(dir.join("work"));
    state
}

#[test]
fn intake_probes_folders_and_skips_other_files() {
    let Some(ffmpeg) = testing::ffmpeg() else {
        return;
    };
    let dir = testing::scratch("intake");
    std::fs::create_dir_all(dir.join("sub")).unwrap();
    make_clip(&ffmpeg, &dir.join("a.mp4"), &Clip::default());
    let plain = Clip { audio: false, ..Clip::default() };
    make_clip(&ffmpeg, &dir.join("sub").join("b.mov"), &plain);
    rotate_tag(&ffmpeg, &dir.join("sub").join("b.mov"), &dir.join("sub").join("c.mov"), 90);
    std::fs::write(dir.join("notes.txt"), b"not a video").unwrap();
    std::fs::write(dir.join("broken.mp4"), b"not a video either").unwrap();

    let state = state_in(&dir);
    let (channel, seen) = collecting_channel::<VideoItem>();
    let summary = add_videos_blocking(&state, &ffmpeg, vec![to_str(&dir)], true, &channel).unwrap();
    assert_eq!(summary.added, 3);
    assert_eq!(summary.skipped_unsupported, 1);
    assert_eq!(summary.failed, 1, "broken.mp4 fails to probe");

    let items = lock(&seen).clone();
    let names: Vec<&str> = items.iter().map(|i| i["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["a.mp4", "b.mov", "c.mov"]);
    let a = &items[0];
    assert_eq!((a["width"].as_u64(), a["height"].as_u64()), (Some(640), Some(360)));
    assert!((a["durationSec"].as_f64().unwrap() - 2.0).abs() < 0.1);
    assert_eq!(a["videoCodec"], "h264");
    assert_eq!(a["hasAudio"], true);
    assert_eq!(items[1]["hasAudio"], false);
    assert!(a["thumbDataUrl"].as_str().unwrap().starts_with("data:image/jpeg;base64,"));
    // The rotation tag swaps the displayed size.
    assert_eq!((items[2]["width"].as_u64(), items[2]["height"].as_u64()), (Some(360), Some(640)));

    // Adding the folder again only finds duplicates.
    let (channel, _) = collecting_channel::<VideoItem>();
    let again = add_videos_blocking(&state, &ffmpeg, vec![to_str(&dir)], true, &channel).unwrap();
    assert_eq!((again.added, again.skipped_duplicate), (0, 3));

    let ids: Vec<String> = lock(&state.items).keys().cloned().collect();
    remove_entries(&state, &ids).unwrap();
    assert!(lock(&state.items).is_empty());
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn mp4_compression_reports_progress_and_posters() {
    let Some(ffmpeg) = testing::ffmpeg() else {
        return;
    };
    let dir = testing::scratch("mp4");
    let source = dir.join("source.mp4");
    make_clip(&ffmpeg, &source, &Clip::default());
    let run = compress(&ffmpeg, &dir, std::slice::from_ref(&source), options(VideoFormat::Mp4));
    assert_eq!((run.summary.done, run.summary.failed), (1, 0));
    let id = &run.ids[0];
    let types: Vec<&str> = run.events.iter().map(|e| e["type"].as_str().unwrap()).collect();
    assert_eq!(types.first(), Some(&"started"));
    assert!(types.contains(&"progress") && types.contains(&"finishing"));
    let done = done_event(&run.events, id);
    assert_eq!(done["keptOriginal"], false);
    assert_eq!(done["ext"], "mp4");
    assert_eq!(done["posterCount"], 8);

    let output = result_path(&run.state, id);
    let size = std::fs::metadata(&output).unwrap().len();
    assert_eq!(done["sizeBytes"].as_u64(), Some(size));
    assert!(size < std::fs::metadata(&source).unwrap().len());
    let (info, _) = probe(&ffmpeg, &output);
    assert_eq!(info.video_codec, "h264");
    assert_eq!((info.width, info.height), (640, 360));
    assert_eq!(info.audio.map(|a| a.codec), Some("aac".to_string()));

    let previews = posters_blocking(&run.state, id).unwrap();
    assert_eq!(previews.len(), 8);
    assert_eq!((previews[0].width, previews[0].height), (640, 360));
    assert!(previews.windows(2).all(|w| w[0].time_sec < w[1].time_sec));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn audio_removal_downscale_and_rotation() {
    let Some(ffmpeg) = testing::ffmpeg() else {
        return;
    };
    let dir = testing::scratch("transform");
    let portrait = dir.join("portrait.mp4");
    make_clip(&ffmpeg, &portrait, &Clip { size: (720, 1280), ..Clip::default() });
    let landscape = dir.join("landscape.mp4");
    make_clip(&ffmpeg, &landscape, &Clip::default());
    let rotated = dir.join("rotated.mov");
    rotate_tag(&ffmpeg, &landscape, &rotated, 90);

    let opts = VideoOptions {
        max_resolution: Some(480),
        remove_audio: true,
        ..options(VideoFormat::Mp4)
    };
    let run = compress(&ffmpeg, &dir, &[portrait, rotated], opts);
    assert_eq!(run.summary.done, 2, "{:?}", run.events);
    // Portrait 720x1280 capped at 480 on the short side.
    let (info, _) = probe(&ffmpeg, &result_path(&run.state, &run.ids[0]));
    assert_eq!((info.width, info.height), (480, 854));
    assert!(info.audio.is_none());
    // The rotation tag is baked into the pixels: upright, no tag left.
    let (info, _) = probe(&ffmpeg, &result_path(&run.state, &run.ids[1]));
    assert_eq!((info.width, info.height, info.rotation), (360, 640, 0));
    let done = done_event(&run.events, &run.ids[1]);
    assert_eq!((done["width"].as_u64(), done["height"].as_u64()), (Some(360), Some(640)));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn well_compressed_mp4_keeps_the_original() {
    let Some(ffmpeg) = testing::ffmpeg() else {
        return;
    };
    let dir = testing::scratch("keep");
    let source = dir.join("small.mp4");
    let clip = Clip {
        video_args: &["-c:v", "libx264", "-crf", "45", "-pix_fmt", "yuv420p"],
        ..Clip::default()
    };
    make_clip(&ffmpeg, &source, &clip);
    let run = compress(&ffmpeg, &dir, std::slice::from_ref(&source), options(VideoFormat::Mp4));
    assert_eq!((run.summary.done, run.summary.kept_original), (1, 1));
    let done = done_event(&run.events, &run.ids[0]);
    assert_eq!(done["keptOriginal"], true);
    let output = result_path(&run.state, &run.ids[0]);
    assert!(output.ends_with("remux.mp4"));
    let source_size = std::fs::metadata(&source).unwrap().len() as f64;
    let output_size = std::fs::metadata(&output).unwrap().len() as f64;
    assert!((output_size - source_size).abs() / source_size < 0.05);
    assert_eq!(probe(&ffmpeg, &output).0.video_codec, "h264");
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn webm_output_uses_vp9_and_opus() {
    let Some(ffmpeg) = testing::ffmpeg() else {
        return;
    };
    let dir = testing::scratch("webm");
    let source = dir.join("source.mp4");
    make_clip(&ffmpeg, &source, &Clip::default());
    let opts = VideoOptions {
        quality: VideoQuality::Smallest,
        ..options(VideoFormat::Webm)
    };
    let run = compress(&ffmpeg, &dir, &[source], opts);
    assert_eq!(run.summary.done, 1, "{:?}", run.events);
    let output = result_path(&run.state, &run.ids[0]);
    let (info, _) = probe(&ffmpeg, &output);
    assert_eq!(info.video_codec, "vp9");
    assert_eq!(info.audio.map(|a| a.codec), Some("opus".to_string()));
    assert_eq!(done_event(&run.events, &run.ids[0])["keptOriginal"], false);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn hlg_sources_are_tonemapped_to_bt709() {
    let Some(ffmpeg) = testing::ffmpeg() else {
        return;
    };
    let dir = testing::scratch("hlg");
    let source = dir.join("hlg.mov");
    let clip = Clip {
        audio: false,
        video_args: &[
            "-vf",
            "setparams=color_primaries=bt2020:color_trc=arib-std-b67:colorspace=bt2020nc,format=yuv420p10le",
            "-c:v",
            "libx265",
            "-x265-params",
            "colorprim=bt2020:transfer=arib-std-b67:colormatrix=bt2020nc:log-level=error",
        ],
        ..Clip::default()
    };
    make_clip(&ffmpeg, &source, &clip);
    let state = state_in(&dir);
    let ids = add(&state, &ffmpeg, &[source]);
    assert_eq!(lock(&state.items)[&ids[0]].probe.hdr, Some(crate::video::probe::Hdr::Hlg));
    let (channel, seen) = collecting_channel::<VideoProgress>();
    let summary = compress_blocking(&state, &ffmpeg, &ids, options(VideoFormat::Mp4), &channel).unwrap();
    assert_eq!(summary.done, 1, "{:?}", lock(&seen));
    let (info, stderr) = probe(&ffmpeg, &result_path(&state, &ids[0]));
    assert_eq!((info.hdr, info.pix_fmt.as_str()), (None, "yuv420p"));
    assert!(stderr.contains("bt709"), "{stderr}");
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn saving_videos_and_posters() {
    let Some(ffmpeg) = testing::ffmpeg() else {
        return;
    };
    let dir = testing::scratch("save");
    let source = dir.join("clip.mov");
    make_clip(&ffmpeg, &source, &Clip { seconds: 1.0, ..Clip::default() });
    let run = compress(&ffmpeg, &dir, &[source], options(VideoFormat::Mp4));
    let out = dir.join("out");
    std::fs::create_dir_all(&out).unwrap();
    let ids = vec![run.ids[0].clone(), "vid-missing".to_string()];
    let first = write_videos_to_folder(&run.state, &ids, &out).unwrap();
    assert_eq!((first.saved, first.missing), (1, 1));
    write_videos_to_folder(&run.state, &ids, &out).unwrap();
    assert!(out.join("clip.mp4").is_file() && out.join("clip-1.mp4").is_file());

    let poster = out.join("clip-poster.webp");
    let webp = EncodeOptions { format: OutputFormat::Webp, quality: 80, lossless: false };
    save_poster_blocking(&run.state, &run.ids[0], 0, &webp, &poster).unwrap();
    let bytes = std::fs::read(&poster).unwrap();
    assert_eq!(&bytes[0..4], b"RIFF");
    assert!(save_poster_blocking(&run.state, &run.ids[0], 99, &webp, &poster).is_err());
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn cancel_kills_the_encode_and_cleans_up() {
    let Some(ffmpeg) = testing::ffmpeg() else {
        return;
    };
    let dir = testing::scratch("cancel");
    let long = dir.join("long.mp4");
    let clip = Clip {
        size: (1280, 720),
        seconds: 20.0,
        video_args: &["-c:v", "libx264", "-preset", "ultrafast", "-crf", "16", "-pix_fmt", "yuv420p"],
        ..Clip::default()
    };
    make_clip(&ffmpeg, &long, &clip);
    let second = dir.join("second.mp4");
    make_clip(&ffmpeg, &second, &Clip::default());
    let state = state_in(&dir);
    let ids = add(&state, &ffmpeg, &[long, second]);
    let (channel, seen) = collecting_channel::<VideoProgress>();
    let opts = VideoOptions { quality: VideoQuality::High, ..options(VideoFormat::Webm) };

    let summary = std::thread::scope(|scope| {
        let job = scope.spawn(|| compress_blocking(&state, &ffmpeg, &ids, opts, &channel));
        while !lock(&seen).iter().any(|e| e["type"] == "progress") {
            assert!(!job.is_finished(), "encode finished before it could be cancelled");
            std::thread::sleep(Duration::from_millis(20));
        }
        state.cancel.store(true, Ordering::SeqCst);
        state.kill_child();
        job.join().unwrap().unwrap()
    });
    assert_eq!((summary.done, summary.cancelled), (0, 2));
    assert!(!state.item_dir(&ids[0]).exists());
    assert!(lock(&state.child).is_none());
    assert!(!state.running.load(Ordering::SeqCst));
    assert!(lock(&state.results).is_empty());
    std::fs::remove_dir_all(&dir).unwrap();
}
