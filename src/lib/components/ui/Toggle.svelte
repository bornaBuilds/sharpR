<script lang="ts">
let {
  checked = $bindable(false),
  label,
  disabled = false,
}: { checked?: boolean; label: string; disabled?: boolean } = $props();
</script>

<label
  class={[
    "inline-flex select-none items-center gap-2 text-sm",
    disabled ? "cursor-not-allowed opacity-40" : "cursor-pointer",
  ]}
>
  <button
    type="button"
    role="switch"
    aria-checked={checked}
    aria-label={label}
    {disabled}
    class={[
      "relative h-6 w-11 shrink-0 rounded-pill transition-colors duration-200",
      checked ? "bg-bid-gold" : "bg-white-text/20",
    ]}
    onclick={() => {
      checked = !checked;
    }}
  >
    <span
      class={[
        "absolute top-1 left-1 size-4 rounded-full transition-transform duration-200",
        checked ? "translate-x-5 bg-bid-purple" : "bg-white-text",
      ]}
    ></span>
  </button>
  <span>{label}</span>
</label>
