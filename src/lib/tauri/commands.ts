import { Channel, invoke } from "@tauri-apps/api/core";
import type {
  AddSummary,
  CompressSummary,
  ConvertOptions,
  ConvertProgress,
  ConvertSummary,
  CropOpen,
  CropParams,
  CropResult,
  CropSource,
  EncodeOptions,
  ImageItem,
  PosterPreview,
  SaveSummary,
  VideoItem,
  VideoOptions,
  VideoProgress,
} from "./types";

function addPaths(
  paths: string[],
  recursive: boolean,
  onItem: (item: ImageItem) => void,
): Promise<AddSummary> {
  const channel = new Channel<ImageItem>();
  channel.onmessage = onItem;
  return invoke<AddSummary>("add_paths", { paths, recursive, onItem: channel });
}

function removeItem(id: string): Promise<void> {
  return invoke<void>("remove_item", { id });
}

function convert(
  ids: string[],
  options: ConvertOptions,
  onProgress: (progress: ConvertProgress) => void,
): Promise<ConvertSummary> {
  const channel = new Channel<ConvertProgress>();
  channel.onmessage = onProgress;
  return invoke<ConvertSummary>("convert", {
    ids,
    options,
    onProgress: channel,
  });
}

function cancelConvert(): Promise<void> {
  return invoke<void>("cancel_convert");
}

function saveItem(id: string, destPath: string): Promise<void> {
  return invoke<void>("save_item", { id, destPath });
}

function saveZip(ids: string[], destPath: string): Promise<SaveSummary> {
  return invoke<SaveSummary>("save_zip", { ids, destPath });
}

function saveToFolder(ids: string[], dir: string): Promise<SaveSummary> {
  return invoke<SaveSummary>("save_to_folder", { ids, dir });
}

function openForCrop(source: CropOpen): Promise<CropSource> {
  return invoke<CropSource>("open_for_crop", { source });
}

function renderCrop(params: CropParams): Promise<CropResult> {
  return invoke<CropResult>("render_crop", { params });
}

function saveCrop(resultId: string, destPath: string): Promise<void> {
  return invoke<void>("save_crop", { resultId, destPath });
}

function sendCropToConverter(resultId: string): Promise<ImageItem> {
  return invoke<ImageItem>("send_crop_to_converter", { resultId });
}

function addVideos(
  paths: string[],
  recursive: boolean,
  onItem: (item: VideoItem) => void,
): Promise<AddSummary> {
  const channel = new Channel<VideoItem>();
  channel.onmessage = onItem;
  return invoke<AddSummary>("add_videos", {
    paths,
    recursive,
    onItem: channel,
  });
}

function removeVideo(id: string): Promise<void> {
  return invoke<void>("remove_video", { id });
}

function clearVideos(): Promise<void> {
  return invoke<void>("clear_videos");
}

function compressVideos(
  ids: string[],
  options: VideoOptions,
  onProgress: (progress: VideoProgress) => void,
): Promise<CompressSummary> {
  const channel = new Channel<VideoProgress>();
  channel.onmessage = onProgress;
  return invoke<CompressSummary>("compress_videos", {
    ids,
    options,
    onProgress: channel,
  });
}

function cancelCompress(): Promise<void> {
  return invoke<void>("cancel_compress");
}

function videoPosters(id: string): Promise<PosterPreview[]> {
  return invoke<PosterPreview[]>("video_posters", { id });
}

function saveVideo(id: string, destPath: string): Promise<void> {
  return invoke<void>("save_video", { id, destPath });
}

function saveVideosToFolder(ids: string[], dir: string): Promise<SaveSummary> {
  return invoke<SaveSummary>("save_videos_to_folder", { ids, dir });
}

function savePoster(
  id: string,
  index: number,
  options: EncodeOptions,
  destPath: string,
): Promise<void> {
  return invoke<void>("save_poster", { id, index, options, destPath });
}

const commands = {
  addPaths,
  removeItem,
  convert,
  cancelConvert,
  saveItem,
  saveZip,
  saveToFolder,
  openForCrop,
  renderCrop,
  saveCrop,
  sendCropToConverter,
  addVideos,
  removeVideo,
  clearVideos,
  compressVideos,
  cancelCompress,
  videoPosters,
  saveVideo,
  saveVideosToFolder,
  savePoster,
};

export { commands };
