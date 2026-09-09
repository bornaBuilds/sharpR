<script lang="ts">
import { ICON_PATHS, type IconName } from "./paths";
import type { SVGAttributes } from "svelte/elements";

let {
  name,
  class: className = "size-5",
  ...rest
}: SVGAttributes<SVGSVGElement> & { name: IconName } = $props();
</script>

<svg
  xmlns="http://www.w3.org/2000/svg"
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="1.75"
  stroke-linecap="round"
  stroke-linejoin="round"
  class={className}
  aria-hidden="true"
  {...rest}
>
  {#each ICON_PATHS[name] as d (d)}
    <path {d} />
  {/each}
</svg>
