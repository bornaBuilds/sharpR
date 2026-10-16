<script lang="ts">
import FormatControls from "$lib/components/convert/FormatControls.svelte";
import GlassPanel from "$lib/components/layout/GlassPanel.svelte";
import IconButton from "$lib/components/ui/IconButton.svelte";
import PillButton from "$lib/components/ui/PillButton.svelte";
import Icon from "$lib/icons/Icon.svelte";
import { posters } from "$lib/stores/posters.svelte";

function timeLabel(seconds: number): string {
  return `${seconds.toFixed(1)} s`;
}
</script>

<GlassPanel
  class="custom-scrollbar flex min-h-0 flex-col gap-5 overflow-y-auto px-5 py-5"
>
  <div class="flex items-start gap-2">
    <div class="min-w-0 flex-1">
      <h2 class="text-xs font-semibold text-white-text/50 uppercase tracking-wider">
        Poster frames
      </h2>
      <p class="mt-1 truncate font-medium" title={posters.rowName}>
        {posters.rowName}
      </p>
    </div>
    <IconButton
      icon="close"
      label="Close poster frames"
      onclick={() => posters.close()}
    />
  </div>

  {#if posters.loading}
    <div class="flex items-center gap-3 text-sm text-white-text/60">
      <Icon name="spinner" class="size-5 animate-spin text-bid-gold" />
      Loading frames…
    </div>
  {:else if posters.current}
    {@const current = posters.current}
    <figure>
      <div
        class="flex aspect-video items-center justify-center overflow-hidden rounded-xl bg-bid-purple-deep/70"
      >
        <img
          src={current.previewDataUrl}
          alt="Selected poster frame"
          class="max-h-full max-w-full object-contain"
          draggable="false"
        />
      </div>
      <figcaption class="mt-2 text-xs text-white-text/50 tabular-nums">
        {timeLabel(current.timeSec)} · {current.width} × {current.height}
      </figcaption>
    </figure>

    <div class="grid grid-cols-4 gap-2">
      {#each posters.frames as frame, i (frame.index)}
        {@const selected = i === posters.selected}
        <button
          type="button"
          aria-pressed={selected}
          aria-label={`Frame at ${timeLabel(frame.timeSec)}`}
          title={frame.dark
            ? `${timeLabel(frame.timeSec)} (dark frame)`
            : timeLabel(frame.timeSec)}
          class={[
            "flex aspect-square items-center justify-center overflow-hidden rounded-lg bg-bid-purple-deep/70 ring-2 transition",
            selected ? "ring-bid-gold" : "ring-transparent hover:ring-white-text/30",
            frame.dark && !selected && "opacity-50",
          ]}
          onclick={() => posters.select(i)}
        >
          <img
            src={frame.previewDataUrl}
            alt=""
            class="max-h-full max-w-full object-contain"
            draggable="false"
          />
        </button>
      {/each}
    </div>

    <section>
      <h2
        class="mb-3 text-xs font-semibold text-white-text/50 uppercase tracking-wider"
      >
        Image format
      </h2>
      <FormatControls target="poster" disabled={posters.saving} />
    </section>

    <PillButton
      class="w-full"
      disabled={posters.saving}
      onclick={() => posters.save()}
    >
      {#if posters.saving}
        <Icon name="spinner" class="size-5 animate-spin" />Saving…
      {:else}
        <Icon name="download" class="size-5" />Save poster
      {/if}
    </PillButton>
  {:else}
    <p class="text-sm text-white-text/60">
      No frames could be extracted from this video.
    </p>
  {/if}
</GlassPanel>
