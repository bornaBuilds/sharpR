import { open, save } from "@tauri-apps/plugin-dialog";
import { SUPPORTED_EXTENSIONS, VIDEO_EXTENSIONS } from "$lib/utils/paths";

const IMAGE_FILTER = {
  name: "Images",
  extensions: [...SUPPORTED_EXTENSIONS],
};

const VIDEO_FILTER = {
  name: "Videos",
  extensions: [...VIDEO_EXTENSIONS],
};

async function pickFiles(
  filter: { name: string; extensions: string[] },
  title: string,
): Promise<string[]> {
  const result = await open({
    multiple: true,
    directory: false,
    filters: [filter],
    title,
  });
  if (!result) {
    return [];
  }
  return Array.isArray(result) ? result : [result];
}

function pickImages(): Promise<string[]> {
  return pickFiles(IMAGE_FILTER, "Select images");
}

function pickVideos(): Promise<string[]> {
  return pickFiles(VIDEO_FILTER, "Select videos");
}

async function pickImage(): Promise<string | null> {
  const result = await open({
    multiple: false,
    directory: false,
    filters: [IMAGE_FILTER],
    title: "Open image",
  });
  return typeof result === "string" ? result : null;
}

async function pickFolder(title = "Select a folder"): Promise<string | null> {
  const result = await open({ directory: true, multiple: false, title });
  return typeof result === "string" ? result : null;
}

function pickSaveFile(
  defaultName: string,
  ext: string,
  title = "Save image",
): Promise<string | null> {
  return save({
    defaultPath: defaultName,
    filters: [{ name: ext.toUpperCase(), extensions: [ext] }],
    title,
  });
}

function pickSaveZip(defaultName: string): Promise<string | null> {
  return save({
    defaultPath: defaultName,
    filters: [{ name: "ZIP archive", extensions: ["zip"] }],
    title: "Save ZIP",
  });
}

const dialogs = {
  pickImages,
  pickVideos,
  pickImage,
  pickFolder,
  pickSaveFile,
  pickSaveZip,
};

export { dialogs };
