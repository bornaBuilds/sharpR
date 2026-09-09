<script lang="ts">
import GlassPanel from "$lib/components/layout/GlassPanel.svelte";
import OutlinePillButton from "$lib/components/ui/OutlinePillButton.svelte";
import PillButton from "$lib/components/ui/PillButton.svelte";
import Toggle from "$lib/components/ui/Toggle.svelte";
import Icon from "$lib/icons/Icon.svelte";
import { converter } from "$lib/stores/converter.svelte";
import { settings } from "$lib/stores/settings.svelte";
import FormatControls from "./FormatControls.svelte";

const busy = $derived(converter.adding || converter.running);
</script>

<GlassPanel class="z-20 flex flex-wrap items-center gap-4 px-6 py-4">
  <PillButton onclick={() => converter.pickImages()} disabled={busy}>
    <Icon name="image-plus" class="size-5" />Select images
  </PillButton>
  <OutlinePillButton onclick={() => converter.pickFolder()} disabled={busy}>
    <Icon name="folder" class="size-5" />Select folder
  </OutlinePillButton>
  <Toggle
    label="Include subfolders"
    bind:checked={settings.value.convert.recursive}
  />
  <div class="mx-2 h-8 w-px bg-white-text/15"></div>
  <FormatControls target="convert" disabled={converter.running} />
  <div class="flex-1"></div>
  {#if converter.rows.length > 0}
    <OutlinePillButton
      onclick={() => converter.clear()}
      disabled={busy}
      class="text-white-text/70"
    >
      <Icon name="close" class="size-4" />Clear list
    </OutlinePillButton>
  {/if}
</GlassPanel>
