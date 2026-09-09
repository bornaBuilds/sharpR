<script lang="ts">
import Checkbox from "$lib/components/ui/Checkbox.svelte";
import IconButton from "$lib/components/ui/IconButton.svelte";
import OutlinePillButton from "$lib/components/ui/OutlinePillButton.svelte";
import Icon from "$lib/icons/Icon.svelte";
import { type ConverterRow, converter } from "$lib/stores/converter.svelte";
import { crop } from "$lib/stores/crop.svelte";
import { formatBytes } from "$lib/utils/format";
import RowStatus from "./RowStatus.svelte";

let { row }: { row: ConverterRow } = $props();
</script>

<li
  class={[
    "group flex items-center gap-4 rounded-2xl px-3 py-2 transition-colors",
    row.checked ? "hover:bg-white-text/5" : "opacity-50 hover:opacity-80",
  ]}
>
  <Checkbox
    label={`Include ${row.name}`}
    checked={row.checked}
    disabled={converter.running}
    onchange={(e) => converter.setChecked(row.id, e.currentTarget.checked)}
  />
  <div
    class="flex h-16 w-24 shrink-0 items-center justify-center overflow-hidden rounded-lg bg-bid-purple-deep/70"
  >
    <img
      src={row.thumbDataUrl}
      alt=""
      class="max-h-full max-w-full object-contain"
      draggable="false"
    />
  </div>
  <div class="min-w-0 flex-1">
    <p class="truncate font-medium" title={row.path ?? row.name}>{row.name}</p>
    <p class="text-xs text-white-text/50">{row.width} × {row.height}</p>
  </div>
  <div class="w-20 shrink-0 text-right text-sm text-white-text/70 tabular-nums">
    {formatBytes(row.sizeBytes)}
  </div>
  <RowStatus {row} />
  <div class="flex w-44 shrink-0 items-center justify-end gap-1">
    {#if row.status === "done"}
      <OutlinePillButton
        class="px-4 py-1.5 text-sm"
        onclick={() => converter.saveOne(row.id)}
      >
        <Icon name="download" class="size-4" />Save
      </OutlinePillButton>
    {/if}
    <IconButton
      icon="crop"
      label="Open in crop"
      disabled={converter.running || crop.loading}
      class="opacity-0 transition-opacity group-hover:opacity-100 focus-visible:opacity-100"
      onclick={() => crop.open({ kind: "item", id: row.id })}
    />
    <IconButton
      icon="trash"
      label="Remove from list"
      danger
      disabled={converter.running}
      class="opacity-0 transition-opacity group-hover:opacity-100 focus-visible:opacity-100"
      onclick={() => converter.remove(row.id)}
    />
  </div>
</li>
