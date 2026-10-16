<script lang="ts">
import Checkbox from "$lib/components/ui/Checkbox.svelte";
import IconButton from "$lib/components/ui/IconButton.svelte";
import OutlinePillButton from "$lib/components/ui/OutlinePillButton.svelte";
import Icon from "$lib/icons/Icon.svelte";
import { posters } from "$lib/stores/posters.svelte";
import { type VideoRow, video } from "$lib/stores/video.svelte";
import { formatBytes, formatDuration } from "$lib/utils/format";
import { codecLabel } from "$lib/video/helpers";
import VideoRowStatus from "./VideoRowStatus.svelte";

let { row }: { row: VideoRow } = $props();

const details = $derived(
  [
    `${row.width} × ${row.height}`,
    codecLabel(row.videoCodec),
    row.hdr ? "HDR" : null,
    row.hasAudio ? null : "No audio",
  ]
    .filter(Boolean)
    .join(" · "),
);
const postersOpen = $derived(posters.rowId === row.id);
const canShowPosters = $derived(
  row.status === "done" && (row.result?.posterCount ?? 0) > 0,
);
</script>

<li
  class={[
    "group flex items-center gap-3 rounded-2xl px-3 py-2 transition-colors @4xl:gap-4",
    row.checked ? "hover:bg-white-text/5" : "opacity-50 hover:opacity-80",
    postersOpen && "bg-white-text/5",
  ]}
>
  <Checkbox
    label={`Include ${row.name}`}
    checked={row.checked}
    disabled={video.running}
    onchange={(e) => video.setChecked(row.id, e.currentTarget.checked)}
  />
  <div
    class="relative flex h-14 w-20 shrink-0 items-center justify-center overflow-hidden rounded-lg bg-bid-purple-deep/70 @4xl:h-16 @4xl:w-24"
  >
    <img
      src={row.thumbDataUrl}
      alt=""
      class="max-h-full max-w-full object-contain"
      draggable="false"
    />
    <span
      class="absolute right-1 bottom-1 rounded bg-bid-purple-deep/85 px-1 text-[10px] font-semibold tabular-nums"
    >
      {formatDuration(row.durationSec)}
    </span>
  </div>
  <div class="min-w-0 flex-1">
    <p class="truncate font-medium" title={row.path}>{row.name}</p>
    <p class="truncate text-xs text-white-text/50">{details}</p>
  </div>
  <div
    class="hidden w-20 shrink-0 text-right text-sm text-white-text/70 tabular-nums @4xl:block"
  >
    {formatBytes(row.sizeBytes)}
  </div>
  <VideoRowStatus {row} />
  <div class="flex shrink-0 items-center justify-end gap-1 @4xl:w-44">
    {#if row.status === "done"}
      <OutlinePillButton
        class="px-3 py-1.5 text-sm @xl:px-4"
        title={`Save ${row.name}`}
        onclick={() => video.saveOne(row.id)}
      >
        <Icon name="download" class="size-4" /><span class="hidden @xl:inline"
          >Save</span
        >
      </OutlinePillButton>
    {/if}
    {#if canShowPosters}
      <IconButton
        icon="images"
        label="Poster frames"
        active={postersOpen}
        onclick={() => posters.toggle(row.id, row.name)}
      />
    {/if}
    <IconButton
      icon="trash"
      label="Remove from list"
      danger
      disabled={video.running}
      class="opacity-0 transition-opacity group-hover:opacity-100 focus-visible:opacity-100"
      onclick={() => video.remove(row.id)}
    />
  </div>
</li>
