//! FFmpeg argument lists. Pure: every command sharpR runs is built here and
//! unit-tested. Paths are passed as separate argv entries, never through a
//! shell.

use std::ffi::OsString;
use std::path::{Path, MAIN_SEPARATOR};

use super::formats::{Preset, VideoFormat};

/// HDR (PQ/HLG) to SDR BT.709, so phone footage doesn't look washed out.
const TONEMAP: &str = "zscale=t=linear:npl=100,format=gbrpf32le,zscale=p=bt709,\
tonemap=tonemap=hable:desat=0,zscale=t=bt709:m=bt709:r=tv";
/// Longest side of the list-row thumbnail frame.
pub const THUMB_PX: u32 = 512;
pub const POSTER_PATTERN: &str = "poster-%02d.png";
const POSTER_COUNT: u32 = 8;
const POSTER_START: f64 = 0.5;
const POSTER_WINDOW: f64 = 5.0;
/// Below this duration posters are spread over the whole clip.
const SHORT_CLIP: f64 = 1.5;
const FALLBACK_FPS: f64 = 30.0;

/// Everything needed to build the encode command for one item.
#[derive(Clone, Copy, Debug)]
pub struct EncodePlan {
    pub format: VideoFormat,
    pub preset: Preset,
    /// Exact output size (even numbers), see [`scaled_dims`].
    pub target: (u32, u32),
    pub hdr: bool,
    /// Channel count of the source audio to keep; None drops audio.
    pub audio_channels: Option<u32>,
}

/// Which stretch of the source the poster frames are taken from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PosterPlan {
    pub start: f64,
    pub window: f64,
    pub count: u32,
}

impl PosterPlan {
    /// Approximate timestamp of frame `index` in the source.
    pub fn time_of(&self, index: u32) -> f64 {
        self.start + f64::from(index) * self.window / f64::from(self.count)
    }
}

/// Output size for a display size and an optional short-side cap. Never
/// upscales; dimensions are even (required by yuv420p) and the long side is
/// rounded like FFmpeg's `-2`.
pub fn scaled_dims(display: (u32, u32), cap: Option<u32>) -> (u32, u32) {
    let even = |v: u32| (v & !1).max(2);
    let (w, h) = display;
    let short = w.min(h);
    match cap {
        Some(cap) if short > cap => {
            let new_short = even(cap);
            let long = f64::from(w.max(h));
            let half_long = (f64::from(new_short) * long / (f64::from(short) * 2.0)).round();
            let new_long = (half_long as u32 * 2).max(2);
            if w <= h {
                (new_short, new_long)
            } else {
                (new_long, new_short)
            }
        }
        _ => (even(w), even(h)),
    }
}

pub fn is_downscale(display: (u32, u32), cap: Option<u32>) -> bool {
    cap.is_some_and(|cap| display.0.min(display.1) > cap)
}

/// Scale (and tonemap) filter chain shared by the encode and the posters, so
/// poster frames match the compressed video exactly. No trailing format.
fn video_filter(target: (u32, u32), hdr: bool) -> String {
    let scale = format!("scale={}:{}:flags=lanczos,setsar=1", target.0, target.1);
    if hdr {
        format!("{scale},{TONEMAP}")
    } else {
        scale
    }
}

fn push(args: &mut Vec<OsString>, items: &[&str]) {
    args.extend(items.iter().map(OsString::from));
}

fn quiet() -> Vec<OsString> {
    let mut args = Vec::new();
    push(&mut args, &["-hide_banner", "-nostdin", "-v", "error", "-y"]);
    args
}

/// Quiet run that reports machine-readable progress on stdout.
fn tracked() -> Vec<OsString> {
    let mut args = quiet();
    push(&mut args, &["-progress", "pipe:1", "-nostats"]);
    args
}

fn push_input(args: &mut Vec<OsString>, input: &Path) {
    args.push("-i".into());
    args.push(input.into());
}

/// First real video stream (capital V skips cover art) and, optionally, the
/// first audio stream if there is one.
fn push_maps(args: &mut Vec<OsString>, keep_audio: bool) {
    push(args, &["-map", "0:V:0"]);
    if keep_audio {
        push(args, &["-map", "0:a:0?"]);
    } else {
        args.push("-an".into());
    }
    push(args, &["-map_metadata", "-1", "-map_chapters", "-1"]);
}

