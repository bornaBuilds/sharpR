import { open, save } from "@tauri-apps/plugin-dialog";
import { SUPPORTED_EXTENSIONS } from "$lib/utils/paths";

const IMAGE_FILTER = {
  name: "Images",
  extensions: [...SUPPORTED_EXTENSIONS],
};

async function pickImages(): Promise<string[]> {
  const result = await open({
    multiple: true,
    directory: false,
    filters: [IMAGE_FILTER],
    title: "Select images",
  });
  if (!result) {
    return [];
  }
  return Array.isArray(result) ? result : [result];
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
): Promise<string | null> {
  return save({
    defaultPath: defaultName,
    filters: [{ name: ext.toUpperCase(), extensions: [ext] }],
    title: "Save image",
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
  pickImage,
  pickFolder,
  pickSaveFile,
  pickSaveZip,
};

export { dialogs };
