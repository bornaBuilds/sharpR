<script lang="ts">
import type { Snippet } from "svelte";
import type { HTMLButtonAttributes } from "svelte/elements";

type Variant = "default" | "error" | "success";

let {
  variant = "default",
  class: className = "",
  children,
  ...rest
}: HTMLButtonAttributes & { variant?: Variant; children: Snippet } = $props();
</script>

<button
  type="button"
  class={`btn-pill ${className}`}
  data-variant={variant}
  {...rest}
>
  {@render children()}
</button>
