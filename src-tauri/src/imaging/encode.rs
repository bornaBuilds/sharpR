//! Per-format encoders. Every output is re-encoded from raw pixels, so no
//! metadata (EXIF/XMP/ICC) ever carries over.

use image::codecs::png::{CompressionType, FilterType as PngFilter, PngEncoder};
use image::{DynamicImage, ExtendedColorType, GenericImageView, ImageEncoder};
use rgb::FromSlice;
use serde::{Deserialize, Serialize};

use super::pixels::{flatten_on, has_transparency, WHITE};
use crate::error::{AppError, AppResult};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Webp,
    Avif,
    Jpeg,
    Png,
}

impl OutputFormat {
    pub fn ext(self) -> &'static str {
        match self {
            OutputFormat::Webp => "webp",
            OutputFormat::Avif => "avif",
            OutputFormat::Jpeg => "jpg",
            OutputFormat::Png => "png",
        }
    }
}

#[derive(Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EncodeOptions {
    pub format: OutputFormat,
    /// 1..=100 for lossy formats; ignored for PNG and lossless WebP.
    pub quality: u8,
    /// WebP only.
    pub lossless: bool,
}

pub struct Encoded {
    pub bytes: Vec<u8>,
    pub ext: &'static str,
    pub width: u32,
    pub height: u32,
}

const MIN_QUALITY: u8 = 1;
const MAX_QUALITY: u8 = 100;
/// rav1e speed preset: 1 = slowest/best, 10 = fastest. 6 is the usual still-image sweet spot.
const AVIF_SPEED: u8 = 6;
/// Above this quality JPEG keeps full chroma resolution (4:4:4) instead of 4:2:0.
const JPEG_FULL_CHROMA_QUALITY: u8 = 90;
const JPEG_MAX_DIMENSION: u32 = u16::MAX as u32;
/// oxipng preset (0-6). 2 is fast and already close to optimal for photos.
const PNG_OPTIMIZATION_LEVEL: u8 = 2;

/// Encode `img` with the given options. `threads` bounds the AVIF encoder's
/// internal parallelism (other formats are single-threaded per image).
pub fn encode(img: &DynamicImage, opts: &EncodeOptions, threads: usize) -> AppResult<Encoded> {
    let quality = opts.quality.clamp(MIN_QUALITY, MAX_QUALITY);
    let (width, height) = img.dimensions();
    let alpha = has_transparency(img);
    let bytes = match opts.format {
        OutputFormat::Webp => encode_webp(img, quality, opts.lossless, alpha),
        OutputFormat::Avif => encode_avif(img, quality, alpha, threads)?,
        OutputFormat::Jpeg => encode_jpeg(img, quality)?,
        OutputFormat::Png => encode_png(img, alpha)?,
    };
    Ok(Encoded {
        bytes,
        ext: opts.format.ext(),
        width,
        height,
    })
}

fn encode_webp(img: &DynamicImage, quality: u8, lossless: bool, alpha: bool) -> Vec<u8> {
    if alpha {
        let rgba = img.to_rgba8();
        let encoder = webp::Encoder::from_rgba(rgba.as_raw(), rgba.width(), rgba.height());
        webp_bytes(&encoder, quality, lossless)
    } else {
        let rgb = img.to_rgb8();
        let encoder = webp::Encoder::from_rgb(rgb.as_raw(), rgb.width(), rgb.height());
        webp_bytes(&encoder, quality, lossless)
    }
}

fn webp_bytes(encoder: &webp::Encoder<'_>, quality: u8, lossless: bool) -> Vec<u8> {
    if lossless {
        encoder.encode_lossless().to_vec()
    } else {
        encoder.encode(f32::from(quality)).to_vec()
    }
}

fn encode_avif(img: &DynamicImage, quality: u8, alpha: bool, threads: usize) -> AppResult<Vec<u8>> {
    let encoder = ravif::Encoder::new()
        .with_quality(f32::from(quality))
        .with_alpha_quality(f32::from(quality))
        .with_speed(AVIF_SPEED)
        .with_num_threads(Some(threads.max(1)));
    let (w, h) = (img.width() as usize, img.height() as usize);
    let encoded = if alpha {
        let rgba = img.to_rgba8();
        encoder.encode_rgba(imgref::Img::new(rgba.as_raw().as_rgba(), w, h))?
    } else {
        let rgb = img.to_rgb8();
        encoder.encode_rgb(imgref::Img::new(rgb.as_raw().as_rgb(), w, h))?
    };
    Ok(encoded.avif_file)
}

