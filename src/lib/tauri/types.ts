/** DTOs mirroring the Rust command contract (serde camelCase). */

type OutputFormat = "webp" | "avif" | "jpeg" | "png";

interface ConvertOptions {
  format: OutputFormat;
  quality: number;
  lossless: boolean;
  maxDimension: number | null;
}

interface ConvertedInfo {
  sizeBytes: number;
  ext: string;
  width: number;
  height: number;
}

interface ImageItem {
  id: string;
  name: string;
  path: string | null;
  sizeBytes: number;
  width: number;
  height: number;
  thumbDataUrl: string;
  preconverted: ConvertedInfo | null;
}

interface AddSummary {
  added: number;
  skippedUnsupported: number;
  skippedDuplicate: number;
  failed: number;
}

type ConvertProgress =
  | { type: "started"; id: string }
  | {
      type: "done";
      id: string;
      sizeBytes: number;
      ext: string;
      width: number;
      height: number;
    }
  | { type: "error"; id: string; message: string }
  | { type: "cancelled"; id: string };

interface ConvertSummary {
  done: number;
  failed: number;
  cancelled: number;
}

interface SaveSummary {
  saved: number;
  missing: number;
}

type CropOpen = { kind: "path"; path: string } | { kind: "item"; id: string };

interface CropSource {
  id: string;
  name: string;
  width: number;
  height: number;
  hasAlpha: boolean;
  previewDataUrl: string;
  previewScale: number;
}

interface CropRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

interface TransformParams {
  rotationDeg: number;
  skewXDeg: number;
  skewYDeg: number;
  flipX: boolean;
  flipY: boolean;
}

interface CropParams extends TransformParams {
  sourceId: string;
  canvasWidth: number;
  canvasHeight: number;
  crop: CropRect;
  outputWidth: number;
  outputHeight: number;
  format: OutputFormat;
  quality: number;
  lossless: boolean;
}

interface CropResult {
  resultId: string;
  width: number;
  height: number;
  sizeBytes: number;
  ext: string;
  previewDataUrl: string;
}

export type {
  AddSummary,
  ConvertedInfo,
  ConvertOptions,
  ConvertProgress,
  ConvertSummary,
  CropOpen,
  CropParams,
  CropRect,
  CropResult,
  CropSource,
  ImageItem,
  OutputFormat,
  SaveSummary,
  TransformParams,
};
