//! Supported video inputs, output formats and quality presets.

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::probe::ProbeInfo;

/// Container extensions picked up from pickers, drops and folder walks.
/// `.ts` is left out on purpose: recursive folder walks would try to probe
/// every TypeScript file.
pub const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "m4v", "mov", "mkv", "webm", "avi", "wmv", "flv", "mpg", "mpeg", "3gp", "ogv", "mts",
    "m2ts",
];

pub fn is_supported(path: &Path) -> bool {
    extension(path).is_some_and(|ext| VIDEO_EXTENSIONS.contains(&ext.as_str()))
}

fn extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase)
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum VideoFormat {
    Mp4,
    Webm,
}

impl VideoFormat {
    pub fn ext(self) -> &'static str {
        match self {
            VideoFormat::Mp4 => "mp4",
            VideoFormat::Webm => "webm",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum VideoQuality {
    High,
    Balanced,
    Small,
    Smallest,
}

/// Encoder settings behind a quality preset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Preset {
    pub crf: u8,
    pub audio_kbps: u16,
    /// libvpx `-cpu-used` (ignored for MP4).
    pub cpu_used: u8,
}

/// x264 CRF + AAC for MP4, VP9 CRF + Opus for WebM.
pub fn preset(format: VideoFormat, quality: VideoQuality) -> Preset {
    use VideoQuality::*;
    let (crf, audio_kbps) = match (format, quality) {
        (VideoFormat::Mp4, High) => (20, 160),
        (VideoFormat::Mp4, Balanced) => (23, 128),
        (VideoFormat::Mp4, Small) => (26, 112),
        (VideoFormat::Mp4, Smallest) => (29, 96),
        (VideoFormat::Webm, High) => (28, 128),
        (VideoFormat::Webm, Balanced) => (32, 96),
        (VideoFormat::Webm, Small) => (36, 80),
        (VideoFormat::Webm, Smallest) => (40, 64),
    };
    let cpu_used = if matches!(quality, Small | Smallest) { 3 } else { 2 };
    Preset {
        crf,
        audio_kbps,
        cpu_used,
    }
}

/// Whether the source may be kept (stream-copied) instead of the re-encode
/// when re-encoding saves too little: same container family, a codec every
/// browser plays for that container, 8-bit 4:2:0 SDR, and compatible audio.
pub fn fallback_allowed(
    src: &Path,
    probe: &ProbeInfo,
    format: VideoFormat,
    remove_audio: bool,
) -> bool {
    if probe.hdr.is_some() || !matches!(probe.pix_fmt.as_str(), "yuv420p" | "yuvj420p") {
        return false;
    }
    let ext = extension(src).unwrap_or_default();
    let audio_ok = |allowed: &[&str]| {
        remove_audio
            || probe
                .audio
                .as_ref()
                .is_none_or(|audio| allowed.contains(&audio.codec.as_str()))
    };
    let codec = probe.video_codec.as_str();
    match format {
        VideoFormat::Mp4 => {
            matches!(ext.as_str(), "mp4" | "m4v") && codec == "h264" && audio_ok(&["aac", "mp3"])
        }
        VideoFormat::Webm => {
            ext == "webm" && matches!(codec, "vp8" | "vp9" | "av1") && audio_ok(&["opus", "vorbis"])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::video::probe::{AudioInfo, Hdr};

    fn probe(codec: &str, audio: Option<&str>) -> ProbeInfo {
        ProbeInfo {
            duration: Some(3.0),
            video_codec: codec.to_string(),
            pix_fmt: "yuv420p".to_string(),
            width: 1280,
            height: 720,
            sar: (1, 1),
            rotation: 0,
            fps: Some(30.0),
            hdr: None,
            audio: audio.map(|codec| AudioInfo {
                codec: codec.to_string(),
                channels: 2,
            }),
        }
    }

    #[test]
    fn extensions_are_case_insensitive() {
        assert!(is_supported(Path::new("clip.MOV")));
        assert!(is_supported(Path::new("/a/b/c.m2ts")));
        assert!(!is_supported(Path::new("app.ts")));
        assert!(!is_supported(Path::new("photo.jpg")));
        assert!(!is_supported(Path::new("noext")));
    }

    #[test]
    fn presets_map_to_codec_ranges() {
        assert_eq!(preset(VideoFormat::Mp4, VideoQuality::Balanced).crf, 23);
        assert_eq!(preset(VideoFormat::Webm, VideoQuality::Balanced).crf, 32);
        assert_eq!(preset(VideoFormat::Webm, VideoQuality::Smallest).cpu_used, 3);
        assert_eq!(preset(VideoFormat::Mp4, VideoQuality::High).audio_kbps, 160);
    }

    #[test]
    fn fallback_requires_matching_family() {
        let mp4 = Path::new("in.mp4");
        assert!(fallback_allowed(mp4, &probe("h264", Some("aac")), VideoFormat::Mp4, false));
        assert!(fallback_allowed(mp4, &probe("h264", None), VideoFormat::Mp4, false));
        // HEVC plays poorly in browsers: always re-encode.
        assert!(!fallback_allowed(mp4, &probe("hevc", Some("aac")), VideoFormat::Mp4, false));
        // Opus in MP4 is unusual; removing audio makes it irrelevant.
        assert!(!fallback_allowed(mp4, &probe("h264", Some("opus")), VideoFormat::Mp4, false));
        assert!(fallback_allowed(mp4, &probe("h264", Some("opus")), VideoFormat::Mp4, true));
        // Container must match the target.
        assert!(!fallback_allowed(Path::new("in.mov"), &probe("h264", None), VideoFormat::Mp4, false));
        assert!(!fallback_allowed(mp4, &probe("h264", None), VideoFormat::Webm, false));
        let webm = Path::new("in.webm");
        assert!(fallback_allowed(webm, &probe("vp9", Some("opus")), VideoFormat::Webm, false));
        assert!(!fallback_allowed(webm, &probe("vp9", Some("aac")), VideoFormat::Webm, false));
    }

    #[test]
    fn fallback_rejects_hdr_and_high_bit_depth() {
        let mut hdr = probe("h264", None);
        hdr.hdr = Some(Hdr::Hlg);
        assert!(!fallback_allowed(Path::new("in.mp4"), &hdr, VideoFormat::Mp4, false));
        let mut ten_bit = probe("h264", None);
        ten_bit.pix_fmt = "yuv420p10le".to_string();
        assert!(!fallback_allowed(Path::new("in.mp4"), &ten_bit, VideoFormat::Mp4, false));
    }
}
