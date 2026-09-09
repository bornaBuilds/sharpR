<script lang="ts">
import GlassPanel from "$lib/components/layout/GlassPanel.svelte";
import OutlinePillButton from "$lib/components/ui/OutlinePillButton.svelte";
import PillButton from "$lib/components/ui/PillButton.svelte";
import ProgressBar from "$lib/components/ui/ProgressBar.svelte";
import Icon from "$lib/icons/Icon.svelte";
import { converter } from "$lib/stores/converter.svelte";
import { formatBytes } from "$lib/utils/format";

const originalTotal = $derived(
  converter.checkedRows.reduce((sum, r) => sum + r.sizeBytes, 0),
);
const convertedTotal = $derived(
  converter.doneRows.reduce((sum, r) => sum + (r.result?.sizeBytes ?? 0), 0),
);
const showProgress = $derived(converter.running || converter.batch.total > 0);
</script>

<GlassPanel class="flex items-center gap-4 px-6 py-3">
  <div class="min-w-0 flex-1">
    {#if showProgress}
      <ProgressBar value={converter.progress} label="Conversion progress" />
    {/if}
    <p class="mt-1 truncate text-xs text-white-text/60">
      {converter.checkedRows.length} of {converter.rows.length} selected ·
      {formatBytes(originalTotal)} original
      {#if converter.doneRows.length > 0}
        · {converter.doneRows.length} converted ({formatBytes(convertedTotal)})
      {/if}
      {#if showProgress}
        · {converter.batch.finished}/{converter.batch.total} processed
      {/if}
    </p>
  </div>
  <OutlinePillButton
    disabled={converter.doneRows.length === 0 || converter.running}
    onclick={() => converter.saveZip()}
  >
    <Icon name="archive" class="size-5" />Save all as ZIP
  </OutlinePillButton>
  <OutlinePillButton
    disabled={converter.doneRows.length === 0 || converter.running}
    onclick={() => converter.saveToFolder()}
  >
    <Icon name="folder-down" class="size-5" />Save all to folder
  </OutlinePillButton>
  {#if converter.running}
    <PillButton variant="error" onclick={() => converter.cancel()}>
      <Icon name="close" class="size-5" />Cancel
    </PillButton>
  {:else}
    <PillButton
      disabled={converter.checkedRows.length === 0 || converter.adding}
      onclick={() => converter.convert()}
    >
      <Icon name="check" class="size-5" />Convert ({converter.checkedRows.length})
    </PillButton>
  {/if}
</GlassPanel>