fn push_container(args: &mut Vec<OsString>, format: VideoFormat, output: &Path) {
    match format {
        VideoFormat::Mp4 => push(args, &["-movflags", "+faststart", "-f", "mp4"]),
        VideoFormat::Webm => push(args, &["-cues_to_front", "1", "-f", "webm"]),
    }
    args.push(output.into());
}

fn push_audio(args: &mut Vec<OsString>, codec: &str, kbps: u16, channels: u32) {
    push(args, &["-c:a", codec, "-b:a"]);
    args.push(format!("{kbps}k").into());
    if channels > 2 {
        push(args, &["-ac", "2"]);
    }
}

/// `ffmpeg -i IN` without an output: prints the stream summary for
/// [`super::probe::parse_probe`] and exits with status 1.
pub fn probe_args(input: &Path) -> Vec<OsString> {
    let mut args = Vec::new();
    push(&mut args, &["-hide_banner", "-nostdin"]);
    push_input(&mut args, input);
    args
}

/// One frame as PNG on stdout, at most [`THUMB_PX`] on its long side.
pub fn thumbnail_args(input: &Path, seek: f64, hdr: bool) -> Vec<OsString> {
    let mut args = quiet();
    args.push("-ss".into());
    args.push(format!("{seek:.3}").into());
    push_input(&mut args, input);
    push(&mut args, &["-map", "0:V:0", "-an", "-frames:v", "1", "-vf"]);
    let scale = format!("scale={THUMB_PX}:{THUMB_PX}:force_original_aspect_ratio=decrease");
    let tonemap = if hdr { format!(",{TONEMAP}") } else { String::new() };
    args.push(format!("{scale}{tonemap},format=rgb24").into());
    push(&mut args, &["-c:v", "png", "-f", "image2pipe", "pipe:1"]);
    args
}

/// Where to grab the row thumbnail: a second in (past fade-ins), or 10% into
/// very short clips.
pub fn thumbnail_seek(duration: Option<f64>) -> f64 {
    duration.map_or(0.0, |d| (d * 0.1).min(1.0))
}

pub fn encode_args(input: &Path, output: &Path, plan: &EncodePlan) -> Vec<OsString> {
    let mut args = tracked();
    push_input(&mut args, input);
    push_maps(&mut args, plan.audio_channels.is_some());
    args.push("-vf".into());
    args.push(format!("{},format=yuv420p", video_filter(plan.target, plan.hdr)).into());
    let crf = plan.preset.crf.to_string();
    let channels = plan.audio_channels.unwrap_or(0);
    match plan.format {
        VideoFormat::Mp4 => {
            push(&mut args, &["-c:v", "libx264", "-preset", "medium", "-crf", &crf]);
            push(&mut args, &["-profile:v", "high", "-pix_fmt", "yuv420p"]);
            if plan.audio_channels.is_some() {
                push_audio(&mut args, "aac", plan.preset.audio_kbps, channels);
            }
        }
        VideoFormat::Webm => {
            let cpu_used = plan.preset.cpu_used.to_string();
            push(&mut args, &["-c:v", "libvpx-vp9", "-crf", &crf, "-b:v", "0"]);
            push(&mut args, &["-deadline", "good", "-cpu-used", &cpu_used, "-row-mt", "1"]);
            push(&mut args, &["-tile-columns", "2", "-g", "240", "-pix_fmt", "yuv420p"]);
            if plan.audio_channels.is_some() {
                push_audio(&mut args, "libopus", plan.preset.audio_kbps, channels);
            }
        }
    }
    if plan.hdr {
        push(&mut args, &["-colorspace", "bt709", "-color_primaries", "bt709"]);
        push(&mut args, &["-color_trc", "bt709"]);
    }
    push_container(&mut args, plan.format, output);
    args
}

/// Lossless stream copy of the source into the target container (used when
/// re-encoding saves too little). Metadata is still stripped and the MP4
/// index moved to the front for streaming.
pub fn remux_args(input: &Path, output: &Path, format: VideoFormat, keep_audio: bool) -> Vec<OsString> {
    let mut args = tracked();
    push_input(&mut args, input);
    push_maps(&mut args, keep_audio);
    push(&mut args, &["-c", "copy"]);
    push_container(&mut args, format, output);
    args
}

