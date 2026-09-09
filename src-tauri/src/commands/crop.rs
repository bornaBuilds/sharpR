//! Crop tab: open a source, render a crop on the full-resolution pixels,
//! save it, or hand it to the converter list.

use std::path::PathBuf;
use std::sync::Arc;

use image::codecs::png::{CompressionType, FilterType as PngFilter, PngEncoder};
use image::{DynamicImage, ExtendedColorType, ImageEncoder};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use super::files::{ConvertedInfo, ImageItem};
use crate::error::{AppError, AppResult};
use crate::imaging::encode::{encode, EncodeOptions, OutputFormat};
use crate::imaging::transform::{
    apply_transform, canvas_size, crop_and_resize, TransformParams, MAX_SKEW_DEG,
};
use crate::imaging::{decode, thumbnail};
use crate::naming;
use crate::state::{
    lock, AppState, ConvertedImage, CropRendered, CropSourceState, SourceKind, StoredItem,
};

#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum CropOpen {
    Path { path: String },
    Item { id: String },
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CropSource {
    pub id: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub has_alpha: bool,
    pub preview_data_url: String,
    pub preview_scale: f64,
}

#[derive(Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CropRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CropParams {
    pub source_id: String,
    #[serde(flatten)]
    pub transform: TransformParams,
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub crop: CropRect,
    pub output_width: u32,
    pub output_height: u32,
    pub format: OutputFormat,
    pub quality: u8,
    pub lossless: bool,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CropResult {
    pub result_id: String,
    pub width: u32,
    pub height: u32,
    pub size_bytes: u64,
    pub ext: String,
    pub preview_data_url: String,
}

const OPAQUE: u8 = 255;
const MAX_OUTPUT_PX: u32 = 16_384;
/// The frontend rounds the canvas the same way; allow 1px of slack.
const CANVAS_TOLERANCE: u32 = 1;

/// Load the full-resolution image into state (replacing any previous crop
/// source) and return a downscaled preview for the stage.
#[tauri::command]
pub async fn open_for_crop(app: AppHandle, source: CropOpen) -> AppResult<CropSource> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        open_for_crop_blocking(&state, source)
    })
    .await
    .map_err(|_| AppError::Panic)?
}

/// Transform + crop + resize + encode on the full-resolution source.
#[tauri::command]
pub async fn render_crop(app: AppHandle, params: CropParams) -> AppResult<CropResult> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        render_crop_blocking(&state, params)
    })
    .await
    .map_err(|_| AppError::Panic)?
}

#[tauri::command]
pub async fn save_crop(app: AppHandle, result_id: String, dest_path: String) -> AppResult<()> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let bytes = lock(&state.crop_results)
            .get(&result_id)
            .map(|r| r.encoded.bytes.clone())
            .ok_or_else(|| AppError::NotFound(result_id.clone()))?;
        std::fs::write(&dest_path, bytes)?;
        Ok(())
    })
    .await
    .map_err(|_| AppError::Panic)?
}

/// Register the rendered crop as a converter item that is already converted.
#[tauri::command]
pub async fn send_crop_to_converter(app: AppHandle, result_id: String) -> AppResult<ImageItem> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        send_crop_to_converter_blocking(&state, &result_id)
    })
    .await
    .map_err(|_| AppError::Panic)?
}

pub(crate) fn open_for_crop_blocking(state: &AppState, source: CropOpen) -> AppResult<CropSource> {
    let (name, kind) = match source {
        CropOpen::Path { path } => {
            let path_buf = PathBuf::from(&path);
            if !decode::is_supported(&path_buf) {
                return Err(AppError::Unsupported(path));
            }
            let name = path_buf
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.clone());
            (name, SourceKind::Path(path_buf))
        }
        CropOpen::Item { id } => {
            let items = lock(&state.items);
            let item = items.get(&id).ok_or_else(|| AppError::NotFound(id.clone()))?;
            (item.name.clone(), item.source.clone())
        }
    };

    let img = decode::load_oriented(&kind)?.into_rgba8();
    let has_alpha = img.pixels().any(|p| p[3] != OPAQUE);
    let (width, height) = img.dimensions();
    let dynamic = DynamicImage::ImageRgba8(img);
    let (preview_data_url, preview_scale) =
        thumbnail::preview_data_url(&dynamic, thumbnail::PREVIEW_MAX_PX)?;
    let image = dynamic.into_rgba8();

    let id = state.next_id("crop");
    *lock(&state.crop_source) = Some(CropSourceState {
        id: id.clone(),
        name: name.clone(),
        image,
    });
    lock(&state.crop_results).clear();

    Ok(CropSource {
        id,
        name,
        width,
        height,
        has_alpha,
        preview_data_url,
        preview_scale,
    })
}

