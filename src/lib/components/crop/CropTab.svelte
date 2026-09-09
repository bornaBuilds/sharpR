<script lang="ts">
import GlassPanel from "$lib/components/layout/GlassPanel.svelte";
import PillButton from "$lib/components/ui/PillButton.svelte";
import Icon from "$lib/icons/Icon.svelte";
import { crop } from "$lib/stores/crop.svelte";
import { ui } from "$lib/stores/ui.svelte";
import CropSidebar from "./CropSidebar.svelte";
import CropStage from "./CropStage.svelte";
import CropToolbar from "./CropToolbar.svelte";
</script>

<div class="grid h-full grid-cols-[minmax(0,1fr)_360px] gap-4">
  <div class="flex min-h-0 flex-col gap-4">
    <CropToolbar />
    <GlassPanel
      dashed
      glow={ui.dragHover && ui.activeTab === "crop"}
      class="min-h-0 flex-1 overflow-hidden p-1"
    >
      {#if crop.source}
        <CropStage />
      {:else}
        <div class="flex h-full flex-col items-center justify-center gap-6 text-center">
          <Icon name="crop" class="size-20 text-bid-gold" />
          <div>
            <p class="text-2xl font-semibold">Open an image to crop</p>
            <p class="mt-1 text-white-text/60">
              Drop a file here, pick one, or use "Open in crop" on a converter row
            </p>
          </div>
          <PillButton onclick={() => crop.pickImage()} disabled={crop.loading}>
            {#if crop.loading}
              <Icon name="spinner" class="size-5 animate-spin" />Loading…
            {:else}
              <Icon name="images" class="size-5" />Open image
            {/if}
          </PillButton>
        </div>
      {/if}
    </GlassPanel>
  </div>
  <CropSidebar />
</div>
