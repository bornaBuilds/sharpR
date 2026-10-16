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

/** Image encoder settings (poster export). */
interface EncodeOptions {
  format: OutputFormat;
  quality: number;
  lossless: boolean;
}

type VideoFormat = "mp4" | "webm";
type VideoQuality = "high" | "balanced" | "small" | "smallest";

interface VideoOptions {
  format: VideoFormat;
  quality: VideoQuality;
  /** Short-side cap in pixels; null keeps the original size. */
  maxResolution: number | null;
  removeAudio: boolean;
}

interface VideoItem {
  id: string;
  name: string;
  path: string;
  sizeBytes: number;
  /** Display size (rotation and pixel aspect applied). */
  width: number;
  height: number;
  durationSec: number | null;
  videoCodec: string;
  hasAudio: boolean;
  hdr: boolean;
  thumbDataUrl: string;
}

interface VideoResult {
  sizeBytes: number;
  ext: string;
  width: number;
  height: number;
  /** Re-encoding saved too little, so the source was kept (stream copy). */
  keptOriginal: boolean;
  posterCount: number;
}

type VideoProgress =
  | { type: "started"; id: string }
  | { type: "progress"; id: string; fraction: number | null }
  | { type: "finishing"; id: string }
  | ({ type: "done"; id: string } & VideoResult)
  | { type: "error"; id: string; message: string }
  | { type: "cancelled"; id: string };

interface CompressSummary {
  done: number;
  failed: number;
  cancelled: number;
  keptOriginal: number;
}

interface PosterPreview {
  index: number;
  timeSec: number;
  dark: boolean;
  width: number;
  height: number;
  previewDataUrl: string;
}

export type {
  AddSummary,
  CompressSummary,
  ConvertedInfo,
  ConvertOptions,
  ConvertProgress,
  ConvertSummary,
  CropOpen,
  CropParams,
  CropRect,
  CropResult,
  CropSource,
  EncodeOptions,
  ImageItem,
  OutputFormat,
  PosterPreview,
  SaveSummary,
  TransformParams,
  VideoFormat,
  VideoItem,
  VideoOptions,
  VideoProgress,
  VideoQuality,
  VideoResult,
};
