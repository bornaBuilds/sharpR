<script lang="ts">
import type { Snippet } from "svelte";

type Status = "default" | "pending" | "success" | "error";

let {
  status = "default",
  dashed = false,
  glow = false,
  class: className = "",
  children,
}: {
  status?: Status;
  dashed?: boolean;
  glow?: boolean;
  class?: string;
  children: Snippet;
} = $props();

const GLOW: Record<Status, string> = {
  default: "shadow-glow",
  pending: "shadow-glow-pending",
  success: "shadow-glow-success",
  error: "shadow-glow-error",
};
</script>

<div
  class={`glass-panel ${glow ? GLOW[status] : ""} ${className}`}
  data-status={status}
  data-dashed={dashed}
>
  <svg class="panel-border" aria-hidden="true">
    <rect
      x="1.5"
      y="1.5"
      rx="30"
      ry="30"
      style="width: calc(100% - 3px); height: calc(100% - 3px)"
    />
  </svg>
  {@render children()}
</div>
