<script lang="ts">
import FormatControls from "$lib/components/convert/FormatControls.svelte";
import GlassPanel from "$lib/components/layout/GlassPanel.svelte";
import PillButton from "$lib/components/ui/PillButton.svelte";
import Icon from "$lib/icons/Icon.svelte";
import { crop } from "$lib/stores/crop.svelte";
import AspectPresets from "./AspectPresets.svelte";
import CropResultCard from "./CropResultCard.svelte";
import OutputSizeFields from "./OutputSizeFields.svelte";
</script>

<GlassPanel class="custom-scrollbar flex min-h-0 flex-col gap-6 overflow-y-auto px-6 py-5">
  <section>
    <h2 class="mb-3 text-xs font-semibold text-white-text/50 uppercase tracking-wider">
      Aspect ratio
    </h2>
    <AspectPresets />
  </section>

  <section>
    <h2 class="mb-3 text-xs font-semibold text-white-text/50 uppercase tracking-wider">
      Output size
    </h2>
    <OutputSizeFields />
  </section>

  <section>
    <h2 class="mb-3 text-xs font-semibold text-white-text/50 uppercase tracking-wider">
      Format
    </h2>
    <FormatControls target="crop" disabled={!crop.source} />
  </section>

  <PillButton
    class="w-full"
    disabled={!crop.source || crop.rendering}
    onclick={() => crop.render()}
  >
    {#if crop.rendering}
      <Icon name="spinner" class="size-5 animate-spin" />Rendering…
    {:else}
      <Icon name="crop" class="size-5" />Crop
    {/if}
  </PillButton>

  <CropResultCard />
</GlassPanel>
