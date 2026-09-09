<script lang="ts">
let {
  value = $bindable(0),
  min,
  max,
  step = 1,
  label,
  suffix = "",
  disabled = false,
  class: className = "w-24",
  oncommit,
}: {
  value?: number;
  min?: number;
  max?: number;
  step?: number;
  label: string;
  suffix?: string;
  disabled?: boolean;
  class?: string;
  oncommit?: (value: number) => void;
} = $props();

function clamp(next: number): number {
  let out = Number.isFinite(next) ? next : (min ?? 0);
  if (min !== undefined) {
    out = Math.max(min, out);
  }
  if (max !== undefined) {
    out = Math.min(max, out);
  }
  return out;
}

function commit(): void {
  value = clamp(value);
  oncommit?.(value);
}
</script>

<label
  class={[
    "inline-flex items-center gap-2 text-sm",
    disabled && "cursor-not-allowed opacity-40",
  ]}
>
  {#if label}
    <span class="text-white-text/70">{label}</span>
  {/if}
  <input
    type="number"
    class={`field text-right tabular-nums ${className}`}
    aria-label={label || suffix}
    bind:value
    {min}
    {max}
    {step}
    {disabled}
    onchange={commit}
    onkeydown={(e) => {
      if (e.key === "Enter") {
        commit();
      }
    }}
  />
  {#if suffix}
    <span class="text-white-text/50">{suffix}</span>
  {/if}
</label>
