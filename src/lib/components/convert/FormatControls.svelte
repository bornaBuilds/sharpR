<script lang="ts">
import NumberField from "$lib/components/ui/NumberField.svelte";
import PillDropdown from "$lib/components/ui/PillDropdown.svelte";
import Slider from "$lib/components/ui/Slider.svelte";
import Toggle from "$lib/components/ui/Toggle.svelte";
import { type FormatTarget, settings } from "$lib/stores/settings.svelte";
import type { OutputFormat } from "$lib/tauri/types";

let { target, disabled = false }: { target: FormatTarget; disabled?: boolean } =
  $props();

const FORMATS: { value: OutputFormat; label: string }[] = [
  { value: "webp", label: "WebP" },
  { value: "avif", label: "AVIF" },
  { value: "jpeg", label: "JPEG" },
  { value: "png", label: "PNG" },
];

const fs = $derived(settings.value[target]);
const showQuality = $derived(
  fs.format !== "png" && !(fs.format === "webp" && fs.lossless),
);
</script>

<div class="flex flex-wrap items-center gap-4">
  <PillDropdown
    label="Output format"
    options={FORMATS}
    bind:value={settings.value[target].format}
    {disabled}
  />
  {#if showQuality}
    <Slider
      label="Quality"
      min={1}
      max={100}
      bind:value={settings.value[target].quality[settings.value[target].format]}
      {disabled}
    />
  {/if}
  {#if fs.format === "webp"}
    <Toggle
      label="Lossless"
      bind:checked={settings.value[target].lossless}
      {disabled}
    />
  {/if}
  {#if target === "convert"}
    <Toggle
      label="Max dimension"
      bind:checked={settings.value.convert.maxDimensionEnabled}
      {disabled}
    />
    {#if settings.value.convert.maxDimensionEnabled}
      <NumberField
        label=""
        suffix="px"
        min={16}
        max={16384}
        bind:value={settings.value.convert.maxDimension}
        {disabled}
      />
    {/if}
  {/if}
</div>