/// Poster frames spread over the plan's window, written as
/// `<dir>/poster-00.png`, `poster-01.png`, ...
pub fn poster_args(
    input: &Path,
    dir: &Path,
    plan: PosterPlan,
    target: (u32, u32),
    hdr: bool,
) -> Vec<OsString> {
    let mut args = quiet();
    push(&mut args, &["-ss", &format!("{:.3}", plan.start)]);
    push(&mut args, &["-t", &format!("{:.3}", plan.window)]);
    push_input(&mut args, input);
    push(&mut args, &["-map", "0:V:0", "-an", "-vf"]);
    let rate = f64::from(plan.count) / plan.window;
    args.push(format!("fps={rate:.4},{},format=rgb24", video_filter(target, hdr)).into());
    let count = plan.count.to_string();
    push(&mut args, &["-frames:v", &count, "-c:v", "png", "-start_number", "0"]);
    push(&mut args, &["-f", "image2"]);
    // image2 treats '%' as a pattern character, so escape it in the folder.
    let dir = dir.to_string_lossy().replace('%', "%%");
    args.push(format!("{dir}{MAIN_SEPARATOR}{POSTER_PATTERN}").into());
    args
}

/// Eight frames from 0.5 s to ~5.5 s (skipping fade-ins), or spread over the
/// whole clip when it is shorter than 1.5 s.
pub fn poster_plan(duration: Option<f64>, fps: Option<f64>) -> PosterPlan {
    match duration {
        Some(d) if d < SHORT_CLIP => {
            let frames = (d * fps.unwrap_or(FALLBACK_FPS)).round();
            PosterPlan {
                start: 0.0,
                window: d.max(0.04),
                count: (frames as u32).clamp(1, POSTER_COUNT),
            }
        }
        Some(d) => PosterPlan {
            start: POSTER_START,
            window: (d - POSTER_START).min(POSTER_WINDOW),
            count: POSTER_COUNT,
        },
        None => PosterPlan {
            start: POSTER_START,
            window: POSTER_WINDOW,
            count: POSTER_COUNT,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::video::formats::{preset, VideoQuality};

    fn strings(args: &[OsString]) -> Vec<String> {
        args.iter().map(|a| a.to_string_lossy().into_owned()).collect()
    }

    fn has_pair(args: &[String], key: &str, value: &str) -> bool {
        args.windows(2).any(|w| w[0] == key && w[1] == value)
    }

    fn plan(format: VideoFormat, audio: Option<u32>, hdr: bool) -> EncodePlan {
        EncodePlan {
            format,
            preset: preset(format, VideoQuality::Balanced),
            target: (1280, 720),
            hdr,
            audio_channels: audio,
        }
    }

    #[test]
    fn scaled_dims_caps_the_short_side_without_upscaling() {
        assert_eq!(scaled_dims((4096, 2160), Some(720)), (1366, 720));
        assert_eq!(scaled_dims((241, 479), Some(240)), (240, 478));
        assert_eq!(scaled_dims((720, 1280), Some(480)), (480, 854));
        assert_eq!(scaled_dims((1920, 1080), Some(1080)), (1920, 1080));
        assert_eq!(scaled_dims((200, 100), Some(240)), (200, 100));
        assert_eq!(scaled_dims((1921, 1081), None), (1920, 1080));
        assert!(is_downscale((1920, 1080), Some(720)));
        assert!(!is_downscale((1280, 720), Some(720)));
        assert!(!is_downscale((1280, 720), None));
    }

    #[test]
    fn mp4_encode_keeps_audio_and_downmixes_surround() {
        let args = strings(&encode_args(
            Path::new("/in/a.mov"),
            Path::new("/out/a.mp4"),
            &plan(VideoFormat::Mp4, Some(6), false),
        ));
        assert!(has_pair(&args, "-c:v", "libx264"));
        assert!(has_pair(&args, "-crf", "23"));
        assert!(has_pair(&args, "-map", "0:a:0?"));
        assert!(has_pair(&args, "-c:a", "aac"));
        assert!(has_pair(&args, "-b:a", "128k"));
        assert!(has_pair(&args, "-ac", "2"));
        assert!(has_pair(&args, "-movflags", "+faststart"));
        assert!(has_pair(&args, "-vf", "scale=1280:720:flags=lanczos,setsar=1,format=yuv420p"));
        assert!(has_pair(&args, "-progress", "pipe:1"));
        assert_eq!(args.last().map(String::as_str), Some("/out/a.mp4"));
    }

    #[test]
    fn removing_audio_drops_the_stream_and_codec() {
        let args = strings(&encode_args(
            Path::new("in.mp4"),
            Path::new("out.mp4"),
            &plan(VideoFormat::Mp4, None, false),
        ));
        assert!(args.contains(&"-an".to_string()));
        assert!(!args.contains(&"-c:a".to_string()));
        assert!(!has_pair(&args, "-map", "0:a:0?"));
    }

    #[test]
    fn stereo_is_not_remixed() {
        let args = strings(&encode_args(
            Path::new("in.mp4"),
            Path::new("out.mp4"),
            &plan(VideoFormat::Mp4, Some(2), false),
        ));
        assert!(!args.contains(&"-ac".to_string()));
    }

    #[test]
    fn webm_uses_constant_quality_vp9_and_opus() {
        let args = strings(&encode_args(
            Path::new("in.mp4"),
            Path::new("out.webm"),
            &plan(VideoFormat::Webm, Some(1), false),
        ));
        assert!(has_pair(&args, "-c:v", "libvpx-vp9"));
        assert!(has_pair(&args, "-crf", "32"));
        assert!(has_pair(&args, "-b:v", "0"));
        assert!(has_pair(&args, "-c:a", "libopus"));
        assert!(has_pair(&args, "-f", "webm"));
        assert!(!args.contains(&"-movflags".to_string()));
    }

    #[test]
    fn hdr_sources_are_tonemapped_and_tagged_bt709() {
        let args = strings(&encode_args(
            Path::new("in.mov"),
            Path::new("out.mp4"),
            &plan(VideoFormat::Mp4, None, true),
        ));
        let vf = &args[args.iter().position(|a| a == "-vf").unwrap() + 1];
        assert!(vf.contains("tonemap=tonemap=hable"), "{vf}");
        assert!(vf.ends_with(",format=yuv420p"));
        assert!(has_pair(&args, "-color_trc", "bt709"));
    }

    #[test]
    fn remux_copies_streams() {
        let args = strings(&remux_args(
            Path::new("in.mp4"),
            Path::new("out.mp4"),
            VideoFormat::Mp4,
            false,
        ));
        assert!(has_pair(&args, "-c", "copy"));
        assert!(args.contains(&"-an".to_string()));
        assert!(has_pair(&args, "-map_metadata", "-1"));
        assert!(has_pair(&args, "-movflags", "+faststart"));
    }

    #[test]
    fn poster_pattern_escapes_percent_signs() {
        let plan = poster_plan(Some(30.0), Some(30.0));
        let args = strings(&poster_args(
            Path::new("in.mp4"),
            Path::new("/work/100%"),
            plan,
            (1280, 720),
            false,
        ));
        let last = args.last().unwrap();
        assert!(last.starts_with("/work/100%%"), "{last}");
        assert!(last.ends_with("poster-%02d.png"));
        assert!(has_pair(&args, "-frames:v", "8"));
        assert!(has_pair(&args, "-ss", "0.500"));
        assert!(has_pair(&args, "-t", "5.000"));
    }

    #[test]
    fn poster_plans_adapt_to_duration() {
        let long = poster_plan(Some(30.0), None);
        assert_eq!((long.start, long.window, long.count), (0.5, 5.0, 8));
        assert!((long.time_of(4) - 3.0).abs() < 1e-9);
        assert_eq!(poster_plan(Some(3.0), None).window, 2.5);
        assert_eq!(poster_plan(None, None).count, 8);
        let short = poster_plan(Some(1.0), Some(30.0));
        assert_eq!((short.start, short.count), (0.0, 8));
        let tiny = poster_plan(Some(0.1), Some(30.0));
        assert_eq!(tiny.count, 3);
        assert_eq!(poster_plan(Some(0.01), Some(30.0)).count, 1);
    }

    #[test]
    fn thumbnail_seeks_past_fade_ins() {
        assert_eq!(thumbnail_seek(Some(60.0)), 1.0);
        assert!((thumbnail_seek(Some(3.0)) - 0.3).abs() < 1e-9);
        assert_eq!(thumbnail_seek(None), 0.0);
        let args = strings(&thumbnail_args(Path::new("in.mov"), 1.0, true));
        assert!(has_pair(&args, "-ss", "1.000"));
        assert!(args.iter().any(|a| a.contains("tonemap") && a.ends_with("format=rgb24")));
        assert_eq!(args.last().map(String::as_str), Some("pipe:1"));
    }
}
