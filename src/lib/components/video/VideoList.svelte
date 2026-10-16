<script lang="ts">
import GlassPanel from "$lib/components/layout/GlassPanel.svelte";
import Icon from "$lib/icons/Icon.svelte";
import { ui } from "$lib/stores/ui.svelte";
import { video } from "$lib/stores/video.svelte";
import VideoEmptyState from "./VideoEmptyState.svelte";
import VideoRow from "./VideoRow.svelte";

type PanelStatus = "default" | "pending" | "success" | "error";

const status = $derived.by((): PanelStatus => {
  if (video.running) {
    return "pending";
  }
  if (video.rows.some((r) => r.status === "error")) {
    return "error";
  }
  const done = video.doneRows.length;
  if (done > 0 && done === video.checkedRows.length) {
    return "success";
  }
  return "default";
});
</script>

<GlassPanel
  dashed
  {status}
  glow={ui.dragHover && ui.activeTab === "video"}
  class="@container flex min-h-0 flex-col overflow-hidden"
>
  {#if video.rows.length === 0 && !video.adding}
    <VideoEmptyState />
  {:else}
    <ul class="custom-scrollbar min-h-0 flex-1 overflow-y-auto p-3">
      {#each video.rows as row (row.id)}
        <VideoRow {row} />
      {/each}
      {#if video.adding}
        <li class="flex items-center gap-3 px-3 py-4 text-sm text-white-text/60">
          <Icon name="spinner" class="size-5 animate-spin text-bid-gold" />
          Reading videos…
        </li>
      {/if}
    </ul>
  {/if}
</GlassPanel>
