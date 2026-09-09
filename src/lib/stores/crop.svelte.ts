import {
  type AspectChoice,
  clampRect,
  fitRect,
  ratioOf,
  roundRect,
} from "$lib/crop/aspect";
import {
  canvasSize,
  clampSkew,
  defaultParams,
  forwardMatrix,
  IDENTITY,
  normalizeRotation,
  type Size,
  toCssMatrix,
} from "$lib/crop/transform";
import { commands } from "$lib/tauri/commands";
import { dialogs } from "$lib/tauri/dialogs";
import { cropName, errorMessage } from "$lib/utils/paths";
import { converter } from "./converter.svelte";
import { settings } from "./settings.svelte";
import { toasts } from "./toasts.svelte";
import { ui } from "./ui.svelte";
import type {
  CropOpen,
  CropRect,
  CropResult,
  CropSource,
  TransformParams,
} from "$lib/tauri/types";

const ZOOM_STEP = 1.25;
const MIN_ZOOM = 1;
const MAX_ZOOM = 8;
const MIN_OUTPUT_PX = 1;
const MAX_OUTPUT_PX = 16_384;
const DEFAULT_CUSTOM_RATIO = { w: 1200, h: 630 };

interface OutputSize {
  width: number;
  height: number;
  locked: boolean;
  touched: boolean;
}

function clampOutput(value: number): number {
  const rounded = Math.round(Number.isFinite(value) ? value : MIN_OUTPUT_PX);
  return Math.max(MIN_OUTPUT_PX, Math.min(MAX_OUTPUT_PX, rounded));
}

class CropStore {
  source = $state<CropSource | null>(null);
  loading = $state(false);
  params = $state<TransformParams>(defaultParams());
  zoom = $state(MIN_ZOOM);
  rect = $state<CropRect>({ x: 0, y: 0, width: 0, height: 0 });
  ratio = $state<AspectChoice>({ kind: "free" });
  custom = $state({ ...DEFAULT_CUSTOM_RATIO });
  output = $state<OutputSize>({
    width: 0,
    height: 0,
    locked: true,
    touched: false,
  });
  result = $state<CropResult | null>(null);
  rendering = $state(false);

  canvas = $derived<Size>(
    this.source
      ? canvasSize(this.source.width, this.source.height, this.params)
      : { width: 0, height: 0 },
  );
  matrix = $derived(
    this.source
      ? forwardMatrix(
          this.source.width,
          this.source.height,
          this.params,
          this.canvas,
        )
      : IDENTITY,
  );
  cssMatrix = $derived(toCssMatrix(this.matrix));
  ratioValue = $derived(ratioOf(this.ratio, this.custom));
  roundedRect = $derived(roundRect(this.rect));
  upscaling = $derived(
    this.output.width > this.roundedRect.width ||
      this.output.height > this.roundedRect.height,
  );

  async open(open: CropOpen): Promise<void> {
    this.loading = true;
    try {
      this.source = await commands.openForCrop(open);
      this.params = defaultParams();
      this.zoom = MIN_ZOOM;
      this.result = null;
      this.output = { width: 0, height: 0, locked: true, touched: false };
      this.refitRect();
      ui.activeTab = "crop";
    } catch (error) {
      toasts.push(errorMessage(error), "error");
    } finally {
      this.loading = false;
    }
  }

  async pickImage(): Promise<void> {
    const path = await dialogs.pickImage();
    if (path) {
      await this.open({ kind: "path", path });
    }
  }

  // ----- crop rectangle -----

  refitRect(): void {
    this.rect = fitRect(this.canvas, this.ratioValue);
    this.syncOutput();
  }

  setRect(rect: CropRect): void {
    this.rect = clampRect(rect, this.canvas);
    this.result = null;
    this.syncOutput();
  }

  setRatio(choice: AspectChoice): void {
    this.ratio = choice;
    this.result = null;
    this.refitRect();
  }

  setCustomRatio(w: number, h: number): void {
    this.custom = {
      w: Math.max(1, Math.round(w)),
      h: Math.max(1, Math.round(h)),
    };
    if (this.ratio.kind === "custom") {
      this.result = null;
      this.refitRect();
    }
  }

  // ----- output size -----

