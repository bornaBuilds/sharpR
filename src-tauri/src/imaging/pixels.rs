//! Small pixel-level helpers shared by encoders and previews.

use image::{DynamicImage, RgbImage, RgbaImage};

/// The app's dark background, used to matte transparent thumbnails.
pub const UI_BACKGROUND: [u8; 3] = [0x1d, 0x1c, 0x2c];
/// Matte colour for formats without alpha (JPEG).
pub const WHITE: [u8; 3] = [255, 255, 255];

const OPAQUE: u8 = 255;

/// True when the image has an alpha channel with at least one non-opaque pixel.
pub fn has_transparency(img: &DynamicImage) -> bool {
    if !img.color().has_alpha() {
        return false;
    }
    match img {
        DynamicImage::ImageRgba8(buf) => buf.pixels().any(|p| p[3] != OPAQUE),
        DynamicImage::ImageLumaA8(buf) => buf.pixels().any(|p| p[1] != OPAQUE),
        other => other.to_rgba8().pixels().any(|p| p[3] != OPAQUE),
    }
}

/// Composite the image over an opaque background colour, returning RGB.
pub fn flatten_on(img: &DynamicImage, bg: [u8; 3]) -> RgbImage {
    if !img.color().has_alpha() {
        return img.to_rgb8();
    }
    flatten_rgba_on(&img.to_rgba8(), bg)
}

/// Composite straight-alpha RGBA pixels over an opaque background colour.
pub fn flatten_rgba_on(rgba: &RgbaImage, bg: [u8; 3]) -> RgbImage {
    let (w, h) = rgba.dimensions();
    let mut out = RgbImage::new(w, h);
    for (dst, src) in out.pixels_mut().zip(rgba.pixels()) {
        let alpha = u32::from(src[3]);
        let inv = u32::from(OPAQUE) - alpha;
        for c in 0..3 {
            dst[c] = ((u32::from(src[c]) * alpha + u32::from(bg[c]) * inv)
                / u32::from(OPAQUE)) as u8;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn flatten_blends_half_transparent_pixel_on_white() {
        let mut rgba = RgbaImage::new(1, 1);
        rgba.put_pixel(0, 0, Rgba([0, 0, 0, 128]));
        let rgb = flatten_rgba_on(&rgba, WHITE);
        let p = rgb.get_pixel(0, 0);
        assert!(p[0] >= 126 && p[0] <= 128, "got {}", p[0]);
    }

    #[test]
    fn detects_transparency_only_when_alpha_is_used() {
        let opaque = DynamicImage::ImageRgba8(RgbaImage::from_pixel(2, 2, Rgba([1, 2, 3, 255])));
        let clear = DynamicImage::ImageRgba8(RgbaImage::from_pixel(2, 2, Rgba([1, 2, 3, 0])));
        assert!(!has_transparency(&opaque));
        assert!(has_transparency(&clear));
    }
}
