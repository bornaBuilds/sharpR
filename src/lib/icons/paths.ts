/** Stroke-only icon paths on a 24x24 grid (rendered by Icon.svelte). */
const ICON_PATHS = {
  "image-plus": [
    "M4 5h11M4 5v14a1 1 0 0 0 1 1h14a1 1 0 0 0 1-1v-9",
    "M19 2v6M16 5h6",
    "M4 16l4-4 4 4 3-3 5 5",
    "M9 9.5a.5.5 0 1 0 0-1 .5.5 0 0 0 0 1z",
  ],
  folder: [
    "M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z",
  ],
  "folder-down": [
    "M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z",
    "M12 10v6M9.5 13.5 12 16l2.5-2.5",
  ],
  trash: [
    "M4 7h16",
    "M10 11v6M14 11v6",
    "M5 7l1 12a2 2 0 0 0 2 2h8a2 2 0 0 0 2-2l1-12",
    "M9 7V4h6v3",
  ],
  crop: ["M6 2v14a2 2 0 0 0 2 2h14", "M18 22V8a2 2 0 0 0-2-2H2"],
  check: ["M5 12l5 5L20 7"],
  "chevron-down": ["M6 9l6 6 6-6"],
  close: ["M6 6l12 12M18 6L6 18"],
  spinner: ["M12 3a9 9 0 1 0 9 9"],
  download: ["M12 3v12", "M7 10l5 5 5-5", "M4 19h16"],
  archive: [
    "M3 5h18v4H3z",
    "M5 9v10a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1V9",
    "M10 13h4",
  ],
  "upload-cloud": [
    "M16 16l-4-4-4 4",
    "M12 12v9",
    "M20.4 16.6A5 5 0 0 0 18 7h-1.3A8 8 0 1 0 4 15.3",
  ],
  "zoom-in": [
    "M11 4a7 7 0 1 0 0 14 7 7 0 0 0 0-14z",
    "M21 21l-5-5",
    "M11 8v6M8 11h6",
  ],
  "zoom-out": [
    "M11 4a7 7 0 1 0 0 14 7 7 0 0 0 0-14z",
    "M21 21l-5-5",
    "M8 11h6",
  ],
  "rotate-left": ["M3 12a9 9 0 1 0 2.6-6.4", "M3 3v6h6"],
  "rotate-right": ["M21 12a9 9 0 1 1-2.6-6.4", "M21 3v6h-6"],
  "skew-x": ["M8 5h13l-5 14H3z"],
  "skew-y": ["M5 8v13l14-5V3z"],
  "flip-horizontal": ["M12 3v18", "M8 7L3 12l5 5", "M16 7l5 5-5 5"],
  "flip-vertical": ["M3 12h18", "M7 8l5-5 5 5", "M7 16l5 5 5-5"],
  reset: ["M4 4v6h6", "M4.6 15a8 8 0 1 0 .9-8.4"],
  lock: ["M6 11V8a6 6 0 0 1 12 0v3", "M5 11h14v10H5z"],
  unlock: ["M6 11V8a6 6 0 0 1 11.5-2.3", "M5 11h14v10H5z"],
  warning: ["M12 3l10 18H2z", "M12 10v4", "M12 18h.01"],
  send: ["M22 2L11 13", "M22 2l-7 20-4-9-9-4z"],
  plus: ["M12 5v14M5 12h14"],
  minus: ["M5 12h14"],
  error: ["M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18z", "M9 9l6 6M15 9l-6 6"],
  success: ["M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18z", "M8 12l3 3 5-6"],
  info: ["M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18z", "M12 11v5", "M12 8h.01"],
  images: ["M8 3h13v13H8z", "M3 8v13h13", "M8 13l3-3 4 4 2-2 4 4"],
  film: [
    "M4 4h16v16H4z",
    "M8 4v16M16 4v16",
    "M4 8h4M4 12h4M4 16h4M16 8h4M16 12h4M16 16h4",
  ],
} as const;

type IconName = keyof typeof ICON_PATHS;

export { ICON_PATHS };
export type { IconName };
