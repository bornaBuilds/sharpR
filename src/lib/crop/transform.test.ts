import { describe, expect, it } from "vitest";
import {
  apply,
  canvasSize,
  defaultParams,
  forwardMatrix,
  invert,
  multiply,
  normalizeRotation,
  toCssMatrix,
} from "./transform";
import type { TransformParams } from "$lib/tauri/types";

function params(overrides: Partial<TransformParams>): TransformParams {
  return { ...defaultParams(), ...overrides };
}

function mapPoint(
  w: number,
  h: number,
  p: TransformParams,
  x: number,
  y: number,
): [number, number] {
  const canvas = canvasSize(w, h, p);
  return apply(forwardMatrix(w, h, p, canvas), x, y);
}

// The same fixtures live in src-tauri/src/imaging/transform.rs.
describe("crop transform", () => {
  it("identity keeps the canvas equal to the source", () => {
    const p = defaultParams();
    expect(canvasSize(400, 200, p)).toEqual({ width: 400, height: 200 });
    expect(mapPoint(400, 200, p, 0, 0)).toEqual([0, 0]);
    expect(mapPoint(400, 200, p, 400, 200)).toEqual([400, 200]);
  });

  it("rotates 90° clockwise into a swapped canvas", () => {
    const p = params({ rotationDeg: 90 });
    expect(canvasSize(400, 200, p)).toEqual({ width: 200, height: 400 });
    expect(mapPoint(400, 200, p, 0, 0)).toEqual([200, 0]);
    expect(mapPoint(400, 200, p, 400, 200)).toEqual([0, 400]);
  });

  it("flips horizontally without changing the canvas", () => {
    const p = params({ flipX: true });
    expect(canvasSize(400, 200, p)).toEqual({ width: 400, height: 200 });
    expect(mapPoint(400, 200, p, 0, 0)).toEqual([400, 0]);
  });

  it("rotates 45° into a √2 canvas with the centre fixed", () => {
    const p = params({ rotationDeg: 45 });
    expect(canvasSize(100, 100, p)).toEqual({ width: 142, height: 142 });
    const [cx, cy] = mapPoint(100, 100, p, 50, 50);
    expect(cx).toBeCloseTo(71, 9);
    expect(cy).toBeCloseTo(71, 9);
  });

  it("skews X by 45° doubling the horizontal extent", () => {
    const p = params({ skewXDeg: 45 });
    expect(canvasSize(200, 100, p)).toEqual({ width: 300, height: 100 });
    expect(mapPoint(200, 100, p, 0, 0)).toEqual([0, 0]);
    expect(mapPoint(200, 100, p, 200, 100)).toEqual([300, 100]);
  });

  it("clamps skew to ±45°", () => {
    expect(canvasSize(200, 100, params({ skewXDeg: 80 }))).toEqual(
      canvasSize(200, 100, params({ skewXDeg: 45 })),
    );
  });

  it("is invertible for arbitrary parameters", () => {
    const p = params({
      rotationDeg: 33,
      skewXDeg: 12,
      skewYDeg: -20,
      flipX: true,
      flipY: false,
    });
    const canvas = canvasSize(640, 480, p);
    const m = forwardMatrix(640, 480, p, canvas);
    const inv = invert(m);
    expect(inv).not.toBeNull();
    if (!inv) {
      return;
    }
    const identity = multiply(m, inv);
    const expected = [1, 0, 0, 0, 1, 0, 0, 0, 1];
    for (let i = 0; i < 9; i += 1) {
      expect(identity[i]).toBeCloseTo(expected[i], 9);
    }
  });

  it("emits a CSS matrix in column order", () => {
    expect(toCssMatrix([1, 2, 3, 4, 5, 6, 0, 0, 1])).toBe(
      "matrix(1, 4, 2, 5, 3, 6)",
    );
  });

  it("normalizes rotation into (-180, 180]", () => {
    expect(normalizeRotation(180)).toBe(180);
    expect(normalizeRotation(225)).toBe(-135);
    expect(normalizeRotation(-180)).toBe(180);
    expect(normalizeRotation(360)).toBe(0);
    expect(normalizeRotation(-45)).toBe(-45);
  });
});