  private syncOutput(): void {
    const r = this.roundedRect;
    if (!this.output.touched) {
      this.output.width = r.width;
      this.output.height = r.height;
    } else if (this.output.locked) {
      this.output.height = clampOutput(
        (this.output.width * r.height) / r.width,
      );
    }
  }

  setOutputWidth(width: number): void {
    const r = this.roundedRect;
    this.output.touched = true;
    this.output.width = clampOutput(width);
    if (this.output.locked) {
      this.output.height = clampOutput(
        (this.output.width * r.height) / r.width,
      );
    }
    this.result = null;
  }

  setOutputHeight(height: number): void {
    const r = this.roundedRect;
    this.output.touched = true;
    this.output.height = clampOutput(height);
    if (this.output.locked) {
      this.output.width = clampOutput(
        (this.output.height * r.width) / r.height,
      );
    }
    this.result = null;
  }

  toggleLock(): void {
    this.output.locked = !this.output.locked;
    if (this.output.locked) {
      const r = this.roundedRect;
      this.output.height = clampOutput(
        (this.output.width * r.height) / r.width,
      );
      this.result = null;
    }
  }

  resetOutput(): void {
    this.output.touched = false;
    this.result = null;
    this.syncOutput();
  }

  // ----- transforms -----

  private afterTransform(): void {
    this.result = null;
    this.refitRect();
  }

  rotateBy(deg: number): void {
    this.params.rotationDeg = normalizeRotation(this.params.rotationDeg + deg);
    this.afterTransform();
  }

  setRotation(deg: number): void {
    this.params.rotationDeg = normalizeRotation(Number.isFinite(deg) ? deg : 0);
    this.afterTransform();
  }

  skewXBy(deg: number): void {
    this.params.skewXDeg = clampSkew(this.params.skewXDeg + deg);
    this.afterTransform();
  }

  skewYBy(deg: number): void {
    this.params.skewYDeg = clampSkew(this.params.skewYDeg + deg);
    this.afterTransform();
  }

  toggleFlipX(): void {
    this.params.flipX = !this.params.flipX;
    this.afterTransform();
  }

  toggleFlipY(): void {
    this.params.flipY = !this.params.flipY;
    this.afterTransform();
  }

  resetTransform(): void {
    this.params = defaultParams();
    this.afterTransform();
  }

  zoomIn(): void {
    this.zoom = Math.min(MAX_ZOOM, this.zoom * ZOOM_STEP);
  }

  zoomOut(): void {
    this.zoom = Math.max(MIN_ZOOM, this.zoom / ZOOM_STEP);
  }

  resetZoom(): void {
    this.zoom = MIN_ZOOM;
  }

  // ----- render / save / hand-off -----

  async render(): Promise<void> {
    if (!this.source || this.rendering) {
      return;
    }
    this.rendering = true;
    const options = settings.cropOptions;
    try {
      this.result = await commands.renderCrop({
        sourceId: this.source.id,
        ...$state.snapshot(this.params),
        canvasWidth: this.canvas.width,
        canvasHeight: this.canvas.height,
        crop: this.roundedRect,
        outputWidth: this.output.width,
        outputHeight: this.output.height,
        format: options.format,
        quality: options.quality,
        lossless: options.lossless,
      });
    } catch (error) {
      toasts.push(errorMessage(error), "error");
    } finally {
      this.rendering = false;
    }
  }

  async save(): Promise<void> {
    if (!(this.result && this.source)) {
      return;
    }
    const name = cropName(this.source.name, this.result.ext);
    const dest = await dialogs.pickSaveFile(name, this.result.ext);
    if (!dest) {
      return;
    }
    try {
      await commands.saveCrop(this.result.resultId, dest);
      toasts.push(`Saved ${name}`, "success");
    } catch (error) {
      toasts.push(errorMessage(error), "error");
    }
  }

  async sendToConverter(): Promise<void> {
    if (!this.result) {
      return;
    }
    try {
      const item = await commands.sendCropToConverter(this.result.resultId);
      converter.addPreconverted(item);
      toasts.push(`Added ${item.name} to the converter list`, "success");
    } catch (error) {
      toasts.push(errorMessage(error), "error");
    }
  }
}

const crop = new CropStore();

export { crop };
