<script lang="ts">
import Icon from "$lib/icons/Icon.svelte";
import { formatBytes, formatPercent, percentChange } from "$lib/utils/format";
import type { ConverterRow } from "$lib/stores/converter.svelte";

let { row }: { row: ConverterRow } = $props();

const change = $derived(
  row.result ? percentChange(row.sizeBytes, row.result.sizeBytes) : 0,
);
</script>

<div class="flex w-44 shrink-0 items-center justify-end gap-2 text-sm">
  {#if row.status === "queued"}
    <span class="text-xs text-white-text/50">Queued</span>
  {:else if row.status === "converting"}
    <Icon name="spinner" class="size-5 animate-spin text-bid-gold" />
  {:else if row.status === "done" && row.result}
    <span class="tabular-nums">{formatBytes(row.result.sizeBytes)}</span>
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
    <Icon name="success" class="size-5 text-status-success" />
  {:else if row.status === "error"}
    <span class="truncate text-xs text-status-error" title={row.error ?? ""}>
      {row.error ?? "Failed"}
    </span>
    <Icon name="error" class="size-5 shrink-0 text-status-error" />
  {:else if row.status === "cancelled"}
    <span class="text-xs text-white-text/50">Cancelled</span>
  {/if}
</div>
