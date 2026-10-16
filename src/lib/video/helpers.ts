import type { OutputFormat, PosterPreview } from "$lib/tauri/types";

type MaxResolution = "original" | "1080" | "720" | "480";

/** File extensions the Rust encoder uses (`OutputFormat::ext`). */
const IMAGE_EXT: Record<OutputFormat, string> = {
  webp: "webp",
  avif: "avif",
  jpeg: "jpg",
  png: "png",
};

const CODEC_LABELS: Record<string, string> = {
  h264: "H.264",
  hevc: "HEVC",
  vp8: "VP8",
  vp9: "VP9",
  av1: "AV1",
  prores: "ProRes",
  mpeg4: "MPEG-4",
  mpeg2video: "MPEG-2",
  mpeg1video: "MPEG-1",
  mjpeg: "MJPEG",
  wmv3: "WMV",
  theora: "Theora",
};

/** Short-side pixel cap sent to Rust; null keeps the original size. */
function resolutionToCap(value: MaxResolution): number | null {
  return value === "original" ? null : Number(value);
}

/** Overall batch progress: finished items plus the running item's share. */
function batchProgress(
  finished: number,
  total: number,
  current: number | null,
): number {
  if (total === 0) {
    return 0;
  }
  return Math.min(1, (finished + (current ?? 0)) / total);
}

/** The first frame that isn't black (fade-ins), else the first frame. */
function defaultPosterIndex(frames: PosterPreview[]): number {
  return Math.max(
    frames.findIndex((frame) => !frame.dark),
    0,
  );
}

function codecLabel(codec: string): string {
  return CODEC_LABELS[codec] ?? codec.toUpperCase();
}

function imageExt(format: OutputFormat): string {
  return IMAGE_EXT[format];
}

export {
  batchProgress,
  codecLabel,
  defaultPosterIndex,
  imageExt,
  resolutionToCap,
};
export type { MaxResolution };
