//! Poster candidates: frames from the first seconds of the source, rendered
//! at the compressed video's size and offered as `<video poster>` images.

use std::path::Path;

use image::DynamicImage;

use super::args::{PosterPlan, POSTER_PATTERN};
use crate::error::AppResult;
use crate::imaging::thumbnail;
use crate::state::PosterFrame;

/// Long side of the previews shown in the poster sidebar.
pub const PREVIEW_PX: u32 = 480;
/// Mean luma (0-255) below which a frame counts as black (fade-ins).
const DARK_LUMA: f64 = 16.0;
const DARK_SAMPLE_PX: u32 = 64;

/// The frames FFmpeg wrote into `dir` for `plan`, in order.
pub fn collect(dir: &Path, plan: PosterPlan) -> AppResult<Vec<PosterFrame>> {
    let mut frames = Vec::new();
    for index in 0..plan.count {
        let path = dir.join(POSTER_PATTERN.replace("%02d", &format!("{index:02}")));
        if !path.is_file() {
            break; // the clip ended before the window did
        }
        let img = image::open(&path)?;
        frames.push(PosterFrame {
            dark: is_dark(&img),
            width: img.width(),
            height: img.height(),
            time_sec: plan.time_of(index),
            path,
        });
    }
    Ok(frames)
}

pub fn is_dark(img: &DynamicImage) -> bool {
    let sample = img.thumbnail(DARK_SAMPLE_PX, DARK_SAMPLE_PX).to_luma8();
    let count = sample.pixels().len().max(1) as f64;
    let mean = sample.pixels().map(|p| f64::from(p[0])).sum::<f64>() / count;
    mean < DARK_LUMA
}

pub fn preview_data_url(frame: &PosterFrame) -> AppResult<String> {
    let img = image::open(&frame.path)?;
    Ok(thumbnail::preview_data_url(&img, PREVIEW_PX)?.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgb, RgbImage};

    #[test]
    fn black_frames_are_flagged() {
        let black = DynamicImage::ImageRgb8(RgbImage::from_pixel(320, 180, Rgb([4, 4, 4])));
        let gray = DynamicImage::ImageRgb8(RgbImage::from_pixel(320, 180, Rgb([90, 90, 90])));
        assert!(is_dark(&black));
        assert!(!is_dark(&gray));
    }

    #[test]
    fn collect_stops_at_the_first_missing_frame() {
        let dir = std::env::temp_dir().join(format!("sharpr-posters-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        for index in 0..3 {
            let img = RgbImage::from_pixel(64, 36, Rgb([200, 100, 50]));
            img.save(dir.join(format!("poster-{index:02}.png"))).unwrap();
        }
        let plan = PosterPlan { start: 0.5, window: 5.0, count: 8 };
        let frames = collect(&dir, plan).unwrap();
        assert_eq!(frames.len(), 3);
        assert_eq!((frames[2].width, frames[2].height), (64, 36));
        assert!((frames[2].time_sec - 1.75).abs() < 1e-9);
        assert!(!frames[0].dark);
        assert!(preview_data_url(&frames[0]).unwrap().starts_with("data:image/jpeg;base64,"));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
