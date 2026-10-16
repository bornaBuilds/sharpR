const SUPPORTED_EXTENSIONS = [
  "jpg",
  "jpeg",
  "png",
  "webp",
  "gif",
  "bmp",
  "tif",
  "tiff",
] as const;

/** Keep in sync with VIDEO_EXTENSIONS in src-tauri/src/video/formats.rs. */
const VIDEO_EXTENSIONS = [
  "mp4",
  "m4v",
  "mov",
  "mkv",
  "webm",
  "avi",
  "wmv",
  "flv",
  "mpg",
  "mpeg",
  "3gp",
  "ogv",
  "mts",
  "m2ts",
] as const;

function basename(path: string): string {
  const cut = Math.max(path.lastIndexOf("/"), path.lastIndexOf("\\"));
  return path.slice(cut + 1);
}

function extOf(name: string): string {
  const dot = name.lastIndexOf(".");
  return dot < 0 ? "" : name.slice(dot + 1).toLowerCase();
}

function stem(name: string): string {
  const dot = name.lastIndexOf(".");
  return dot <= 0 ? name : name.slice(0, dot);
}

function isSupportedPath(path: string): boolean {
  const ext = extOf(basename(path));
  return (SUPPORTED_EXTENSIONS as readonly string[]).includes(ext);
}

function isVideoPath(path: string): boolean {
  const ext = extOf(basename(path));
  return (VIDEO_EXTENSIONS as readonly string[]).includes(ext);
}

function outputName(name: string, ext: string): string {
  return `${stem(name)}.${ext}`;
}

function cropName(name: string, ext: string): string {
  return `${stem(name)}-crop.${ext}`;
}

function posterName(name: string, ext: string): string {
  return `${stem(name)}-poster.${ext}`;
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export {
  basename,
  cropName,
  errorMessage,
  extOf,
  isSupportedPath,
  isVideoPath,
  outputName,
  posterName,
  SUPPORTED_EXTENSIONS,
  stem,
  VIDEO_EXTENSIONS,
};
