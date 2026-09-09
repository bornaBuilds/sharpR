<script lang="ts">
import { crop } from "$lib/stores/crop.svelte";
import CropBox from "./CropBox.svelte";

const PADDING_PX = 48;

let viewportWidth = $state(0);
let viewportHeight = $state(0);

const fitScale = $derived.by(() => {
  const { width, height } = crop.canvas;
  if (width === 0 || height === 0 || viewportWidth === 0) {
    return 1;
  }
  const scale = Math.min(
    (viewportWidth - PADDING_PX) / width,
    (viewportHeight - PADDING_PX) / height,
    1,
  );
  return Math.max(scale, 0.01);
});
const viewScale = $derived(fitScale * crop.zoom);
const stageWidth = $derived(crop.canvas.width * viewScale);
const stageHeight = $derived(crop.canvas.height * viewScale);
</script>

<div
  class="checker custom-scrollbar flex h-full w-full overflow-auto rounded-[28px] p-6"
  bind:clientWidth={viewportWidth}
  bind:clientHeight={viewportHeight}
>
  {#if crop.source}
    <div
      class="relative m-auto shrink-0"
      style={`width: ${stageWidth}px; height: ${stageHeight}px`}
    >
      <div
        class="absolute top-0 left-0 origin-top-left"
        style={`width: ${crop.canvas.width}px; height: ${crop.canvas.height}px; transform: scale(${viewScale})`}
      >
        <img
          src={crop.source.previewDataUrl}
          alt=""
          draggable="false"
          class="absolute top-0 left-0 max-w-none select-none"
          style={`width: ${crop.source.width}px; height: ${crop.source.height}px; transform-origin: 0 0; transform: ${crop.cssMatrix}`}
        />
      </div>
      <CropBox {viewScale} />
    </div>
  {/if}
</div>
