import { commands } from "$lib/tauri/commands";
import { dialogs } from "$lib/tauri/dialogs";
import { errorMessage, outputName } from "$lib/utils/paths";
import { batchProgress } from "$lib/video/helpers";
import { posters } from "./posters.svelte";
import { settings } from "./settings.svelte";
import { toasts } from "./toasts.svelte";
import type {
  AddSummary,
  CompressSummary,
  VideoItem,
  VideoProgress,
  VideoResult,
} from "$lib/tauri/types";

type VideoRowStatus =
  | "idle"
  | "queued"
  | "compressing"
  | "finishing"
  | "done"
  | "error"
  | "cancelled";

interface VideoRow extends VideoItem {
  checked: boolean;
  status: VideoRowStatus;
  /** Encode progress 0..1 while compressing; null when unknown. */
  fraction: number | null;
  result: VideoResult | null;
  error: string | null;
}

type ProgressOf<T extends VideoProgress["type"]> = Extract<
  VideoProgress,
  { type: T }
>;

function toRow(item: VideoItem): VideoRow {
  return {
    ...item,
    checked: true,
    status: "idle",
    fraction: null,
    result: null,
    error: null,
  };
}

function summaryMessages(summary: AddSummary): string[] {
  const messages: string[] = [];
  if (summary.skippedUnsupported > 0) {
    messages.push(`${summary.skippedUnsupported} unsupported file(s) skipped`);
  }
  if (summary.skippedDuplicate > 0) {
    messages.push(`${summary.skippedDuplicate} duplicate(s) skipped`);
  }
  if (summary.failed > 0) {
    messages.push(`${summary.failed} file(s) could not be read as video`);
  }
  return messages;
}

function reportCompressSummary(summary: CompressSummary): void {
  if (summary.failed > 0) {
    toasts.push(`${summary.failed} video(s) failed to compress`, "error");
  } else if (summary.cancelled > 0) {
    toasts.push(`Cancelled after ${summary.done} video(s)`);
  } else {
    const kept =
      summary.keptOriginal > 0
        ? ` (${summary.keptOriginal} already optimized)`
        : "";
    toasts.push(`Compressed ${summary.done} video(s)${kept}`, "success");
  }
}

class VideoStore {
  rows = $state<VideoRow[]>([]);
  adding = $state(false);
  running = $state(false);
  batch = $state({ total: 0, finished: 0 });
  /** Share of the item currently encoding, for the overall progress bar. */
  currentFraction = $state<number | null>(null);

  checkedRows = $derived(this.rows.filter((r) => r.checked));
  doneRows = $derived(this.rows.filter((r) => r.status === "done"));
  progress = $derived(
    batchProgress(this.batch.finished, this.batch.total, this.currentFraction),
  );

  async addPaths(paths: string[]): Promise<void> {
    if (paths.length === 0) {
      return;
    }
    this.adding = true;
    try {
      const summary = await commands.addVideos(
        paths,
        settings.value.video.recursive,
        (item) => {
          this.rows.push(toRow(item));
        },
      );
      const messages = summaryMessages(summary);
      for (const message of messages) {
        toasts.push(message);
      }
      if (summary.added === 0 && messages.length === 0) {
        toasts.push("No supported videos found");
      }
    } catch (error) {
      toasts.push(errorMessage(error), "error");
    } finally {
      this.adding = false;
    }
  }

  async pickVideos(): Promise<void> {
    const paths = await dialogs.pickVideos();
    await this.addPaths(paths);
  }

  async pickFolder(): Promise<void> {
    const dir = await dialogs.pickFolder();
    if (dir) {
      await this.addPaths([dir]);
    }
  }

  async remove(id: string): Promise<void> {
    if (this.running) {
      return;
    }
    posters.closeIf(id);
    this.rows = this.rows.filter((r) => r.id !== id);
    try {
      await commands.removeVideo(id);
    } catch (error) {
      toasts.push(errorMessage(error), "error");
    }
  }

