<script lang="ts">
import NumberField from "$lib/components/ui/NumberField.svelte";
import { ASPECT_PRESETS } from "$lib/crop/aspect";
import { crop } from "$lib/stores/crop.svelte";

const disabled = $derived(!crop.source);
</script>

<div class="flex flex-wrap gap-2">
  <button
    type="button"
    class="chip"
    data-active={crop.ratio.kind === "free"}
    onclick={() => crop.setRatio({ kind: "free" })}
    {disabled}
  >
    Free
  </button>
  {#each ASPECT_PRESETS as preset (preset.label)}
    <button
      type="button"
      class="chip"
      data-active={crop.ratio.kind === "preset" && crop.ratio.label === preset.label}
      onclick={() => crop.setRatio({ kind: "preset", label: preset.label })}
      {disabled}
    >
      {preset.label}
    </button>
  {/each}
  <button
    type="button"
    class="chip"
    data-active={crop.ratio.kind === "custom"}
    onclick={() => crop.setRatio({ kind: "custom" })}
    {disabled}
  >
    Custom
  </button>
</div>
{#if crop.ratio.kind === "custom"}
  <div class="mt-3 flex items-center gap-2">
    <NumberField
      label=""
      class="w-20"
      min={1}
      value={crop.custom.w}
      oncommit={(w) => crop.setCustomRatio(w, crop.custom.h)}
    />
    <span class="text-white-text/60">:</span>
    <NumberField
      label=""
      class="w-20"
      min={1}
      value={crop.custom.h}
      oncommit={(h) => crop.setCustomRatio(crop.custom.w, h)}
    />
  </div>
{/if}