pub(crate) fn render_crop_blocking(state: &AppState, params: CropParams) -> AppResult<CropResult> {
    validate(&params)?;

    let source_guard = lock(&state.crop_source);
    let source = source_guard
        .as_ref()
        .ok_or_else(|| AppError::NotFound("No image is open in the crop tab".into()))?;
    if source.id != params.source_id {
        return Err(AppError::Invalid("The crop source changed; open the image again".into()));
    }

    let (w, h) = source.image.dimensions();
    let expected = canvas_size(w, h, &params.transform);
    let canvas = (params.canvas_width, params.canvas_height);
    if expected.0.abs_diff(canvas.0) > CANVAS_TOLERANCE
        || expected.1.abs_diff(canvas.1) > CANVAS_TOLERANCE
    {
        return Err(AppError::Invalid(format!(
            "Canvas mismatch (frontend {}x{}, backend {}x{})",
            canvas.0, canvas.1, expected.0, expected.1
        )));
    }

    let transformed = apply_transform(&source.image, &params.transform, canvas)?;
    drop(source_guard);

    let (x, y, cw, ch) = clamp_rect(&params.crop, canvas);
    let pixels = crop_and_resize(&transformed, x, y, cw, ch, params.output_width, params.output_height);
    drop(transformed);

    let dynamic = DynamicImage::ImageRgba8(pixels);
    let options = EncodeOptions {
        format: params.format,
        quality: params.quality,
        lossless: params.lossless,
    };
    let encoded = encode(&dynamic, &options, rayon::current_num_threads())?;
    let (preview_data_url, _) = thumbnail::preview_data_url(&dynamic, thumbnail::RESULT_PREVIEW_PX)?;
    let pixels = dynamic.into_rgba8();

    let result_id = state.next_id("result");
    let result = CropResult {
        result_id: result_id.clone(),
        width: encoded.width,
        height: encoded.height,
        size_bytes: encoded.bytes.len() as u64,
        ext: encoded.ext.to_string(),
        preview_data_url,
    };
    let mut results = lock(&state.crop_results);
    results.clear(); // keep only the latest render in memory
    results.insert(
        result_id,
        CropRendered {
            encoded: ConvertedImage {
                bytes: encoded.bytes,
                ext: encoded.ext,
                width: encoded.width,
                height: encoded.height,
            },
            pixels,
        },
    );
    Ok(result)
}

pub(crate) fn send_crop_to_converter_blocking(state: &AppState, result_id: &str) -> AppResult<ImageItem> {
    let source_name = lock(&state.crop_source)
        .as_ref()
        .map(|s| s.name.clone())
        .unwrap_or_else(|| "image".into());

    let rendered_guard = lock(&state.crop_results);
    let rendered = rendered_guard
        .get(result_id)
        .ok_or_else(|| AppError::NotFound(result_id.to_string()))?;

    // The converter re-decodes items when converting again, and the image
    // crate cannot read AVIF, so keep a lossless PNG of the pixels as source.
    let source_png = fast_png(&rendered.pixels)?;
    let thumb = imageops_thumbnail(&rendered.pixels)?;
    let encoded = rendered.encoded.clone();
    drop(rendered_guard);

    let (width, height) = (encoded.width, encoded.height);
    let name = naming::crop_name(&source_name, encoded.ext);
    let id = state.next_id("img");
    let size_bytes = encoded.bytes.len() as u64;
    let info = ConvertedInfo {
        size_bytes,
        ext: encoded.ext.to_string(),
        width,
        height,
    };

    lock(&state.items).insert(
        id.clone(),
        StoredItem {
            id: id.clone(),
            name: name.clone(),
            source: SourceKind::Memory(Arc::new(source_png)),
            key: None,
        },
    );
    lock(&state.results).insert(id.clone(), encoded);

    Ok(ImageItem {
        id,
        name,
        path: None,
        size_bytes,
        width,
        height,
        thumb_data_url: thumb,
        preconverted: Some(info),
    })
}