  async clear(): Promise<void> {
    if (this.running) {
      return;
    }
    posters.close();
    this.rows = [];
    this.batch = { total: 0, finished: 0 };
    try {
      await commands.clearVideos();
    } catch (error) {
      toasts.push(errorMessage(error), "error");
    }
  }

  setChecked(id: string, checked: boolean): void {
    const row = this.rows.find((r) => r.id === id);
    if (row) {
      row.checked = checked;
    }
  }

  async compress(): Promise<void> {
    if (this.running) {
      return;
    }
    const ids = this.checkedRows.map((r) => r.id);
    if (ids.length === 0) {
      return;
    }
    this.markQueued();
    this.running = true;
    this.batch = { total: ids.length, finished: 0 };
    this.currentFraction = null;
    try {
      const summary = await commands.compressVideos(
        ids,
        settings.videoOptions,
        (progress) => this.applyProgress(progress),
      );
      reportCompressSummary(summary);
    } catch (error) {
      toasts.push(errorMessage(error), "error");
      this.resetInFlight();
    } finally {
      this.running = false;
      this.currentFraction = null;
    }
  }

  cancel(): void {
    commands.cancelCompress().catch((error) => {
      toasts.push(errorMessage(error), "error");
    });
  }

  private markQueued(): void {
    for (const row of this.rows) {
      if (row.checked) {
        posters.closeIf(row.id);
        row.status = "queued";
        row.fraction = null;
        row.result = null;
        row.error = null;
      }
    }
  }

  private resetInFlight(): void {
    for (const row of this.rows) {
      if (["queued", "compressing", "finishing"].includes(row.status)) {
        row.status = "idle";
        row.fraction = null;
      }
    }
  }

  private applyProgress(progress: VideoProgress): void {
    const row = this.rows.find((r) => r.id === progress.id);
    if (!row) {
      return;
    }
    switch (progress.type) {
      case "started":
      case "progress":
        this.onEncoding(row, progress.type === "progress" ? progress : null);
        break;
      case "finishing":
        row.status = "finishing";
        this.currentFraction = 1;
        break;
      case "done":
        this.onDone(row, progress);
        break;
      case "error":
        row.error = progress.message;
        this.finish(row, "error");
        break;
      case "cancelled":
        this.finish(row, "cancelled");
        break;
      default:
        break;
    }
  }

  private onEncoding(
    row: VideoRow,
    progress: ProgressOf<"progress"> | null,
  ): void {
    row.status = "compressing";
    row.fraction = progress ? progress.fraction : 0;
    this.currentFraction = row.fraction;
  }

  private onDone(row: VideoRow, progress: ProgressOf<"done">): void {
    const { type: _type, id: _id, ...result } = progress;
    row.result = result;
    this.finish(row, "done");
  }

  private finish(row: VideoRow, status: VideoRowStatus): void {
    row.status = status;
    row.fraction = null;
    this.batch.finished += 1;
    this.currentFraction = null;
  }

  async saveOne(id: string): Promise<void> {
    const row = this.rows.find((r) => r.id === id);
    if (!row?.result) {
      return;
    }
    const name = outputName(row.name, row.result.ext);
    const dest = await dialogs.pickSaveFile(name, row.result.ext, "Save video");
    if (!dest) {
      return;
    }
    try {
      await commands.saveVideo(id, dest);
      toasts.push(`Saved ${name}`, "success");
    } catch (error) {
      toasts.push(errorMessage(error), "error");
    }
  }

  async saveToFolder(): Promise<void> {
    const ids = this.doneRows.map((r) => r.id);
    if (ids.length === 0) {
      return;
    }
    const dir = await dialogs.pickFolder("Save videos to folder");
    if (!dir) {
      return;
    }
    try {
      const summary = await commands.saveVideosToFolder(ids, dir);
      toasts.push(`Saved ${summary.saved} video(s) to folder`, "success");
    } catch (error) {
      toasts.push(errorMessage(error), "error");
    }
  }
}

const video = new VideoStore();

export { video };
export type { VideoRow, VideoRowStatus };
