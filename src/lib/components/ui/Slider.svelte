<script lang="ts">
let {
  value = $bindable(0),
  min = 0,
  max = 100,
  step = 1,
  label,
  disabled = false,
}: {
  value?: number;
  min?: number;
  max?: number;
  step?: number;
  label: string;
  disabled?: boolean;
} = $props();
</script>

<label
  class={[
    "inline-flex items-center gap-3 text-sm",
    disabled && "cursor-not-allowed opacity-40",
  ]}
>
  <span class="text-white-text/70">{label}</span>
  <input
    type="range"
    class="w-32 cursor-pointer"
    aria-label={label}
    bind:value
    {min}
    {max}
    {step}
    {disabled}
  />
  <span class="w-8 text-right tabular-nums">{value}</span>
</label>
