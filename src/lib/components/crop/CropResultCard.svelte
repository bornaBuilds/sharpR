<script lang="ts">
import OutlinePillButton from "$lib/components/ui/OutlinePillButton.svelte";
import PillButton from "$lib/components/ui/PillButton.svelte";
import Icon from "$lib/icons/Icon.svelte";
import { crop } from "$lib/stores/crop.svelte";
import { formatBytes } from "$lib/utils/format";
</script>

{#if crop.result}
  <div class="flex flex-col gap-3">
    <div
      class="checker flex h-40 items-center justify-center overflow-hidden rounded-2xl"
    >
      <img
        src={crop.result.previewDataUrl}
        alt="Crop result preview"
        class="max-h-full max-w-full object-contain"
        draggable="false"
      />
    </div>
    <p class="text-sm text-white-text/70">
      {crop.result.width} × {crop.result.height} px ·
      {formatBytes(crop.result.sizeBytes)} · {crop.result.ext.toUpperCase()}
    </p>
    <div class="flex flex-col gap-2">
      <PillButton onclick={() => crop.save()}>
        <Icon name="download" class="size-5" />Save
      </PillButton>
      <OutlinePillButton onclick={() => crop.sendToConverter()}>
        <Icon name="send" class="size-4" />Send to converter
      </OutlinePillButton>
    </div>
  </div>
{/if}
