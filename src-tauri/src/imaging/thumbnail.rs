//! Thumbnails and previews delivered to the webview as base64 data URLs.

use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;
use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView};

use super::pixels::{flatten_on, has_transparency, UI_BACKGROUND};
use crate::error::{AppError, AppResult};

/// Thumbnail pixel size (rendered at half that in CSS px for HiDPI screens).
pub const THUMB_PX: u32 = 256;
/// Longest side of the crop-stage preview.
pub const PREVIEW_MAX_PX: u32 = 2048;
/// Longest side of the post-crop result preview.
pub const RESULT_PREVIEW_PX: u32 = 512;

const THUMB_QUALITY: u8 = 72;
const PREVIEW_JPEG_QUALITY: u8 = 85;
const PREVIEW_WEBP_QUALITY: f32 = 88.0;

/// 256px JPEG thumbnail matted on the UI background.
pub fn thumbnail_data_url(img: &DynamicImage) -> AppResult<String> {
    let thumb = img.thumbnail(THUMB_PX, THUMB_PX);
    jpeg_data_url(&thumb, THUMB_QUALITY)
}

/// Preview capped at `max_px` on the longest side. Keeps transparency (WebP)
/// when the image uses it, otherwise JPEG. Returns the data URL and the
/// preview/source scale factor.
pub fn preview_data_url(img: &DynamicImage, max_px: u32) -> AppResult<(String, f64)> {
    let (w, h) = img.dimensions();
    let longest = w.max(h);
    let (preview, scale) = if longest > max_px {
        (
            img.resize(max_px, max_px, FilterType::Triangle),
            f64::from(max_px) / f64::from(longest),
        )
    } else {
        (img.clone(), 1.0)
    };
    let url = if has_transparency(&preview) {
        webp_data_url(&preview, PREVIEW_WEBP_QUALITY)
    } else {
        jpeg_data_url(&preview, PREVIEW_JPEG_QUALITY)
    }?;
    Ok((url, scale))
}

fn jpeg_data_url(img: &DynamicImage, quality: u8) -> AppResult<String> {
    let rgb = flatten_on(img, UI_BACKGROUND);
    let mut buf = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(&mut buf, quality);
    encoder
        .encode_image(&rgb)
        .map_err(|e| AppError::Encode(e.to_string()))?;
    Ok(data_url("image/jpeg", &buf))
}

fn webp_data_url(img: &DynamicImage, quality: f32) -> AppResult<String> {
    let rgba = img.to_rgba8();
    let mem = webp::Encoder::from_rgba(rgba.as_raw(), rgba.width(), rgba.height())
        .encode(quality);
    Ok(data_url("image/webp", &mem))
}

fn data_url(mime: &str, bytes: &[u8]) -> String {
    format!("data:{mime};base64,{}", STANDARD.encode(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    #[test]
    fn thumbnail_is_a_jpeg_data_url() {
        let img = DynamicImage::ImageRgba8(RgbaImage::from_pixel(600, 300, Rgba([200, 90, 30, 255])));
        let url = thumbnail_data_url(&img).unwrap();
        assert!(url.starts_with("data:image/jpeg;base64,"));
    }

    #[test]
    fn preview_scales_down_and_reports_scale() {
        let img = DynamicImage::ImageRgba8(RgbaImage::from_pixel(4096, 1024, Rgba([10, 20, 30, 255])));
        let (url, scale) = preview_data_url(&img, PREVIEW_MAX_PX).unwrap();
        assert!(url.starts_with("data:image/jpeg;base64,"));
        assert!((scale - 0.5).abs() < 1e-9);
    }

    #[test]
    fn transparent_preview_uses_webp() {
        let img = DynamicImage::ImageRgba8(RgbaImage::from_pixel(64, 64, Rgba([10, 20, 30, 0])));
        let (url, scale) = preview_data_url(&img, PREVIEW_MAX_PX).unwrap();
        assert!(url.starts_with("data:image/webp;base64,"));
        assert_eq!(scale, 1.0);
    }
}