fn validate(params: &CropParams) -> AppResult<()> {
    let t = &params.transform;
    if !(t.rotation_deg.is_finite() && t.skew_x_deg.is_finite() && t.skew_y_deg.is_finite()) {
        return Err(AppError::Invalid("Transform angles must be finite".into()));
    }
    if t.skew_x_deg.abs() > MAX_SKEW_DEG || t.skew_y_deg.abs() > MAX_SKEW_DEG {
        return Err(AppError::Invalid(format!("Skew must stay within ±{MAX_SKEW_DEG}°")));
    }
    let out_ok = |v: u32| (1..=MAX_OUTPUT_PX).contains(&v);
    if !out_ok(params.output_width) || !out_ok(params.output_height) {
        return Err(AppError::Invalid(format!("Output size must be 1..={MAX_OUTPUT_PX} px")));
    }
    if params.canvas_width == 0 || params.canvas_height == 0 {
        return Err(AppError::Invalid("Canvas must not be empty".into()));
    }
    if !(params.crop.width >= 1.0 && params.crop.height >= 1.0) {
        return Err(AppError::Invalid("Crop area is empty".into()));
    }
    Ok(())
}

/// Round the crop rect to whole pixels and keep it inside the canvas.
fn clamp_rect(rect: &CropRect, canvas: (u32, u32)) -> (u32, u32, u32, u32) {
    let max_x = f64::from(canvas.0);
    let max_y = f64::from(canvas.1);
    let x0 = rect.x.round().clamp(0.0, max_x - 1.0);
    let y0 = rect.y.round().clamp(0.0, max_y - 1.0);
    let x1 = (rect.x + rect.width).round().clamp(x0 + 1.0, max_x);
    let y1 = (rect.y + rect.height).round().clamp(y0 + 1.0, max_y);
    (x0 as u32, y0 as u32, (x1 - x0) as u32, (y1 - y0) as u32)
}

fn fast_png(pixels: &image::RgbaImage) -> AppResult<Vec<u8>> {
    let mut buf = Vec::new();
    let encoder = PngEncoder::new_with_quality(&mut buf, CompressionType::Fast, PngFilter::NoFilter);
    encoder
        .write_image(pixels.as_raw(), pixels.width(), pixels.height(), ExtendedColorType::Rgba8)
        .map_err(|e| AppError::Encode(e.to_string()))?;
    Ok(buf)
}

