<script lang="ts">
import ProgressBar from "$lib/components/ui/ProgressBar.svelte";
import Icon from "$lib/icons/Icon.svelte";
import { formatBytes, formatPercent, percentChange } from "$lib/utils/format";
import type { VideoRow } from "$lib/stores/video.svelte";

let { row }: { row: VideoRow } = $props();

const change = $derived(
  row.result ? percentChange(row.sizeBytes, row.result.sizeBytes) : 0,
);
</script>

<div
  class="flex w-32 shrink-0 items-center justify-end gap-2 text-sm @4xl:w-44"
>
  {#if row.status === "queued"}
    <span class="text-xs text-white-text/50">Queued</span>
  {:else if row.status === "compressing"}
    {#if row.fraction === null}
      <Icon name="spinner" class="size-5 animate-spin text-bid-gold" />
      <span class="text-xs text-white-text/60">Compressing…</span>
    {:else}
      <div class="w-16 @4xl:w-24">
        <ProgressBar value={row.fraction} label={`Compressing ${row.name}`} />
      </div>
      <span class="w-9 text-right text-xs tabular-nums">
        {Math.round(row.fraction * 100)}%
      </span>
    {/if}
  {:else if row.status === "finishing"}
    <Icon name="spinner" class="size-5 animate-spin text-bid-gold" />
    <span class="text-xs text-white-text/60">Finishing…</span>
  {:else if row.status === "done" && row.result}
    <span class="tabular-nums">{formatBytes(row.result.sizeBytes)}</span>
    {#if row.result.keptOriginal}
      <span
        class="truncate rounded-pill bg-bid-gold/15 px-2 py-0.5 text-xs font-semibold text-bid-gold"
        title="Re-encoding saved less than 5%, so the original video was kept (metadata stripped)"
      >
        <span class="hidden @4xl:inline">Already optimized</span>
        <span class="@4xl:hidden">Optimized</span>
      </span>
    {:else}
      <span
        class={[
          "rounded-pill px-2 py-0.5 text-xs font-semibold tabular-nums",
          change <= 0
            ? "bg-status-success/15 text-status-success"
            : "bg-status-error/15 text-status-error",
        ]}
      >
        {formatPercent(change)}
      </span>
      <Icon name="success" class="hidden size-5 text-status-success @4xl:block" />
    {/if}
  {:else if row.status === "error"}
    <span class="truncate text-xs text-status-error" title={row.error ?? ""}>
      {row.error ?? "Failed"}
    </span>
    <Icon name="error" class="size-5 shrink-0 text-status-error" />
  {:else if row.status === "cancelled"}
    <span class="text-xs text-white-text/50">Cancelled</span>
  {/if}
</div>
