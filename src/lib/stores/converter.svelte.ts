import { commands } from "$lib/tauri/commands";
import { dialogs } from "$lib/tauri/dialogs";
import { errorMessage, outputName } from "$lib/utils/paths";
import { settings } from "./settings.svelte";
import { toasts } from "./toasts.svelte";
import type {
  AddSummary,
  ConvertedInfo,
  ConvertProgress,
  ConvertSummary,
  ImageItem,
} from "$lib/tauri/types";

type RowStatus =
  | "idle"
  | "queued"
  | "converting"
  | "done"
  | "error"
  | "cancelled";

interface ConverterRow extends ImageItem {
  checked: boolean;
  status: RowStatus;
  result: ConvertedInfo | null;
  error: string | null;
}

const ZIP_DEFAULT_NAME = "sharpr-images.zip";

function toRow(item: ImageItem): ConverterRow {
  return {
    ...item,
    checked: true,
    status: item.preconverted ? "done" : "idle",
    result: item.preconverted,
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
    messages.push(`${summary.failed} file(s) could not be decoded`);
  }
  return messages;
}

function reportConvertSummary(summary: ConvertSummary): void {
  if (summary.failed > 0) {
    toasts.push(`${summary.failed} image(s) failed to convert`, "error");
  } else if (summary.cancelled > 0) {
    toasts.push(`Cancelled after ${summary.done} image(s)`);
  } else {
    toasts.push(`Converted ${summary.done} image(s)`, "success");
  }
}

class ConverterStore {
  rows = $state<ConverterRow[]>([]);
  adding = $state(false);
  running = $state(false);
  batch = $state({ total: 0, finished: 0 });

  checkedRows = $derived(this.rows.filter((r) => r.checked));
  doneRows = $derived(this.rows.filter((r) => r.status === "done"));
  progress = $derived(
    this.batch.total === 0 ? 0 : this.batch.finished / this.batch.total,
  );

  async addPaths(paths: string[]): Promise<void> {
    if (paths.length === 0) {
      return;
    }
    this.adding = true;
    try {
      const summary = await commands.addPaths(
        paths,
        settings.value.convert.recursive,
        (item) => {
          this.rows.push(toRow(item));
        },
      );
      const messages = summaryMessages(summary);
      for (const message of messages) {
        toasts.push(message);
      }
      if (summary.added === 0 && messages.length === 0) {
        toasts.push("No supported images found");
      }
    } catch (error) {
      toasts.push(errorMessage(error), "error");
    } finally {
      this.adding = false;
    }
  }

  async pickImages(): Promise<void> {
    const paths = await dialogs.pickImages();
    await this.addPaths(paths);
  }

  async pickFolder(): Promise<void> {
    const dir = await dialogs.pickFolder();
    if (dir) {
      await this.addPaths([dir]);
    }
  }

  addPreconverted(item: ImageItem): void {
    this.rows.push(toRow(item));
  }

  async remove(id: string): Promise<void> {
    this.rows = this.rows.filter((r) => r.id !== id);
    try {
      await commands.removeItem(id);
    } catch (error) {
      toasts.push(errorMessage(error), "error");
    }
  }

  async clear(): Promise<void> {
    const ids = this.rows.map((r) => r.id);
    this.rows = [];
    this.batch = { total: 0, finished: 0 };
    const results = await Promise.allSettled(
      ids.map((id) => commands.removeItem(id)),
    );
    if (results.some((r) => r.status === "rejected")) {
      toasts.push("Some items could not be released", "error");
    }
  }

  setChecked(id: string, checked: boolean): void {
    const row = this.rows.find((r) => r.id === id);
    if (row) {
      row.checked = checked;
    }
  }

  setAllChecked(checked: boolean): void {
    for (const row of this.rows) {
      row.checked = checked;
    }
  }

  async convert(): Promise<void> {
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
    try {
      const summary = await commands.convert(
        ids,
        settings.convertOptions,
        (progress) => this.applyProgress(progress),
      );
      reportConvertSummary(summary);
    } catch (error) {
      toasts.push(errorMessage(error), "error");
      this.resetInFlight();
    } finally {
      this.running = false;
    }
  }

  private markQueued(): void {
    for (const row of this.rows) {
      if (row.checked) {
        row.status = "queued";
        row.result = null;
        row.error = null;
      }
    }
  }

  private resetInFlight(): void {
    for (const row of this.rows) {
      if (row.status === "queued" || row.status === "converting") {
        row.status = "idle";
      }
    }
  }

  cancel(): void {
    commands.cancelConvert().catch((error) => {
      toasts.push(errorMessage(error), "error");
    });
  }

  private applyProgress(progress: ConvertProgress): void {
    const row = this.rows.find((r) => r.id === progress.id);
    if (!row) {
      return;
    }
    switch (progress.type) {
      case "started":
        row.status = "converting";
        break;
      case "done":
        row.status = "done";
        row.result = {
          sizeBytes: progress.sizeBytes,
          ext: progress.ext,
          width: progress.width,
          height: progress.height,
        };
        this.batch.finished += 1;
        break;
      case "error":
        row.status = "error";
        row.error = progress.message;
        this.batch.finished += 1;
        break;
      case "cancelled":
        row.status = "cancelled";
        this.batch.finished += 1;
        break;
      default:
        break;
    }
  }

  async saveOne(id: string): Promise<void> {
    const row = this.rows.find((r) => r.id === id);
    if (!row?.result) {
      return;
    }
    const dest = await dialogs.pickSaveFile(
      outputName(row.name, row.result.ext),
      row.result.ext,
    );
    if (!dest) {
      return;
    }
    try {
      await commands.saveItem(id, dest);
      toasts.push(`Saved ${outputName(row.name, row.result.ext)}`, "success");
    } catch (error) {
      toasts.push(errorMessage(error), "error");
    }
  }

  async saveZip(): Promise<void> {
    const ids = this.doneRows.map((r) => r.id);
    if (ids.length === 0) {
      return;
    }
    const dest = await dialogs.pickSaveZip(ZIP_DEFAULT_NAME);
    if (!dest) {
      return;
    }
    try {
      const summary = await commands.saveZip(ids, dest);
      toasts.push(`Saved ${summary.saved} image(s) to ZIP`, "success");
    } catch (error) {
      toasts.push(errorMessage(error), "error");
    }
  }

  async saveToFolder(): Promise<void> {
    const ids = this.doneRows.map((r) => r.id);
    if (ids.length === 0) {
      return;
    }
    const dir = await dialogs.pickFolder("Save images to folder");
    if (!dir) {
      return;
    }
    try {
      const summary = await commands.saveToFolder(ids, dir);
      toasts.push(`Saved ${summary.saved} image(s) to folder`, "success");
    } catch (error) {
      toasts.push(errorMessage(error), "error");
    }
  }
}

const converter = new ConverterStore();

export { converter };
export type { ConverterRow, RowStatus };
