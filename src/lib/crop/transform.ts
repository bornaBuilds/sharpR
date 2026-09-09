/**
 * Crop-stage transform math. This file is mirrored 1:1 by
 * `src-tauri/src/imaging/transform.rs`; both must produce identical numbers.
 *
 * Conventions: y-down pixel coordinates, angles in degrees, positive rotation
 * is clockwise on screen (CSS convention).
 *
 *   L = R(θ) · SkX(ax) · SkY(ay) · F(flipX, flipY)         (linear part)
 *   M = T(W'/2, H'/2) · L · T(-W/2, -H/2)                  (source px → canvas px)
 *
 * where (W', H') is the bounding box of the four transformed source corners.
 */

import type { TransformParams } from "$lib/tauri/types";

/** Row-major 3x3 matrix. */
type Mat3 = [
  number,
  number,
  number,
  number,
  number,
  number,
  number,
  number,
  number,
];

interface Size {
  width: number;
  height: number;
}

const IDENTITY: Mat3 = [1, 0, 0, 0, 1, 0, 0, 0, 1];
const SNAP_SCALE = 1e9;
const CEIL_EPSILON = 1e-6;
const MAX_SKEW_DEG = 45;
const HALF_TURN_DEG = 180;
const FULL_TURN_DEG = 360;

/** Round trig results so 90° multiples are exactly 0/±1 in both JS and Rust. */
function snap(value: number): number {
  return Math.round(value * SNAP_SCALE) / SNAP_SCALE;
}

function degToRad(deg: number): number {
  return (deg * Math.PI) / HALF_TURN_DEG;
}

function multiply(a: Mat3, b: Mat3): Mat3 {
  const out: number[] = new Array(9).fill(0);
  for (let row = 0; row < 3; row += 1) {
    for (let col = 0; col < 3; col += 1) {
      let sum = 0;
      for (let k = 0; k < 3; k += 1) {
        sum += a[row * 3 + k] * b[k * 3 + col];
      }
      out[row * 3 + col] = sum;
    }
  }
  return out as Mat3;
}

function translation(tx: number, ty: number): Mat3 {
  return [1, 0, tx, 0, 1, ty, 0, 0, 1];
}

function apply(m: Mat3, x: number, y: number): [number, number] {
  return [m[0] * x + m[1] * y + m[2], m[3] * x + m[4] * y + m[5]];
}

function defaultParams(): TransformParams {
  return {
    rotationDeg: 0,
    skewXDeg: 0,
    skewYDeg: 0,
    flipX: false,
    flipY: false,
  };
}

function isIdentityParams(p: TransformParams): boolean {
  return (
    p.rotationDeg === 0 &&
    p.skewXDeg === 0 &&
    p.skewYDeg === 0 &&
    !p.flipX &&
    !p.flipY
  );
}

function clampSkew(deg: number): number {
  return Math.max(-MAX_SKEW_DEG, Math.min(MAX_SKEW_DEG, deg));
}

/** Normalize to (-180, 180]. */
function normalizeRotation(deg: number): number {
  let out = deg % FULL_TURN_DEG;
  if (out > HALF_TURN_DEG) {
    out -= FULL_TURN_DEG;
  } else if (out <= -HALF_TURN_DEG) {
    out += FULL_TURN_DEG;
  }
  return Object.is(out, -0) ? 0 : out;
}

/** The linear (rotation · skew · flip) part of the transform, about the origin. */
function linearPart(p: TransformParams): Mat3 {
  const fx = p.flipX ? -1 : 1;
  const fy = p.flipY ? -1 : 1;
  const flip: Mat3 = [fx, 0, 0, 0, fy, 0, 0, 0, 1];
  const tx = snap(Math.tan(degToRad(clampSkew(p.skewXDeg))));
  const ty = snap(Math.tan(degToRad(clampSkew(p.skewYDeg))));
  const skewX: Mat3 = [1, tx, 0, 0, 1, 0, 0, 0, 1];
  const skewY: Mat3 = [1, 0, 0, ty, 1, 0, 0, 0, 1];
  const rad = degToRad(p.rotationDeg);
  const c = snap(Math.cos(rad));
  const s = snap(Math.sin(rad));
  const rotate: Mat3 = [c, -s, 0, s, c, 0, 0, 0, 1];
  return multiply(multiply(multiply(rotate, skewX), skewY), flip);
}

/** Bounding box of the transformed source, i.e. the expanded canvas size. */
function canvasSize(width: number, height: number, p: TransformParams): Size {
  const linear = linearPart(p);
  const hw = width / 2;
  const hh = height / 2;
  const corners: [number, number][] = [
    [-hw, -hh],
    [hw, -hh],
    [hw, hh],
    [-hw, hh],
  ];
  let maxX = 0;
  let maxY = 0;
  for (const [x, y] of corners) {
    const [px, py] = apply(linear, x, y);
    maxX = Math.max(maxX, Math.abs(px));
    maxY = Math.max(maxY, Math.abs(py));
  }
  return {
    width: Math.max(1, Math.ceil(2 * maxX - CEIL_EPSILON)),
    height: Math.max(1, Math.ceil(2 * maxY - CEIL_EPSILON)),
  };
}

/** Full forward matrix: source pixel → expanded-canvas pixel. */
function forwardMatrix(
  width: number,
  height: number,
  p: TransformParams,
  canvas: Size,
): Mat3 {
  return multiply(
    multiply(translation(canvas.width / 2, canvas.height / 2), linearPart(p)),
    translation(-width / 2, -height / 2),
  );
}

/** CSS `matrix(a, b, c, d, e, f)` maps (x, y) → (a·x + c·y + e, b·x + d·y + f). */
function toCssMatrix(m: Mat3): string {
  return `matrix(${m[0]}, ${m[3]}, ${m[1]}, ${m[4]}, ${m[2]}, ${m[5]})`;
}

function invert(m: Mat3): Mat3 | null {
  const [a, b, c, d, e, f, g, h, i] = m;
  const det = a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g);
  if (Math.abs(det) < Number.EPSILON) {
    return null;
  }
  return [
    (e * i - f * h) / det,
    (c * h - b * i) / det,
    (b * f - c * e) / det,
    (f * g - d * i) / det,
    (a * i - c * g) / det,
    (c * d - a * f) / det,
    (d * h - e * g) / det,
    (b * g - a * h) / det,
    (a * e - b * d) / det,
  ];
}

export {
  apply,
  canvasSize,
  clampSkew,
  defaultParams,
  forwardMatrix,
  IDENTITY,
  invert,
  isIdentityParams,
  linearPart,
  MAX_SKEW_DEG,
  multiply,
  normalizeRotation,
  toCssMatrix,
};
export type { Mat3, Size };
