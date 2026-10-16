import { commands } from "$lib/tauri/commands";
import { dialogs } from "$lib/tauri/dialogs";
import { basename, errorMessage, posterName } from "$lib/utils/paths";
import { defaultPosterIndex, imageExt } from "$lib/video/helpers";
import { settings } from "./settings.svelte";
import { toasts } from "./toasts.svelte";
import type { PosterPreview } from "$lib/tauri/types";

/** The poster sidebar of the video tab: frames of one compressed video. */
class PostersStore {
  rowId = $state<string | null>(null);
  rowName = $state("");
  frames = $state<PosterPreview[]>([]);
  selected = $state(0);
  loading = $state(false);
  saving = $state(false);

  isOpen = $derived(this.rowId !== null);
  current = $derived(this.frames[this.selected] ?? null);

  /** Open the sidebar for a row, or close it if that row is already shown. */
  async toggle(id: string, name: string): Promise<void> {
    if (this.rowId === id) {
      this.close();
      return;
    }
    this.rowId = id;
    this.rowName = name;
    this.frames = [];
    this.selected = 0;
    this.loading = true;
    try {
      const frames = await commands.videoPosters(id);
      if (this.rowId !== id) {
        return; // another row was opened meanwhile
      }
      this.frames = frames;
      this.selected = defaultPosterIndex(frames);
      this.loading = false;
    } catch (error) {
      toasts.push(errorMessage(error), "error");
      this.closeIf(id);
    }
  }

  close(): void {
    this.rowId = null;
    this.frames = [];
    this.selected = 0;
    this.loading = false;
  }

  closeIf(id: string): void {
    if (this.rowId === id) {
      this.close();
    }
  }

  select(index: number): void {
    this.selected = index;
  }

  async save(): Promise<void> {
    const id = this.rowId;
    const frame = this.current;
    if (!(id && frame) || this.saving) {
      return;
    }
    const options = settings.posterOptions;
    const ext = imageExt(options.format);
    const dest = await dialogs.pickSaveFile(
      posterName(this.rowName, ext),
      ext,
      "Save poster",
    );
    if (!dest) {
      return;
    }
    this.saving = true;
    try {
      await commands.savePoster(id, frame.index, options, dest);
      toasts.push(`Saved ${basename(dest)}`, "success");
    } catch (error) {
      toasts.push(errorMessage(error), "error");
    } finally {
      this.saving = false;
    }
  }
}

const posters = new PostersStore();

export { posters };
