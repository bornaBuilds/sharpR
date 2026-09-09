<script lang="ts">
import { type Handle, moveRect, resizeRect } from "$lib/crop/aspect";
import { crop } from "$lib/stores/crop.svelte";
import type { CropRect } from "$lib/tauri/types";

let { viewScale }: { viewScale: number } = $props();

type DragMode = Handle | "move";

interface Drag {
  mode: DragMode;
  startX: number;
  startY: number;
  start: CropRect;
}

const HANDLES: { id: Handle; cursor: string; position: string }[] = [
  { id: "nw", cursor: "nwse-resize", position: "left: 0; top: 0" },
  { id: "n", cursor: "ns-resize", position: "left: 50%; top: 0" },
  { id: "ne", cursor: "nesw-resize", position: "left: 100%; top: 0" },
  { id: "e", cursor: "ew-resize", position: "left: 100%; top: 50%" },
  { id: "se", cursor: "nwse-resize", position: "left: 100%; top: 100%" },
  { id: "s", cursor: "ns-resize", position: "left: 50%; top: 100%" },
  { id: "sw", cursor: "nesw-resize", position: "left: 0; top: 100%" },
  { id: "w", cursor: "ew-resize", position: "left: 0; top: 50%" },
];

let drag = $state<Drag | null>(null);

const stage = $derived({
  w: crop.canvas.width * viewScale,
  h: crop.canvas.height * viewScale,
});
const box = $derived({
  x: crop.rect.x * viewScale,
  y: crop.rect.y * viewScale,
  w: crop.rect.width * viewScale,
  h: crop.rect.height * viewScale,
});
const maskPath = $derived(
  `M0 0H${stage.w}V${stage.h}H0Z M${box.x} ${box.y}H${box.x + box.w}V${box.y + box.h}H${box.x}Z`,
);

function begin(event: PointerEvent, mode: DragMode): void {
  if (event.button !== 0) {
    return;
  }
  event.preventDefault();
  event.stopPropagation();
  (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  drag = {
    mode,
    startX: event.clientX,
    startY: event.clientY,
    start: $state.snapshot(crop.rect),
  };
}

function move(event: PointerEvent): void {
  if (!drag) {
    return;
  }
  const dx = (event.clientX - drag.startX) / viewScale;
  const dy = (event.clientY - drag.startY) / viewScale;
  const next =
    drag.mode === "move"
      ? moveRect(drag.start, dx, dy, crop.canvas)
      : resizeRect(drag.start, drag.mode, dx, dy, crop.ratioValue, crop.canvas);
  crop.setRect(next);
}

function end(): void {
  drag = null;
}
</script>

<svg
  class="pointer-events-none absolute top-0 left-0"
  width={stage.w}
  height={stage.h}
  aria-hidden="true"
>
  <path d={maskPath} fill="rgb(29 28 44 / 0.65)" fill-rule="evenodd" />
</svg>

<div
  role="application"
  aria-label="Crop area. Drag to move, drag the handles to resize."
  class="absolute cursor-move touch-none border-2 border-bid-gold border-dashed"
  style={`left: ${box.x}px; top: ${box.y}px; width: ${box.w}px; height: ${box.h}px`}
  onpointerdown={(e) => begin(e, "move")}
  onpointermove={move}
  onpointerup={end}
  onpointercancel={end}
>
  <div class="pointer-events-none absolute inset-0" aria-hidden="true">
    <div class="absolute top-0 bottom-0 left-1/3 w-px bg-white-text/25"></div>
    <div class="absolute top-0 bottom-0 left-2/3 w-px bg-white-text/25"></div>
    <div class="absolute top-1/3 right-0 left-0 h-px bg-white-text/25"></div>
    <div class="absolute top-2/3 right-0 left-0 h-px bg-white-text/25"></div>
  </div>
  {#each HANDLES as handle (handle.id)}
    <div
      role="presentation"
      class="absolute size-3.5 rounded-full border-2 border-bid-purple bg-bid-gold"
      style={`${handle.position}; transform: translate(-50%, -50%); cursor: ${handle.cursor}`}
      onpointerdown={(e) => begin(e, handle.id)}
      onpointermove={move}
      onpointerup={end}
      onpointercancel={end}
    ></div>
  {/each}
</div>
