<script lang="ts">
import type { HTMLInputAttributes } from "svelte/elements";

let {
  checked = $bindable(false),
  label,
  class: className = "",
  ...rest
}: HTMLInputAttributes & { checked?: boolean; label: string } = $props();
</script>

<input
  type="checkbox"
  class={`size-5 shrink-0 cursor-pointer rounded accent-bid-gold ${className}`}
  aria-label={label}
  title={label}
  bind:checked
  {...rest}
/>