fn imageops_thumbnail(pixels: &image::RgbaImage) -> AppResult<String> {
    let small = image::imageops::thumbnail(pixels, thumbnail::THUMB_PX, thumbnail::THUMB_PX);
    thumbnail::thumbnail_data_url(&DynamicImage::ImageRgba8(small))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    fn open_marker(state: &AppState, test: &str) -> CropSource {
        let dir = std::env::temp_dir().join(format!("sharpr-crop-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(format!("marker-{test}.png"));
        let img = RgbaImage::from_fn(400, 200, |x, y| {
            if x < 200 && y < 100 {
                Rgba([255, 0, 0, 255])
            } else {
                Rgba([0, 0, 255, 255])
            }
        });
        img.save(&path).unwrap();
        open_for_crop_blocking(
            state,
            CropOpen::Path {
                path: path.to_string_lossy().into_owned(),
            },
        )
        .unwrap()
    }

    fn params(source: &CropSource, transform: TransformParams, canvas: (u32, u32), crop: CropRect) -> CropParams {
        CropParams {
            source_id: source.id.clone(),
            transform,
            canvas_width: canvas.0,
            canvas_height: canvas.1,
            crop,
            output_width: crop.width as u32,
            output_height: crop.height as u32,
            format: OutputFormat::Png,
            quality: 100,
            lossless: false,
        }
    }

    #[test]
    fn open_reports_dimensions_and_preview() {
        let state = AppState::default();
        let source = open_marker(&state, "open");
        assert_eq!((source.width, source.height), (400, 200));
        assert!(!source.has_alpha);
        assert!(source.preview_data_url.starts_with("data:image/jpeg;base64,"));
        assert_eq!(source.preview_scale, 1.0);
    }

    #[test]
    fn render_crop_at_identity_extracts_exact_region() {
        let state = AppState::default();
        let source = open_marker(&state, "identity");
        let crop = CropRect { x: 0.0, y: 0.0, width: 200.0, height: 100.0 };
        let p = params(&source, TransformParams::default(), (400, 200), crop);
        let result = render_crop_blocking(&state, p).unwrap();
        assert_eq!((result.width, result.height), (200, 100));
        assert_eq!(result.ext, "png");
        let rendered = lock(&state.crop_results);
        let pixels = &rendered.get(&result.result_id).unwrap().pixels;
        assert!(pixels.pixels().all(|p| *p == Rgba([255, 0, 0, 255])), "region is all red");
    }

    #[test]
    fn render_crop_after_rotation_uses_canvas_space() {
        let state = AppState::default();
        let source = open_marker(&state, "rotated");
        let transform = TransformParams { rotation_deg: 90.0, ..TransformParams::default() };
        // Rotated clockwise: red quadrant is now the top-right 100x200 of a 200x400 canvas.
        let crop = CropRect { x: 100.0, y: 0.0, width: 100.0, height: 200.0 };
        let p = params(&source, transform, (200, 400), crop);
        let result = render_crop_blocking(&state, p).unwrap();
        let rendered = lock(&state.crop_results);
        let pixels = &rendered.get(&result.result_id).unwrap().pixels;
        assert_eq!(pixels.dimensions(), (100, 200));
        assert!(pixels.pixels().all(|p| *p == Rgba([255, 0, 0, 255])));
    }

    #[test]
    fn render_rejects_stale_source_and_bad_canvas() {
        let state = AppState::default();
        let source = open_marker(&state, "stale");
        let crop = CropRect { x: 0.0, y: 0.0, width: 10.0, height: 10.0 };
        let mut stale = params(&source, TransformParams::default(), (400, 200), crop);
        stale.source_id = "crop-999".into();
        assert!(render_crop_blocking(&state, stale).is_err());
        let wrong_canvas = params(&source, TransformParams::default(), (300, 200), crop);
        assert!(render_crop_blocking(&state, wrong_canvas).is_err());
    }

    #[test]
    fn send_to_converter_registers_done_item() {
        let state = AppState::default();
        let source = open_marker(&state, "send");
        let crop = CropRect { x: 0.0, y: 0.0, width: 50.0, height: 40.0 };
        let mut p = params(&source, TransformParams::default(), (400, 200), crop);
        p.format = OutputFormat::Avif;
        p.quality = 50;
        let result = render_crop_blocking(&state, p).unwrap();
        let item = send_crop_to_converter_blocking(&state, &result.result_id).unwrap();
        assert_eq!(item.name, "marker-send-crop.avif");
        assert!(item.preconverted.is_some());
        assert!(item.thumb_data_url.starts_with("data:image/jpeg;base64,"));
        assert!(lock(&state.results).contains_key(&item.id));
        // The in-memory source must be decodable even though the output is AVIF.
        let stored = lock(&state.items);
        let SourceKind::Memory(bytes) = &stored.get(&item.id).unwrap().source else {
            panic!("expected memory source");
        };
        let decoded = decode::load_oriented(&SourceKind::Memory(Arc::clone(bytes))).unwrap();
        assert_eq!(decoded.width(), 50);
    }
}
