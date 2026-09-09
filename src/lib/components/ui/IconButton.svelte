<script lang="ts">
import Icon from "$lib/icons/Icon.svelte";
import type { HTMLButtonAttributes } from "svelte/elements";
import type { IconName } from "$lib/icons/paths";

let {
  icon,
  label,
  active = false,
  danger = false,
  iconClass = "size-5",
  class: className = "",
  ...rest
}: HTMLButtonAttributes & {
  icon: IconName;
  label: string;
  active?: boolean;
  danger?: boolean;
  iconClass?: string;
} = $props();
</script>

<button
  type="button"
  class={`btn-icon ${className}`}
  data-active={active}
  data-danger={danger}
  aria-label={label}
  title={label}
  {...rest}
>
  <Icon name={icon} class={iconClass} />
</button>
