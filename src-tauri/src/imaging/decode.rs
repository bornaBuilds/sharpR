//! Decoding with EXIF orientation applied.

use std::io::{BufRead, Cursor, Seek};
use std::path::Path;

use image::metadata::Orientation;
use image::{DynamicImage, ImageDecoder, ImageReader};

use crate::error::AppResult;
use crate::state::SourceKind;

/// Input extensions accepted by the converter and the cropper (lower-case).
pub const SUPPORTED_EXTENSIONS: &[&str] =
    &["jpg", "jpeg", "png", "webp", "gif", "bmp", "tif", "tiff"];

pub fn is_supported(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| {
            let lower = e.to_ascii_lowercase();
            SUPPORTED_EXTENSIONS.contains(&lower.as_str())
        })
        .unwrap_or(false)
}

/// Decode an image and apply its EXIF orientation so pixels are upright.
/// Animated formats (GIF, WebP) yield their first frame.
pub fn load_oriented(source: &SourceKind) -> AppResult<DynamicImage> {
    match source {
        SourceKind::Path(path) => {
            let reader = ImageReader::open(path)?.with_guessed_format()?;
            decode_reader(reader)
        }
        SourceKind::Memory(bytes) => {
            let reader =
                ImageReader::new(Cursor::new(bytes.as_slice())).with_guessed_format()?;
            decode_reader(reader)
        }
    }
}

fn decode_reader<R: BufRead + Seek>(reader: ImageReader<R>) -> AppResult<DynamicImage> {
    let mut decoder = reader.into_decoder()?;
    let orientation = decoder
        .orientation()
        .unwrap_or(Orientation::NoTransforms);
    let mut img = DynamicImage::from_decoder(decoder)?;
    img.apply_orientation(orientation);
    Ok(img)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_extensions_are_case_insensitive() {
        assert!(is_supported(Path::new("a.JPG")));
        assert!(is_supported(Path::new("b.webp")));
        assert!(is_supported(Path::new("c.TIFF")));
        assert!(!is_supported(Path::new("d.avif")));
        assert!(!is_supported(Path::new("e.txt")));
        assert!(!is_supported(Path::new("noext")));
    }
}
