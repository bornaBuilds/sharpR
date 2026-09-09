//! Crop-stage transform. Mirrors `src/lib/crop/transform.ts` exactly:
//!
//!   L = R(θ) · SkX(ax) · SkY(ay) · F(flipX, flipY)
//!   M = T(W'/2, H'/2) · L · T(-W/2, -H/2)          (source px → canvas px)
//!
//! Conventions: y-down pixel coordinates, degrees, positive rotation is
//! clockwise on screen. The two implementations share the fixtures below.

use image::imageops::{self, FilterType};
use image::{Rgba, RgbaImage};
use imageproc::geometric_transformations::{warp_into, Border, Interpolation, Projection};
use serde::Deserialize;

use crate::error::{AppError, AppResult};

#[derive(Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TransformParams {
    pub rotation_deg: f64,
    pub skew_x_deg: f64,
    pub skew_y_deg: f64,
    pub flip_x: bool,
    pub flip_y: bool,
}

impl Default for TransformParams {
    fn default() -> Self {
        Self {
            rotation_deg: 0.0,
            skew_x_deg: 0.0,
            skew_y_deg: 0.0,
            flip_x: false,
            flip_y: false,
        }
    }
}

/// Row-major 3x3 matrix.
pub type Mat3 = [f64; 9];

pub const MAX_SKEW_DEG: f64 = 45.0;
const SNAP_SCALE: f64 = 1e9;
const CEIL_EPSILON: f64 = 1e-6;
const QUARTER_TURN: f64 = 90.0;
const HALF_TURN: f64 = 180.0;
const FULL_TURN: f64 = 360.0;

/// Round trig results so 90° multiples are exactly 0/±1 (same as the TS side).
fn snap(value: f64) -> f64 {
    (value * SNAP_SCALE).round() / SNAP_SCALE
}

fn multiply(a: &Mat3, b: &Mat3) -> Mat3 {
    let mut out = [0.0; 9];
    for row in 0..3 {
        for col in 0..3 {
            out[row * 3 + col] = (0..3).map(|k| a[row * 3 + k] * b[k * 3 + col]).sum();
        }
    }
    out
}

fn translation(tx: f64, ty: f64) -> Mat3 {
    [1.0, 0.0, tx, 0.0, 1.0, ty, 0.0, 0.0, 1.0]
}

pub fn apply(m: &Mat3, x: f64, y: f64) -> (f64, f64) {
    (m[0] * x + m[1] * y + m[2], m[3] * x + m[4] * y + m[5])
}

pub fn clamp_skew(deg: f64) -> f64 {
    deg.clamp(-MAX_SKEW_DEG, MAX_SKEW_DEG)
}

/// Normalize to (-180, 180].
pub fn normalize_rotation(deg: f64) -> f64 {
    let mut out = deg % FULL_TURN;
    if out > HALF_TURN {
        out -= FULL_TURN;
    } else if out <= -HALF_TURN {
        out += FULL_TURN;
    }
    if out == 0.0 {
        0.0
    } else {
        out
    }
}

