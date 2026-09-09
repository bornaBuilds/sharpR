<script lang="ts">
import type { Snippet } from "svelte";
import type { HTMLButtonAttributes } from "svelte/elements";

let {
  class: className = "",
  children,
  ...rest
}: HTMLButtonAttributes & { children: Snippet } = $props();
</script>

<button type="button" class={`btn-outline-pill ${className}`} {...rest}>
  {@render children()}
</button>
