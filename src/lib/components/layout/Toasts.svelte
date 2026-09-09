<script lang="ts">
import Icon from "$lib/icons/Icon.svelte";
import { type ToastKind, toasts } from "$lib/stores/toasts.svelte";
import type { IconName } from "$lib/icons/paths";

const ICONS: Record<ToastKind, IconName> = {
  info: "info",
  success: "success",
  error: "error",
};
const COLORS: Record<ToastKind, string> = {
  info: "text-bid-gold",
  success: "text-status-success",
  error: "text-status-error",
};
</script>

<div
  class="pointer-events-none fixed inset-x-0 bottom-6 z-50 flex flex-col items-center gap-2"
>
  {#each toasts.list as toast (toast.id)}
    <button
      type="button"
      class="pointer-events-auto flex max-w-xl items-center gap-3 rounded-pill border border-white-text/15 bg-bid-purple-deep/90 px-5 py-3 text-sm shadow-lg backdrop-blur"
      onclick={() => toasts.dismiss(toast.id)}
    >
      <Icon
        name={ICONS[toast.kind]}
        class={`size-5 shrink-0 ${COLORS[toast.kind]}`}
      />
      <span>{toast.message}</span>
    </button>
  {/each}
</div>
