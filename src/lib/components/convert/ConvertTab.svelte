<script lang="ts">
import ConvertFooter from "./ConvertFooter.svelte";
import ConvertToolbar from "./ConvertToolbar.svelte";
import ImageList from "./ImageList.svelte";
</script>

<div class="flex h-full flex-col gap-4">
  <ConvertToolbar />
  <ImageList />
  <ConvertFooter />
</div>
