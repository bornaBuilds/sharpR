<script lang="ts">
import Icon from "$lib/icons/Icon.svelte";
import { ui } from "$lib/stores/ui.svelte";

const message = $derived(
  ui.activeTab === "convert"
    ? "Drop images or folders to add them"
    : "Drop an image to crop it",
);
</script>

{#if ui.dragHover}
  <div
    class="pointer-events-none fixed inset-0 z-40 flex items-center justify-center bg-bid-purple/70 backdrop-blur-sm"
  >
    <div class="flex flex-col items-center gap-4 text-center">
      <Icon name="upload-cloud" class="size-16 text-bid-gold" />
      <p class="text-xl font-semibold">{message}</p>
    </div>
  </div>
{/if}
