<script lang="ts">
import GlassPanel from "$lib/components/layout/GlassPanel.svelte";
import OutlinePillButton from "$lib/components/ui/OutlinePillButton.svelte";
import PillButton from "$lib/components/ui/PillButton.svelte";
import Toggle from "$lib/components/ui/Toggle.svelte";
import Icon from "$lib/icons/Icon.svelte";
import { settings } from "$lib/stores/settings.svelte";
import { video } from "$lib/stores/video.svelte";
import VideoFormatControls from "./VideoFormatControls.svelte";

const busy = $derived(video.adding || video.running);
</script>

<GlassPanel class="z-20 flex flex-wrap items-center gap-4 px-6 py-4">
  <PillButton onclick={() => video.pickVideos()} disabled={busy}>
    <Icon name="film" class="size-5" />Select videos
  </PillButton>
  <OutlinePillButton onclick={() => video.pickFolder()} disabled={busy}>
    <Icon name="folder" class="size-5" />Select folder
  </OutlinePillButton>
  <Toggle
    label="Include subfolders"
    bind:checked={settings.value.video.recursive}
  />
  <div class="mx-2 h-8 w-px bg-white-text/15"></div>
  <VideoFormatControls disabled={video.running} />
  <div class="flex-1"></div>
  {#if video.rows.length > 0}
    <OutlinePillButton
      onclick={() => video.clear()}
      disabled={busy}
      class="text-white-text/70"
    >
      <Icon name="close" class="size-4" />Clear list
    </OutlinePillButton>
  {/if}
</GlassPanel>
