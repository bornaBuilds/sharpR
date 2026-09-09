<script lang="ts" generics="T extends string">
import Icon from "$lib/icons/Icon.svelte";

let {
  value = $bindable(),
  options,
  label,
  disabled = false,
}: {
  value: T;
  options: { value: T; label: string }[];
  label: string;
  disabled?: boolean;
} = $props();

let open = $state(false);
let root: HTMLDivElement | undefined = $state();

const current = $derived(
  options.find((o) => o.value === value)?.label ?? String(value),
);

$effect(() => {
  if (!open) {
    return;
  }
  const onPointerDown = (event: PointerEvent) => {
    if (root && !root.contains(event.target as Node)) {
      open = false;
    }
  };
  const onKeyDown = (event: KeyboardEvent) => {
    if (event.key === "Escape") {
      open = false;
    }
  };
  document.addEventListener("pointerdown", onPointerDown);
  document.addEventListener("keydown", onKeyDown);
  return () => {
    document.removeEventListener("pointerdown", onPointerDown);
    document.removeEventListener("keydown", onKeyDown);
  };
});

function choose(next: T): void {
  value = next;
  open = false;
}
</script>

<div class="relative" bind:this={root}>
  <button
    type="button"
    class="btn-outline-pill min-w-28 justify-between uppercase tracking-wide"
    aria-haspopup="listbox"
    aria-expanded={open}
    aria-label={label}
    title={label}
    {disabled}
    onclick={() => {
      open = !open;
    }}
  >
    {current}
    <Icon
      name="chevron-down"
      class={open
        ? "size-4 rotate-180 transition-transform"
        : "size-4 transition-transform"}
    />
  </button>
  {#if open}
    <ul
      role="listbox"
      aria-label={label}
      class="absolute top-full left-0 z-30 mt-2 min-w-full overflow-hidden rounded-2xl border border-white-text/15 bg-bid-purple-deep/95 p-1 shadow-lg backdrop-blur"
    >
      {#each options as option (option.value)}
        <li>
          <button
            type="button"
            role="option"
            aria-selected={option.value === value}
            class={[
              "w-full rounded-xl px-4 py-2 text-left text-sm font-medium transition-colors",
              option.value === value
                ? "bg-bid-gold text-bid-purple"
                : "hover:bg-white-text/10",
            ]}
            onclick={() => choose(option.value)}
          >
            {option.label}
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>
