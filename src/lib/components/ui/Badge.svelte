<script lang="ts">
import Icon from "$lib/icons/Icon.svelte";
import type { Snippet } from "svelte";
import type { IconName } from "$lib/icons/paths";

type Kind = "info" | "warning" | "success" | "error";

let {
  kind = "info",
  icon,
  children,
}: { kind?: Kind; icon?: IconName; children: Snippet } = $props();

const STYLES: Record<Kind, string> = {
  info: "bg-bid-gold/15 text-bid-gold",
  warning: "bg-status-pending/15 text-status-pending",
  success: "bg-status-success/15 text-status-success",
  error: "bg-status-error/15 text-status-error",
};
</script>

<span
  class={`inline-flex items-center gap-1.5 rounded-pill px-2.5 py-1 text-xs font-medium ${STYLES[kind]}`}
>
  {#if icon}
    <Icon name={icon} class="size-3.5" />
  {/if}
  {@render children()}
</span>