pub fn linear_part(p: &TransformParams) -> Mat3 {
    let fx = if p.flip_x { -1.0 } else { 1.0 };
    let fy = if p.flip_y { -1.0 } else { 1.0 };
    let flip: Mat3 = [fx, 0.0, 0.0, 0.0, fy, 0.0, 0.0, 0.0, 1.0];
    let tx = snap(clamp_skew(p.skew_x_deg).to_radians().tan());
    let ty = snap(clamp_skew(p.skew_y_deg).to_radians().tan());
    let skew_x: Mat3 = [1.0, tx, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
    let skew_y: Mat3 = [1.0, 0.0, 0.0, ty, 1.0, 0.0, 0.0, 0.0, 1.0];
    let rad = p.rotation_deg.to_radians();
    let c = snap(rad.cos());
    let s = snap(rad.sin());
    let rotate: Mat3 = [c, -s, 0.0, s, c, 0.0, 0.0, 0.0, 1.0];
    multiply(&multiply(&multiply(&rotate, &skew_x), &skew_y), &flip)
}

/// Bounding box of the transformed source: the expanded canvas size.
pub fn canvas_size(width: u32, height: u32, p: &TransformParams) -> (u32, u32) {
    let linear = linear_part(p);
    let hw = f64::from(width) / 2.0;
    let hh = f64::from(height) / 2.0;
    let mut max_x: f64 = 0.0;
    let mut max_y: f64 = 0.0;
    for (x, y) in [(-hw, -hh), (hw, -hh), (hw, hh), (-hw, hh)] {
        let (px, py) = apply(&linear, x, y);
        max_x = max_x.max(px.abs());
        max_y = max_y.max(py.abs());
    }
    let w = (2.0 * max_x - CEIL_EPSILON).ceil().max(1.0) as u32;
    let h = (2.0 * max_y - CEIL_EPSILON).ceil().max(1.0) as u32;
    (w, h)
}

/// Full forward matrix: source pixel → expanded-canvas pixel.
pub fn forward_matrix(width: u32, height: u32, p: &TransformParams, canvas: (u32, u32)) -> Mat3 {
    let center_out = translation(f64::from(canvas.0) / 2.0, f64::from(canvas.1) / 2.0);
    let center_in = translation(-f64::from(width) / 2.0, -f64::from(height) / 2.0);
    multiply(&multiply(&center_out, &linear_part(p)), &center_in)
}

/// `Some(quarter turns)` when the transform is an exact multiple of 90° with no skew.
fn quarter_turns(p: &TransformParams) -> Option<u32> {
    if p.skew_x_deg != 0.0 || p.skew_y_deg != 0.0 {
        return None;
    }
    let rot = normalize_rotation(p.rotation_deg);
    if rot % QUARTER_TURN != 0.0 {
        return None;
    }
    Some((rot / QUARTER_TURN).rem_euclid(4.0) as u32)
}

/// Apply the transform to `src`, producing an image of exactly `canvas` size.
/// Exposed corners are transparent black.
pub fn apply_transform(src: &RgbaImage, p: &TransformParams, canvas: (u32, u32)) -> AppResult<RgbaImage> {
    if let Some(turns) = quarter_turns(p) {
        return Ok(fit_to_canvas(exact_transform(src, p, turns), canvas));
    }
    let m = forward_matrix(src.width(), src.height(), p, canvas);
    let projection = Projection::from_matrix(m.map(|v| v as f32))
        .ok_or_else(|| AppError::Invalid("Transform is not invertible".into()))?;
    let mut out = RgbaImage::from_pixel(canvas.0, canvas.1, Rgba([0, 0, 0, 0]));
    warp_into(
        src,
        projection,
        Interpolation::Bicubic,
        Border::Constant(Rgba([0, 0, 0, 0])),
        &mut out,
    );
    Ok(out)
}

/// Flips and quarter turns are lossless: no resampling.
fn exact_transform(src: &RgbaImage, p: &TransformParams, turns: u32) -> RgbaImage {
    let flipped: Option<RgbaImage> = match (p.flip_x, p.flip_y) {
        (false, false) => None,
        (true, false) => Some(imageops::flip_horizontal(src)),
        (false, true) => Some(imageops::flip_vertical(src)),
        (true, true) => Some(imageops::rotate180(src)),
    };
    let base = flipped.as_ref().unwrap_or(src);
    match turns {
        1 => imageops::rotate90(base),
        2 => imageops::rotate180(base),
        3 => imageops::rotate270(base),
        _ => flipped.unwrap_or_else(|| src.clone()),
    }
}

/// Centre `img` on a transparent canvas of the requested size (no-op when equal).
fn fit_to_canvas(img: RgbaImage, canvas: (u32, u32)) -> RgbaImage {
    if img.dimensions() == canvas {
        return img;
    }
    let mut out = RgbaImage::from_pixel(canvas.0, canvas.1, Rgba([0, 0, 0, 0]));
    let dx = (i64::from(canvas.0) - i64::from(img.width())) / 2;
    let dy = (i64::from(canvas.1) - i64::from(img.height())) / 2;
    imageops::overlay(&mut out, &img, dx, dy);
    out
}

/// Crop then resize (Lanczos3) to the requested output size.
pub fn crop_and_resize(
    img: &RgbaImage,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    out_width: u32,
    out_height: u32,
) -> RgbaImage {
    let cropped = imageops::crop_imm(img, x, y, width, height).to_image();
    if (width, height) == (out_width, out_height) {
        cropped
    } else {
        imageops::resize(&cropped, out_width, out_height, FilterType::Lanczos3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params(rotation: f64, skew_x: f64, skew_y: f64, flip_x: bool, flip_y: bool) -> TransformParams {
        TransformParams {
            rotation_deg: rotation,
            skew_x_deg: skew_x,
            skew_y_deg: skew_y,
            flip_x,
            flip_y,
        }
    }

    fn map(w: u32, h: u32, p: &TransformParams, x: f64, y: f64) -> (f64, f64) {
        let canvas = canvas_size(w, h, p);
        apply(&forward_matrix(w, h, p, canvas), x, y)
    }

    fn close(a: (f64, f64), b: (f64, f64)) -> bool {
        (a.0 - b.0).abs() < 1e-9 && (a.1 - b.1).abs() < 1e-9
    }

    // Same fixtures as src/lib/crop/transform.test.ts.
    #[test]
    fn identity_keeps_canvas() {
        let p = TransformParams::default();
        assert_eq!(canvas_size(400, 200, &p), (400, 200));
        assert!(close(map(400, 200, &p, 0.0, 0.0), (0.0, 0.0)));
        assert!(close(map(400, 200, &p, 400.0, 200.0), (400.0, 200.0)));
    }

    #[test]
    fn rotate_90_swaps_canvas() {
        let p = params(90.0, 0.0, 0.0, false, false);
        assert_eq!(canvas_size(400, 200, &p), (200, 400));
        assert!(close(map(400, 200, &p, 0.0, 0.0), (200.0, 0.0)));
        assert!(close(map(400, 200, &p, 400.0, 200.0), (0.0, 400.0)));
    }

    #[test]
    fn flip_x_mirrors() {
        let p = params(0.0, 0.0, 0.0, true, false);
        assert_eq!(canvas_size(400, 200, &p), (400, 200));
        assert!(close(map(400, 200, &p, 0.0, 0.0), (400.0, 0.0)));
    }

    #[test]
    fn rotate_45_expands_by_sqrt2() {
        let p = params(45.0, 0.0, 0.0, false, false);
        assert_eq!(canvas_size(100, 100, &p), (142, 142));
        assert!(close(map(100, 100, &p, 50.0, 50.0), (71.0, 71.0)));
    }

    #[test]
    fn skew_x_45_doubles_extent() {
        let p = params(0.0, 45.0, 0.0, false, false);
        assert_eq!(canvas_size(200, 100, &p), (300, 100));
        assert!(close(map(200, 100, &p, 0.0, 0.0), (0.0, 0.0)));
        assert!(close(map(200, 100, &p, 200.0, 100.0), (300.0, 100.0)));
    }

    #[test]
    fn skew_is_clamped() {
        let a = canvas_size(200, 100, &params(0.0, 80.0, 0.0, false, false));
        let b = canvas_size(200, 100, &params(0.0, 45.0, 0.0, false, false));
        assert_eq!(a, b);
    }

    #[test]
    fn arbitrary_params_are_invertible() {
        let p = params(33.0, 12.0, -20.0, true, false);
        let canvas = canvas_size(640, 480, &p);
        let m = forward_matrix(640, 480, &p, canvas);
        assert!(Projection::from_matrix(m.map(|v| v as f32)).is_some());
    }

    #[test]
    fn normalizes_rotation() {
        assert_eq!(normalize_rotation(180.0), 180.0);
        assert_eq!(normalize_rotation(225.0), -135.0);
        assert_eq!(normalize_rotation(-180.0), 180.0);
        assert_eq!(normalize_rotation(360.0), 0.0);
        assert_eq!(normalize_rotation(-45.0), -45.0);
    }

    fn marker_image() -> RgbaImage {
        // 40x20, red top-left quadrant, blue elsewhere.
        RgbaImage::from_fn(40, 20, |x, y| {
            if x < 20 && y < 10 {
                Rgba([255, 0, 0, 255])
            } else {
                Rgba([0, 0, 255, 255])
            }
        })
    }

    #[test]
    fn exact_paths_match_expected_corners() {
        let src = marker_image();
        let red = Rgba([255, 0, 0, 255]);

        let p = params(90.0, 0.0, 0.0, false, false);
        let out = apply_transform(&src, &p, canvas_size(40, 20, &p)).unwrap();
        assert_eq!(out.dimensions(), (20, 40));
        assert_eq!(*out.get_pixel(19, 0), red, "clockwise: red moves to top-right");

        let p = params(0.0, 0.0, 0.0, true, false);
        let out = apply_transform(&src, &p, canvas_size(40, 20, &p)).unwrap();
        assert_eq!(*out.get_pixel(39, 0), red, "flip x: red moves to top-right");

        let p = params(-90.0, 0.0, 0.0, false, false);
        let out = apply_transform(&src, &p, canvas_size(40, 20, &p)).unwrap();
        assert_eq!(*out.get_pixel(0, 39), red, "counter-clockwise: red moves to bottom-left");
    }

    #[test]
    fn warp_path_agrees_with_matrix() {
        let src = marker_image();
        let p = params(30.0, 10.0, -5.0, false, true);
        let canvas = canvas_size(40, 20, &p);
        let out = apply_transform(&src, &p, canvas).unwrap();
        assert_eq!(out.dimensions(), canvas);
        // The centre of the red quadrant should land on red in the output.
        let (cx, cy) = apply(&forward_matrix(40, 20, &p, canvas), 10.0, 5.0);
        let px = out.get_pixel(cx.round() as u32, cy.round() as u32);
        assert!(px[0] > 200 && px[2] < 60, "expected red at mapped centre, got {px:?}");
        // A corner of the canvas is transparent.
        assert_eq!(out.get_pixel(0, 0)[3], 0);
    }

    #[test]
    fn crop_and_resize_scales() {
        let src = marker_image();
        let out = crop_and_resize(&src, 0, 0, 20, 10, 40, 20);
        assert_eq!(out.dimensions(), (40, 20));
        assert_eq!(*out.get_pixel(5, 5), Rgba([255, 0, 0, 255]));
    }
}
