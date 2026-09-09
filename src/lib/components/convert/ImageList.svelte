<script lang="ts">
import GlassPanel from "$lib/components/layout/GlassPanel.svelte";
import Icon from "$lib/icons/Icon.svelte";
import { converter } from "$lib/stores/converter.svelte";
import { ui } from "$lib/stores/ui.svelte";
import EmptyState from "./EmptyState.svelte";
import ImageRow from "./ImageRow.svelte";

type PanelStatus = "default" | "pending" | "success" | "error";

const status = $derived.by((): PanelStatus => {
  if (converter.running) {
    return "pending";
  }
  if (converter.rows.some((r) => r.status === "error")) {
    return "error";
  }
  const done = converter.doneRows.length;
  if (done > 0 && done === converter.checkedRows.length) {
    return "success";
  }
  return "default";
});
</script>

<GlassPanel
  dashed
  {status}
  glow={ui.dragHover && ui.activeTab === "convert"}
  class="flex min-h-0 flex-1 flex-col overflow-hidden"
>
  {#if converter.rows.length === 0 && !converter.adding}
    <EmptyState />
  {:else}
    <ul class="custom-scrollbar min-h-0 flex-1 overflow-y-auto p-3">
      {#each converter.rows as row (row.id)}
        <ImageRow {row} />
      {/each}
      {#if converter.adding}
        <li class="flex items-center gap-3 px-3 py-4 text-sm text-white-text/60">
          <Icon name="spinner" class="size-5 animate-spin text-bid-gold" />
          Reading images…
        </li>
      {/if}
    </ul>
  {/if}
</GlassPanel>
