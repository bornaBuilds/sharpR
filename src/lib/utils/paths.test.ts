import { describe, expect, it } from "vitest";
import { isSupportedPath, isVideoPath, outputName, posterName } from "./paths";

describe("isVideoPath", () => {
  it("matches video containers case-insensitively", () => {
    expect(isVideoPath("/videos/Clip.MOV")).toBe(true);
    expect(isVideoPath("C:\\clips\\holiday.m2ts")).toBe(true);
    expect(isVideoPath("/src/app.ts")).toBe(false);
    expect(isVideoPath("/photos/a.jpg")).toBe(false);
    expect(isSupportedPath("/videos/a.mp4")).toBe(false);
  });
});

describe("output names", () => {
  it("swaps extensions and adds the poster suffix", () => {
    expect(outputName("holiday.mov", "mp4")).toBe("holiday.mp4");
    expect(posterName("holiday.mov", "webp")).toBe("holiday-poster.webp");
    expect(posterName("my.clip.mkv", "jpg")).toBe("my.clip-poster.jpg");
  });
});
