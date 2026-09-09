import { Channel, invoke } from "@tauri-apps/api/core";
import type {
  AddSummary,
  ConvertOptions,
  ConvertProgress,
  ConvertSummary,
  CropOpen,
  CropParams,
  CropResult,
  CropSource,
  ImageItem,
  SaveSummary,
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
};

export { commands };
