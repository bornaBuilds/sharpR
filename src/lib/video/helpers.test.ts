import { describe, expect, it } from "vitest";
import {
  batchProgress,
  codecLabel,
  defaultPosterIndex,
  imageExt,
  resolutionToCap,
} from "./helpers";
import type { PosterPreview } from "$lib/tauri/types";

function frame(index: number, dark: boolean): PosterPreview {
  return {
    index,
    timeSec: 0.5 + index * 0.6,
    dark,
    width: 1280,
    height: 720,
    previewDataUrl: "",
  };
}

describe("resolutionToCap", () => {
  it("maps presets to short-side caps", () => {
    expect(resolutionToCap("original")).toBeNull();
    expect(resolutionToCap("1080")).toBe(1080);
    expect(resolutionToCap("480")).toBe(480);
  });
});

describe("batchProgress", () => {
  it("adds the running item's share", () => {
    expect(batchProgress(0, 0, null)).toBe(0);
    expect(batchProgress(1, 4, null)).toBe(0.25);
    expect(batchProgress(1, 4, 0.5)).toBeCloseTo(0.375);
    expect(batchProgress(4, 4, 1)).toBe(1);
  });
});

describe("defaultPosterIndex", () => {
  it("skips black fade-in frames", () => {
    expect(defaultPosterIndex([frame(0, true), frame(1, false)])).toBe(1);
    expect(defaultPosterIndex([frame(0, true), frame(1, true)])).toBe(0);
    expect(defaultPosterIndex([])).toBe(0);
  });
});

describe("labels and extensions", () => {
  it("names codecs and image extensions like the backend", () => {
    expect(codecLabel("h264")).toBe("H.264");
    expect(codecLabel("dnxhd")).toBe("DNXHD");
    expect(imageExt("jpeg")).toBe("jpg");
    expect(imageExt("webp")).toBe("webp");
  });
});
