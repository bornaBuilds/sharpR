import type { CropRect } from "$lib/tauri/types";
import type { Size } from "./transform";

interface AspectPreset {
  label: string;
  w: number;
  h: number;
}

type AspectChoice =
  | { kind: "free" }
  | { kind: "preset"; label: string }
  | { kind: "custom" };

type Handle = "n" | "s" | "e" | "w" | "ne" | "nw" | "se" | "sw";

const ASPECT_PRESETS: AspectPreset[] = [
  { label: "1:1", w: 1, h: 1 },
  { label: "16:9", w: 16, h: 9 },
  { label: "9:16", w: 9, h: 16 },
  { label: "4:3", w: 4, h: 3 },
  { label: "3:4", w: 3, h: 4 },
  { label: "3:2", w: 3, h: 2 },
  { label: "2:3", w: 2, h: 3 },
  { label: "21:9", w: 21, h: 9 },
  { label: "4:5", w: 4, h: 5 },
];

const MIN_CROP_PX = 8;

/** Ratio (width / height) for a choice, or null for free-form. */
function ratioOf(
  choice: AspectChoice,
  custom: { w: number; h: number },
): number | null {
  if (choice.kind === "free") {
    return null;
  }
  if (choice.kind === "custom") {
    return custom.w > 0 && custom.h > 0 ? custom.w / custom.h : null;
  }
  const preset = ASPECT_PRESETS.find((p) => p.label === choice.label);
  return preset ? preset.w / preset.h : null;
}

/** Largest rect of `ratio` (or the whole canvas) centred in the canvas. */
function fitRect(canvas: Size, ratio: number | null): CropRect {
  if (ratio === null || canvas.width === 0 || canvas.height === 0) {
    return { x: 0, y: 0, width: canvas.width, height: canvas.height };
  }
  let width = canvas.width;
  let height = width / ratio;
  if (height > canvas.height) {
    height = canvas.height;
    width = height * ratio;
  }
  return {
    x: (canvas.width - width) / 2,
    y: (canvas.height - height) / 2,
    width,
    height,
  };
}

/** Keep a rect inside the canvas, preserving its size where possible. */
function clampRect(rect: CropRect, canvas: Size): CropRect {
  const width = Math.min(Math.max(MIN_CROP_PX, rect.width), canvas.width);
  const height = Math.min(Math.max(MIN_CROP_PX, rect.height), canvas.height);
  const x = Math.min(Math.max(0, rect.x), canvas.width - width);
  const y = Math.min(Math.max(0, rect.y), canvas.height - height);
  return { x, y, width, height };
}

function moveRect(
  start: CropRect,
  dx: number,
  dy: number,
  canvas: Size,
): CropRect {
  return clampRect({ ...start, x: start.x + dx, y: start.y + dy }, canvas);
}

function resizeFree(
  start: CropRect,
  handle: Handle,
  dx: number,
  dy: number,
): CropRect {
  let { x, y, width, height } = start;
  if (handle.includes("e")) {
    width = Math.max(MIN_CROP_PX, width + dx);
  }
  if (handle.includes("w")) {
    const newWidth = Math.max(MIN_CROP_PX, width - dx);
    x += width - newWidth;
    width = newWidth;
  }
  if (handle.includes("s")) {
    height = Math.max(MIN_CROP_PX, height + dy);
  }
  if (handle.includes("n")) {
    const newHeight = Math.max(MIN_CROP_PX, height - dy);
    y += height - newHeight;
    height = newHeight;
  }
  return { x, y, width, height };
}

function resizeWithRatio(
  start: CropRect,
  handle: Handle,
  dx: number,
  dy: number,
  ratio: number,
): CropRect {
  const horizontalOnly = handle === "e" || handle === "w";
  const verticalOnly = handle === "n" || handle === "s";
  let width: number;
  if (verticalOnly) {
    const height = handle === "s" ? start.height + dy : start.height - dy;
    width = Math.max(MIN_CROP_PX, height * ratio);
  } else {
    const signed = handle.includes("w") ? -dx : dx;
    width = Math.max(MIN_CROP_PX, start.width + signed);
  }
  const height = width / ratio;

  // Anchor: the opposite corner/edge stays put; centred edges keep their centre.
  const anchorRight = handle.includes("w");
  const anchorBottom = handle.includes("n");
  let x = anchorRight ? start.x + start.width - width : start.x;
  let y = anchorBottom ? start.y + start.height - height : start.y;
  if (verticalOnly) {
    x = start.x + (start.width - width) / 2;
  }
  if (horizontalOnly) {
    y = start.y + (start.height - height) / 2;
  }
  return { x, y, width, height };
}

/** Shrink a ratio-locked rect (from its anchored edge) until it fits the canvas. */
function fitInsideKeepingRatio(
  rect: CropRect,
  ratio: number,
  canvas: Size,
  handle: Handle,
): CropRect {
  let { x, y, width, height } = rect;
  const overflowRight = x + width - canvas.width;
  const overflowBottom = y + height - canvas.height;
  const overflowLeft = -x;
  const overflowTop = -y;
  const maxOverflowX = Math.max(overflowLeft, overflowRight, 0);
  const maxOverflowY = Math.max(overflowTop, overflowBottom, 0);
  if (maxOverflowX === 0 && maxOverflowY === 0) {
    return rect;
  }
  const scale = Math.min(
    (width - maxOverflowX) / width,
    (height - maxOverflowY) / height,
  );
  const newWidth = Math.max(MIN_CROP_PX, width * scale);
  const newHeight = newWidth / ratio;
  if (handle.includes("w")) {
    x += width - newWidth;
  } else if (!handle.includes("e")) {
    x += (width - newWidth) / 2;
  }
  if (handle.includes("n")) {
    y += height - newHeight;
  } else if (!handle.includes("s")) {
    y += (height - newHeight) / 2;
  }
  width = newWidth;
  height = newHeight;
  return clampRect({ x, y, width, height }, canvas);
}

function resizeRect(
  start: CropRect,
  handle: Handle,
  dx: number,
  dy: number,
  ratio: number | null,
  canvas: Size,
): CropRect {
  if (ratio === null) {
    const free = resizeFree(start, handle, dx, dy);
    const x = Math.max(0, free.x);
    const y = Math.max(0, free.y);
    return {
      x,
      y,
      width: Math.min(free.width - (x - free.x), canvas.width - x),
      height: Math.min(free.height - (y - free.y), canvas.height - y),
    };
  }
  const resized = resizeWithRatio(start, handle, dx, dy, ratio);
  return fitInsideKeepingRatio(resized, ratio, canvas, handle);
}

function roundRect(rect: CropRect): CropRect {
  const x = Math.round(rect.x);
  const y = Math.round(rect.y);
  return {
    x,
    y,
    width: Math.max(1, Math.round(rect.x + rect.width) - x),
    height: Math.max(1, Math.round(rect.y + rect.height) - y),
  };
}

export {
  ASPECT_PRESETS,
  clampRect,
  fitRect,
  MIN_CROP_PX,
  moveRect,
  ratioOf,
  resizeRect,
  roundRect,
};
export type { AspectChoice, AspectPreset, Handle };
