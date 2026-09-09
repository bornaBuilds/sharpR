<script lang="ts">
import OutlinePillButton from "$lib/components/ui/OutlinePillButton.svelte";
import PillButton from "$lib/components/ui/PillButton.svelte";
import Icon from "$lib/icons/Icon.svelte";
import { converter } from "$lib/stores/converter.svelte";
</script>

<div class="flex h-full flex-col items-center justify-center gap-6 p-10 text-center">
  <Icon name="upload-cloud" class="size-20 text-bid-gold" />
  <div>
    <p class="text-2xl font-semibold">Drop images or a folder here</p>
    <p class="mt-1 text-white-text/60">
      JPEG, PNG, WebP, GIF, BMP and TIFF are picked up automatically
    </p>
  </div>
  <div class="flex items-center gap-3">
    <PillButton onclick={() => converter.pickImages()}>
      <Icon name="image-plus" class="size-5" />Select images
    </PillButton>
    <OutlinePillButton onclick={() => converter.pickFolder()}>
      <Icon name="folder" class="size-5" />Select folder
    </OutlinePillButton>
  </div>
</div>
