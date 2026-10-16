//! Reads the stream summary FFmpeg prints for `ffmpeg -i <file>`, so no
//! ffprobe binary has to be shipped. Pure text parsing, tested against output
//! captured from FFmpeg 6.1 and 9.0 (`fixtures/`).

use crate::error::{AppError, AppResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hdr {
    /// SMPTE ST 2084 (HDR10, Dolby Vision base layers).
    Pq,
    /// ARIB STD-B67 (HLG, the iPhone default).
    Hlg,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioInfo {
    pub codec: String,
    pub channels: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProbeInfo {
    /// Seconds; None when the container does not say (e.g. MediaRecorder WebM).
    pub duration: Option<f64>,
    pub video_codec: String,
    pub pix_fmt: String,
    /// Coded size, before SAR and rotation.
    pub width: u32,
    pub height: u32,
    pub sar: (u32, u32),
    /// Clockwise display rotation in degrees: 0, 90, 180 or 270.
    pub rotation: u16,
    pub fps: Option<f64>,
    pub hdr: Option<Hdr>,
    pub audio: Option<AudioInfo>,
}

impl ProbeInfo {
    /// The size the video is displayed at: non-square pixels stretched,
    /// rotation applied. This is what players (and our outputs) show.
    pub fn display_dims(&self) -> (u32, u32) {
        let (num, den) = self.sar;
        let width = if num > 0 && den > 0 && num != den {
            ((f64::from(self.width) * f64::from(num) / f64::from(den)).round() as u32).max(1)
        } else {
            self.width
        };
        if self.rotation % 180 == 90 {
            (self.height, width)
        } else {
            (width, self.height)
        }
    }
}

struct VideoStream {
    codec: String,
    pix_fmt: String,
    width: u32,
    height: u32,
    sar: (u32, u32),
    fps: Option<f64>,
    hdr: Option<Hdr>,
}

/// Parse the stderr of `ffmpeg -hide_banner -i <file>` (which exits with an
/// error because no output is given; the summary is printed regardless).
pub fn parse_probe(stderr: &str) -> AppResult<ProbeInfo> {
    let mut lines = stderr.lines().skip_while(|line| !line.starts_with("Input #0"));
    if lines.next().is_none() {
        return Err(AppError::Unsupported(last_message(stderr)));
    }
    let mut duration = None;
    let mut video: Option<VideoStream> = None;
    let mut audio = None;
    let mut rotation = 0;
    let mut in_chosen_video = false;
    for line in lines {
        // The input summary is indented; the next unindented line ends it.
        if !line.starts_with(' ') {
            break;
        }
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Duration: ") {
            duration = parse_duration(rest);
        } else if trimmed.starts_with("Stream #") {
            in_chosen_video = false;
            if let Some((_, desc)) = trimmed.split_once(": Video: ") {
                // Cover art in MP4/M4A is a video stream flagged as attached pic.
                if video.is_none() && !trimmed.ends_with("(attached pic)") {
                    video = parse_video(desc);
                    in_chosen_video = video.is_some();
                }
            } else if let Some((_, desc)) = trimmed.split_once(": Audio: ") {
                audio = audio.or_else(|| Some(parse_audio(desc)));
            }
        } else if in_chosen_video {
            rotation = parse_rotation(trimmed).unwrap_or(rotation);
        }
    }
    let video = video.ok_or_else(|| AppError::Unsupported("no video stream".to_string()))?;
    Ok(ProbeInfo {
        duration,
        video_codec: video.codec,
        pix_fmt: video.pix_fmt,
        width: video.width,
        height: video.height,
        sar: video.sar,
        rotation,
        fps: video.fps,
        hdr: video.hdr,
        audio,
    })
}

/// The most useful line of an FFmpeg failure: the last one, minus prefixes
/// such as "Error opening input files: ".
fn last_message(stderr: &str) -> String {
    let line = stderr
        .lines()
        .map(str::trim)
        .rfind(|line| !line.is_empty())
        .unwrap_or("not a video file");
    line.rsplit_once(": ").map_or(line, |(_, message)| message).to_string()
}

/// "00:01:02.50, start: 0.000000, bitrate: 1234 kb/s" or "N/A, ...".
fn parse_duration(rest: &str) -> Option<f64> {
    let value = rest.split(',').next()?.trim();
    let mut parts = value.split(':');
    let hours: f64 = parts.next()?.parse().ok()?;
    let minutes: f64 = parts.next()?.parse().ok()?;
    let seconds: f64 = parts.next()?.parse().ok()?;
    let total = hours * 3600.0 + minutes * 60.0 + seconds;
    (total > 0.0).then_some(total)
}

/// Split on commas that are not inside `(...)` or `[...]`, so
/// "yuv420p10le(tv, bt2020nc/bt2020/arib-std-b67)" stays one field.
fn split_fields(desc: &str) -> Vec<&str> {
    let mut fields = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;
    for (i, c) in desc.char_indices() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth -= 1,
            ',' if depth == 0 => {
                fields.push(desc[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
    }
    fields.push(desc[start..].trim());
    fields
}

fn first_word(field: &str) -> String {
    field.split_whitespace().next().unwrap_or_default().to_string()
}

fn parse_video(desc: &str) -> Option<VideoStream> {
    let fields = split_fields(desc);
    let color = fields.get(1).copied().unwrap_or_default();
    let (width, height) = fields.iter().skip(1).find_map(|field| parse_size(field))?;
    let hdr = if color.contains("smpte2084") {
        Some(Hdr::Pq)
    } else if color.contains("arib-std-b67") {
        Some(Hdr::Hlg)
    } else {
        None
    };
    Some(VideoStream {
        codec: first_word(fields[0]),
        pix_fmt: color.split('(').next().unwrap_or_default().trim().to_string(),
        width,
        height,
        sar: parse_sar(desc).unwrap_or((1, 1)),
        fps: fields
            .iter()
            .find_map(|field| field.strip_suffix(" fps"))
            .and_then(|fps| fps.trim().parse().ok()),
        hdr,
    })
}

/// "1280x720 [SAR 1:1 DAR 16:9]" or "640x360" → (w, h).
fn parse_size(field: &str) -> Option<(u32, u32)> {
    let token = field.split_whitespace().next()?;
    let (w, h) = token.split_once('x')?;
    Some((w.parse().ok()?, h.parse().ok()?))
}

/// Finds "SAR 8:9" whether printed in brackets (MP4) or as its own field (WebM).
fn parse_sar(desc: &str) -> Option<(u32, u32)> {
    let (_, rest) = desc.split_once("SAR ")?;
    let ratio = rest.split([' ', ']', ',']).next()?;
    let (num, den) = ratio.split_once(':')?;
    Some((num.parse().ok()?, den.parse().ok()?))
}

fn parse_audio(desc: &str) -> AudioInfo {
    let fields = split_fields(desc);
    AudioInfo {
        codec: first_word(fields[0]),
        channels: fields
            .iter()
            .skip(1)
            .find_map(|field| channel_count(field))
            .unwrap_or(2),
    }
}

fn channel_count(field: &str) -> Option<u32> {
    let layout = field.split('(').next().unwrap_or(field).trim();
    match layout {
        "mono" => Some(1),
        "stereo" => Some(2),
        "2.1" | "3.0" => Some(3),
        "quad" | "4.0" => Some(4),
        "4.1" | "5.0" => Some(5),
        "5.1" | "6.0" | "hexagonal" => Some(6),
        "6.1" | "7.0" => Some(7),
        "7.1" | "octagonal" => Some(8),
        _ => layout.strip_suffix(" channels").and_then(|n| n.parse().ok()),
    }
}

/// "displaymatrix: rotation of -90.00 degrees" (FFmpeg 6) or
/// "Display Matrix: rotation of 90.00 degrees" (FFmpeg 7+). FFmpeg reports
/// the counter-clockwise angle; returns the clockwise one snapped to 90°.
fn parse_rotation(line: &str) -> Option<u16> {
    let (_, rest) = line.split_once("rotation of ")?;
    let degrees: f64 = rest.split_whitespace().next()?.parse().ok()?;
    let clockwise = (-degrees).rem_euclid(360.0);
    let snapped = ((clockwise / 90.0).round() as u16 * 90) % 360;
    Some(snapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(fixture: &str) -> ProbeInfo {
        parse_probe(fixture).expect("fixture should parse")
    }

    #[test]
    fn h264_aac_mp4_on_both_versions() {
        for fixture in [
            include_str!("fixtures/ffmpeg9_h264_aac_mp4.txt"),
            include_str!("fixtures/ffmpeg6_h264_aac_mp4.txt"),
        ] {
            let info = parse(fixture);
            assert_eq!(info.duration, Some(3.0));
            assert_eq!(info.video_codec, "h264");
            assert_eq!(info.pix_fmt, "yuv420p");
            assert_eq!((info.width, info.height), (1280, 720));
            assert_eq!(info.sar, (1, 1));
            assert_eq!(info.fps, Some(30.0));
            assert_eq!(info.rotation, 0);
            assert_eq!(info.hdr, None);
            assert_eq!(info.display_dims(), (1280, 720));
            let audio = info.audio.expect("audio stream");
            assert_eq!((audio.codec.as_str(), audio.channels), ("aac", 1));
        }
    }

    #[test]
    fn rotated_hlg_hevc_on_both_versions() {
        for fixture in [
            include_str!("fixtures/ffmpeg9_hevc_hlg_rot_mov.txt"),
            include_str!("fixtures/ffmpeg6_hevc_hlg_rot_mov.txt"),
        ] {
            let info = parse(fixture);
            assert_eq!(info.video_codec, "hevc");
            assert_eq!(info.pix_fmt, "yuv420p10le");
            assert_eq!(info.hdr, Some(Hdr::Hlg));
            assert_eq!(info.rotation, 270);
            assert_eq!(info.display_dims(), (1080, 1920));
            assert!(info.audio.is_none());
        }
    }

    #[test]
    fn webm_without_duration() {
        let info = parse(include_str!("fixtures/ffmpeg9_recorder_webm.txt"));
        assert_eq!(info.duration, None);
        assert_eq!(info.video_codec, "vp9");
        assert_eq!((info.width, info.height), (640, 360));
        assert_eq!(info.sar, (1, 1));
        assert_eq!(info.audio.expect("audio").codec, "opus");
    }

    #[test]
    fn mkv_with_cover_stream_and_surround_audio() {
        let info = parse(include_str!("fixtures/ffmpeg9_cover_51_mkv.txt"));
        assert_eq!(info.video_codec, "h264");
        assert_eq!((info.width, info.height), (1280, 720));
        assert_eq!(info.fps, Some(25.0));
        assert_eq!(info.audio.expect("audio").channels, 6);
    }

    #[test]
    fn attached_pictures_are_skipped() {
        let info = parse(include_str!("fixtures/ffmpeg9_cover_first_mp4.txt"));
        assert_eq!(info.video_codec, "h264");
        let reordered = "Input #0, mov,mp4,m4a,3gp,3g2,mj2, from 'x.mp4':\n  Duration: 00:00:02.00, start: 0.000000, bitrate: 1 kb/s\n  Stream #0:0[0x0]: Video: mjpeg (Baseline), yuvj444p(pc, bt470bg/unknown/unknown), 320x240 [SAR 1:1 DAR 4:3], 90k tbr, 90k tbn (attached pic)\n  Stream #0:1[0x1](und): Video: h264 (High) (avc1 / 0x31637661), yuv420p(progressive), 1280x720 [SAR 1:1 DAR 16:9], 2701 kb/s, 25 fps, 25 tbr, 12800 tbn (default)\nAt least one output file must be specified\n";
        let info = parse(reordered);
        assert_eq!((info.video_codec.as_str(), info.width), ("h264", 1280));
    }

    #[test]
    fn non_square_pixels_widen_the_display_size() {
        let info = parse(include_str!("fixtures/ffmpeg9_sar_mp4.txt"));
        assert_eq!(info.sar, (8, 9));
        assert_eq!(info.display_dims(), (640, 480));
    }

    #[test]
    fn audio_only_and_garbage_are_rejected() {
        let err = parse_probe(include_str!("fixtures/ffmpeg9_audio_only_m4a.txt")).unwrap_err();
        assert!(err.to_string().contains("no video stream"));
        let err = parse_probe(include_str!("fixtures/ffmpeg9_notes_txt.txt")).unwrap_err();
        assert!(err.to_string().contains("Invalid data found"), "{err}");
    }

    #[test]
    fn pq_and_channel_layouts() {
        let desc = "hevc (Main 10), yuv420p10le(tv, bt2020nc/bt2020/smpte2084), 3840x2160, 24 fps";
        let video = parse_video(desc).expect("video");
        assert_eq!(video.hdr, Some(Hdr::Pq));
        assert_eq!((video.width, video.height), (3840, 2160));
        assert_eq!(parse_audio("ac3, 48000 Hz, 5.1(side), fltp, 384 kb/s").channels, 6);
        assert_eq!(parse_audio("pcm_s16le, 48000 Hz, 3 channels, s16").channels, 3);
        assert_eq!(parse_rotation("displaymatrix: rotation of -90.00 degrees"), Some(90));
        assert_eq!(parse_rotation("Display Matrix: rotation of 180.00 degrees"), Some(180));
    }
}
