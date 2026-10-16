import { describe, expect, it } from "vitest";
import {
  formatBytes,
  formatDuration,
  formatPercent,
  percentChange,
} from "./format";

describe("formatDuration", () => {
  it("formats minutes and hours", () => {
    expect(formatDuration(7)).toBe("0:07");
    expect(formatDuration(62.4)).toBe("1:02");
    expect(formatDuration(754)).toBe("12:34");
    expect(formatDuration(3723)).toBe("1:02:03");
  });

  it("shows a dash for unknown durations", () => {
    expect(formatDuration(null)).toBe("—");
    expect(formatDuration(Number.NaN)).toBe("—");
  });
});

describe("byte and percent helpers", () => {
  it("formats sizes and changes", () => {
    expect(formatBytes(512)).toBe("512 B");
    expect(formatBytes(1536)).toBe("1.5 KB");
    expect(formatBytes(250 * 1024 * 1024)).toBe("250 MB");
    expect(percentChange(200, 50)).toBe(-75);
    expect(percentChange(0, 50)).toBe(0);
    expect(formatPercent(12)).toBe("+12%");
    expect(formatPercent(-30)).toBe("-30%");
  });
});
