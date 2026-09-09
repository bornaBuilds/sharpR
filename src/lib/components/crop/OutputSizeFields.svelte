<script lang="ts">
import Badge from "$lib/components/ui/Badge.svelte";
import IconButton from "$lib/components/ui/IconButton.svelte";
import NumberField from "$lib/components/ui/NumberField.svelte";
import { crop } from "$lib/stores/crop.svelte";

const disabled = $derived(!crop.source);
</script>

<div class="flex items-center gap-2">
  <NumberField
    label="W"
    class="w-20"
    min={1}
    max={16384}
    value={crop.output.width}
    oncommit={(w) => crop.setOutputWidth(w)}
    {disabled}
  />
  <IconButton
    icon={crop.output.locked ? "lock" : "unlock"}
    label={crop.output.locked ? "Unlock aspect ratio" : "Lock aspect ratio"}
    active={crop.output.locked}
    iconClass="size-4"
    class="size-8"
    onclick={() => crop.toggleLock()}
    {disabled}
  />
  <NumberField
    label="H"
    class="w-20"
    min={1}
    max={16384}
    value={crop.output.height}
    oncommit={(h) => crop.setOutputHeight(h)}
    {disabled}
  />
  {#if crop.output.touched}
    <IconButton
      icon="reset"
      label="Use crop size"
      iconClass="size-4"
      class="size-8"
      onclick={() => crop.resetOutput()}
    />
  {/if}
</div>
<p class="mt-2 text-xs text-white-text/50">
  Crop area: {crop.roundedRect.width} × {crop.roundedRect.height} px
</p>
{#if crop.upscaling}
  <div class="mt-2">
    <Badge kind="warning" icon="warning">Upscaling beyond the crop size</Badge>
  </div>
{/if}
