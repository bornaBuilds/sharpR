<script lang="ts">
import pkg from "../../../../package.json";
import TabBar from "./TabBar.svelte";
</script>

<header class="grid grid-cols-[1fr_auto_1fr] items-center px-6 py-4">
  <div class="text-2xl font-extrabold tracking-tight">
    <span class="text-white-text">sharp</span><span class="text-bid-gold"
      >R</span
    >
  </div>
  <TabBar />
  <div class="text-right text-xs text-white-text/40">v{pkg.version}</div>
</header>
