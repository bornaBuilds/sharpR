//! Unique, filesystem-safe output names.

use std::collections::HashSet;
use std::path::Path;

const FALLBACK_STEM: &str = "image";
const INVALID_CHARS: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

/// Hands out `stem.ext` names, appending `-1`, `-2`, ... when a name was
/// already used (comparison is case-insensitive, like Windows and macOS).
#[derive(Default)]
pub struct OutputNamer {
    used: HashSet<String>,
}

impl OutputNamer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Start with every file name already present in `dir` marked as used.
    pub fn with_existing_dir(dir: &Path) -> Self {
        let mut namer = Self::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                namer
                    .used
                    .insert(entry.file_name().to_string_lossy().to_lowercase());
            }
        }
        namer
    }

    pub fn unique(&mut self, source_name: &str, ext: &str) -> String {
        let stem = sanitize(stem_of(source_name));
        let mut candidate = format!("{stem}.{ext}");
        let mut n = 1;
        while !self.used.insert(candidate.to_lowercase()) {
            candidate = format!("{stem}-{n}.{ext}");
            n += 1;
        }
        candidate
    }
}

/// `<stem>-crop.<ext>` for results coming from the crop tab.
pub fn crop_name(source_name: &str, ext: &str) -> String {
    format!("{}-crop.{ext}", sanitize(stem_of(source_name)))
}

pub fn stem_of(name: &str) -> &str {
    match name.rsplit_once('.') {
        Some((stem, _)) if !stem.is_empty() => stem,
        _ => name,
    }
}

fn sanitize(stem: &str) -> String {
    let cleaned: String = stem
        .chars()
        .map(|c| {
            if c.is_control() || INVALID_CHARS.contains(&c) {
                '_'
            } else {
                c
            }
        })
        .collect();
    let trimmed = cleaned
        .trim()
        .trim_start_matches('.')
        .trim_end_matches('.')
        .to_string();
    if trimmed.is_empty() {
        FALLBACK_STEM.to_string()
    } else {
        trimmed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appends_counters_on_collisions() {
        let mut namer = OutputNamer::new();
        assert_eq!(namer.unique("photo.jpg", "webp"), "photo.webp");
        assert_eq!(namer.unique("photo.png", "webp"), "photo-1.webp");
        assert_eq!(namer.unique("PHOTO.tiff", "webp"), "PHOTO-2.webp");
        assert_eq!(namer.unique("other.jpg", "webp"), "other.webp");
    }

    #[test]
    fn different_extensions_do_not_collide() {
        let mut namer = OutputNamer::new();
        assert_eq!(namer.unique("a.jpg", "webp"), "a.webp");
        assert_eq!(namer.unique("a.jpg", "png"), "a.png");
    }

    #[test]
    fn sanitizes_invalid_characters_and_empty_stems() {
        let mut namer = OutputNamer::new();
        assert_eq!(namer.unique("we:ird/name?.jpg", "webp"), "we_ird_name_.webp");
        assert_eq!(namer.unique(".hidden", "webp"), "hidden.webp");
        assert_eq!(namer.unique("", "webp"), "image.webp");
    }

    #[test]
    fn crop_name_uses_stem() {
        assert_eq!(crop_name("hero.banner.jpg", "webp"), "hero.banner-crop.webp");
        assert_eq!(crop_name("noext", "png"), "noext-crop.png");
    }

    #[test]
    fn existing_files_in_dir_are_reserved() {
        let dir = std::env::temp_dir().join(format!("sharpr-naming-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("Photo.webp"), b"x").unwrap();
        let mut namer = OutputNamer::with_existing_dir(&dir);
        assert_eq!(namer.unique("photo.jpg", "webp"), "photo-1.webp");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
