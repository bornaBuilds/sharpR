<script lang="ts">
let { value, label = "Progress" }: { value: number; label?: string } = $props();

const percent = $derived(Math.round(Math.min(1, Math.max(0, value)) * 100));
</script>

<div
  class="h-2 w-full overflow-hidden rounded-pill bg-white-text/10"
  role="progressbar"
  aria-label={label}
  aria-valuemin="0"
  aria-valuemax="100"
  aria-valuenow={percent}
>
  <div
    class="h-full rounded-pill bg-linear-to-r from-bid-gold-deep via-bid-gold to-bid-gold-deep transition-[width] duration-300"
    style={`width: ${percent}%`}
  ></div>
</div>
