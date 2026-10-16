<script lang="ts">
import GlassPanel from "$lib/components/layout/GlassPanel.svelte";
import OutlinePillButton from "$lib/components/ui/OutlinePillButton.svelte";
import PillButton from "$lib/components/ui/PillButton.svelte";
import ProgressBar from "$lib/components/ui/ProgressBar.svelte";
import Icon from "$lib/icons/Icon.svelte";
import { video } from "$lib/stores/video.svelte";
import { formatBytes, formatPercent, percentChange } from "$lib/utils/format";

const originalTotal = $derived(
  video.checkedRows.reduce((sum, r) => sum + r.sizeBytes, 0),
);
const doneOriginal = $derived(
  video.doneRows.reduce((sum, r) => sum + r.sizeBytes, 0),
);
const compressedTotal = $derived(
  video.doneRows.reduce((sum, r) => sum + (r.result?.sizeBytes ?? 0), 0),
);
const showProgress = $derived(video.running || video.batch.total > 0);
</script>

<GlassPanel class="flex items-center gap-4 px-6 py-3">
  <div class="min-w-0 flex-1">
    {#if showProgress}
      <ProgressBar value={video.progress} label="Compression progress" />
    {/if}
    <p class="mt-1 truncate text-xs text-white-text/60">
      {video.checkedRows.length} of {video.rows.length} selected ·
      {formatBytes(originalTotal)} original
      {#if video.doneRows.length > 0}
        · {video.doneRows.length} compressed ({formatBytes(compressedTotal)},
        {formatPercent(percentChange(doneOriginal, compressedTotal))})
      {/if}
      {#if showProgress}
        · {video.batch.finished}/{video.batch.total} processed
      {/if}
    </p>
  </div>
  <OutlinePillButton
    disabled={video.doneRows.length === 0 || video.running}
    onclick={() => video.saveToFolder()}
  >
    <Icon name="folder-down" class="size-5" />Save all to folder
  </OutlinePillButton>
  {#if video.running}
    <PillButton variant="error" onclick={() => video.cancel()}>
      <Icon name="close" class="size-5" />Cancel
    </PillButton>
  {:else}
    <PillButton
      disabled={video.checkedRows.length === 0 || video.adding}
      onclick={() => video.compress()}
    >
      <Icon name="check" class="size-5" />Compress ({video.checkedRows.length})
    </PillButton>
  {/if}
</GlassPanel>