fn encode_jpeg(img: &DynamicImage, quality: u8) -> AppResult<Vec<u8>> {
    let (w, h) = img.dimensions();
    if w > JPEG_MAX_DIMENSION || h > JPEG_MAX_DIMENSION {
        return Err(AppError::Invalid(format!(
            "JPEG supports at most {JPEG_MAX_DIMENSION} px per side"
        )));
    }
    // JPEG has no alpha: matte on white, which is what web pages expect.
    let rgb = flatten_on(img, WHITE);
    let mut buf = Vec::new();
    let mut encoder = jpeg_encoder::Encoder::new(&mut buf, quality);
    encoder.set_progressive(true);
    encoder.set_sampling_factor(if quality >= JPEG_FULL_CHROMA_QUALITY {
        jpeg_encoder::SamplingFactor::F_1_1
    } else {
        jpeg_encoder::SamplingFactor::F_2_2
    });
    encoder.encode(rgb.as_raw(), w as u16, h as u16, jpeg_encoder::ColorType::Rgb)?;
    Ok(buf)
}

fn encode_png(img: &DynamicImage, alpha: bool) -> AppResult<Vec<u8>> {
    let (w, h) = img.dimensions();
    let mut buf = Vec::new();
    // Fast first pass; oxipng recompresses and picks filters/colour type below.
    let encoder = PngEncoder::new_with_quality(&mut buf, CompressionType::Fast, PngFilter::NoFilter);
    let written = if alpha {
        let rgba = img.to_rgba8();
        encoder.write_image(rgba.as_raw(), w, h, ExtendedColorType::Rgba8)
    } else {
        let rgb = img.to_rgb8();
        encoder.write_image(rgb.as_raw(), w, h, ExtendedColorType::Rgb8)
    };
    written.map_err(|e| AppError::Encode(e.to_string()))?;

    let mut options = oxipng::Options::from_preset(PNG_OPTIMIZATION_LEVEL);
    options.strip = oxipng::StripChunks::All;
    Ok(oxipng::optimize_from_memory(&buf, &options)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    fn sample(alpha: bool) -> DynamicImage {
        let mut img = RgbaImage::from_fn(48, 32, |x, y| {
            Rgba([(x * 5) as u8, (y * 7) as u8, 120, 255])
        });
        if alpha {
            img.put_pixel(0, 0, Rgba([0, 0, 0, 0]));
        }
        DynamicImage::ImageRgba8(img)
    }

    fn opts(format: OutputFormat, lossless: bool) -> EncodeOptions {
        EncodeOptions { format, quality: 80, lossless }
    }

    #[test]
    fn webp_lossy_and_lossless_have_riff_header() {
        for lossless in [false, true] {
            let out = encode(&sample(true), &opts(OutputFormat::Webp, lossless), 1).unwrap();
            assert_eq!(&out.bytes[0..4], b"RIFF");
            assert_eq!(&out.bytes[8..12], b"WEBP");
            assert_eq!(out.ext, "webp");
        }
    }

    #[test]
    fn avif_has_ftyp_box() {
        let out = encode(&sample(false), &opts(OutputFormat::Avif, false), 1).unwrap();
        assert_eq!(&out.bytes[4..12], b"ftypavif");
        assert_eq!(out.ext, "avif");
    }

    #[test]
    fn jpeg_has_soi_marker_and_flattens_alpha() {
        let out = encode(&sample(true), &opts(OutputFormat::Jpeg, false), 1).unwrap();
        assert_eq!(&out.bytes[0..2], &[0xFF, 0xD8]);
        assert_eq!(out.ext, "jpg");
        let decoded = image::load_from_memory(&out.bytes).unwrap();
        assert_eq!(decoded.dimensions(), (48, 32));
    }

    #[test]
    fn png_has_signature_and_round_trips() {
        let out = encode(&sample(true), &opts(OutputFormat::Png, false), 1).unwrap();
        assert_eq!(&out.bytes[0..8], &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);
        let decoded = image::load_from_memory(&out.bytes).unwrap();
        assert_eq!(decoded.dimensions(), (48, 32));
        assert!(has_transparency(&decoded));
    }
}
