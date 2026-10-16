<script lang="ts">
import PillDropdown from "$lib/components/ui/PillDropdown.svelte";
import Toggle from "$lib/components/ui/Toggle.svelte";
import { settings } from "$lib/stores/settings.svelte";
import type { VideoFormat, VideoQuality } from "$lib/tauri/types";
import type { MaxResolution } from "$lib/video/helpers";

let { disabled = false }: { disabled?: boolean } = $props();

const FORMATS: { value: VideoFormat; label: string }[] = [
  { value: "mp4", label: "MP4" },
  { value: "webm", label: "WebM" },
];

const QUALITIES: { value: VideoQuality; label: string }[] = [
  { value: "high", label: "High quality" },
  { value: "balanced", label: "Balanced" },
  { value: "small", label: "Small file" },
  { value: "smallest", label: "Smallest file" },
];

const RESOLUTIONS: { value: MaxResolution; label: string }[] = [
  { value: "original", label: "Original size" },
  { value: "1080", label: "Max 1080p" },
  { value: "720", label: "Max 720p" },
  { value: "480", label: "Max 480p" },
];
</script>

<div class="flex flex-wrap items-center gap-4">
  <PillDropdown
    label="Output format"
    options={FORMATS}
    bind:value={settings.value.video.format}
    {disabled}
  />
  <PillDropdown
    label="Quality"
    options={QUALITIES}
    bind:value={settings.value.video.quality}
    {disabled}
  />
  <PillDropdown
    label="Maximum resolution"
    options={RESOLUTIONS}
    bind:value={settings.value.video.maxResolution}
    {disabled}
  />
  <Toggle
    label="Remove audio"
    bind:checked={settings.value.video.removeAudio}
    {disabled}
  />
</div>
