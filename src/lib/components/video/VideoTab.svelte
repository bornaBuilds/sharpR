<script lang="ts">
import { posters } from "$lib/stores/posters.svelte";
import PosterSidebar from "./PosterSidebar.svelte";
import VideoFooter from "./VideoFooter.svelte";
import VideoList from "./VideoList.svelte";
import VideoToolbar from "./VideoToolbar.svelte";
</script>

<div class="flex h-full flex-col gap-4">
  <VideoToolbar />
  <div
    class={[
      "grid min-h-0 flex-1 grid-rows-[minmax(0,1fr)] gap-4",
      posters.isOpen
        ? "grid-cols-[minmax(0,1fr)_300px] xl:grid-cols-[minmax(0,1fr)_340px]"
        : "grid-cols-1",
    ]}
  >
    <VideoList />
    {#if posters.isOpen}
      <PosterSidebar />
    {/if}
  </div>
  <VideoFooter />
</div>
