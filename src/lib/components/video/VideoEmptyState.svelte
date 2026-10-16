<script lang="ts">
import OutlinePillButton from "$lib/components/ui/OutlinePillButton.svelte";
import PillButton from "$lib/components/ui/PillButton.svelte";
import Icon from "$lib/icons/Icon.svelte";
import { video } from "$lib/stores/video.svelte";
</script>

<div class="flex h-full flex-col items-center justify-center gap-6 p-10 text-center">
  <Icon name="upload-cloud" class="size-20 text-bid-gold" />
  <div>
    <p class="text-2xl font-semibold">Drop videos or a folder here</p>
    <p class="mt-1 text-white-text/60">
      MP4, MOV, MKV, WebM, AVI and more are compressed locally to MP4 or WebM
    </p>
  </div>
  <div class="flex items-center gap-3">
    <PillButton onclick={() => video.pickVideos()}>
      <Icon name="film" class="size-5" />Select videos
    </PillButton>
    <OutlinePillButton onclick={() => video.pickFolder()}>
      <Icon name="folder" class="size-5" />Select folder
    </OutlinePillButton>
  </div>
  <p class="text-xs text-white-text/40">
    Powered by the bundled FFmpeg (GPL), run as a separate program
  </p>
</div>
