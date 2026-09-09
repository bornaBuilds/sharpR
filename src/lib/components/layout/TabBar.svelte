<script lang="ts">
import { type Tab, ui } from "$lib/stores/ui.svelte";

const TABS: { id: Tab; label: string }[] = [
  { id: "convert", label: "Convert" },
  { id: "crop", label: "Crop & resize" },
];
</script>

<nav
  class="flex items-center gap-1 rounded-pill border border-white-text/15 p-1"
  aria-label="Sections"
>
  {#each TABS as tab (tab.id)}
    {@const active = ui.activeTab === tab.id}
    <button
      type="button"
      class={[
        "rounded-pill px-5 py-2 text-sm font-semibold transition-colors duration-200",
        active
          ? "bg-bid-gold text-bid-purple"
          : "text-white-text hover:bg-white-text/10",
      ]}
      aria-current={active ? "page" : undefined}
      onclick={() => {
        ui.activeTab = tab.id;
      }}
    >
      {tab.label}
    </button>
  {/each}
</nav>
