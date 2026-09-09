<script lang="ts">
import GlassPanel from "$lib/components/layout/GlassPanel.svelte";
import IconButton from "$lib/components/ui/IconButton.svelte";
import NumberField from "$lib/components/ui/NumberField.svelte";
import OutlinePillButton from "$lib/components/ui/OutlinePillButton.svelte";
import { isIdentityParams, MAX_SKEW_DEG } from "$lib/crop/transform";
import Icon from "$lib/icons/Icon.svelte";
import { crop } from "$lib/stores/crop.svelte";

const ROTATE_STEP = 45;
const SKEW_STEP = 5;

const disabled = $derived(!crop.source || crop.loading);
const zoomPercent = $derived(Math.round(crop.zoom * 100));
</script>

<GlassPanel class="flex flex-wrap items-center gap-1 px-4 py-2">
  <IconButton
    icon="zoom-out"
    label="Zoom out"
    onclick={() => crop.zoomOut()}
    disabled={disabled || crop.zoom <= 1}
  />
  <button
    type="button"
    class="w-12 text-center text-xs text-white-text/70 tabular-nums hover:text-white-text"
    title="Reset zoom"
    onclick={() => crop.resetZoom()}
    {disabled}
  >
    {zoomPercent}%
  </button>
  <IconButton
    icon="zoom-in"
    label="Zoom in"
    onclick={() => crop.zoomIn()}
    {disabled}
  />

  <div class="mx-2 h-6 w-px bg-white-text/15"></div>

  <IconButton
    icon="rotate-left"
    label="Rotate 45° counter-clockwise"
    onclick={() => crop.rotateBy(-ROTATE_STEP)}
    {disabled}
  />
  <NumberField
    label=""
    suffix="°"
    class="w-16"
    min={-180}
    max={180}
    value={crop.params.rotationDeg}
    oncommit={(deg) => crop.setRotation(deg)}
    {disabled}
  />
  <IconButton
    icon="rotate-right"
    label="Rotate 45° clockwise"
    onclick={() => crop.rotateBy(ROTATE_STEP)}
    {disabled}
  />

  <div class="mx-2 h-6 w-px bg-white-text/15"></div>

  <span class="flex items-center gap-1" title="Skew horizontally">
    <Icon name="skew-x" class="size-5 text-white-text/60" />
    <IconButton
      icon="minus"
      label="Skew X −5°"
      iconClass="size-4"
      onclick={() => crop.skewXBy(-SKEW_STEP)}
      disabled={disabled || crop.params.skewXDeg <= -MAX_SKEW_DEG}
    />
    <span class="w-9 text-center text-xs tabular-nums">{crop.params.skewXDeg}°</span>
    <IconButton
      icon="plus"
      label="Skew X +5°"
      iconClass="size-4"
      onclick={() => crop.skewXBy(SKEW_STEP)}
      disabled={disabled || crop.params.skewXDeg >= MAX_SKEW_DEG}
    />
  </span>
  <span class="flex items-center gap-1" title="Skew vertically">
    <Icon name="skew-y" class="size-5 text-white-text/60" />
    <IconButton
      icon="minus"
      label="Skew Y −5°"
      iconClass="size-4"
      onclick={() => crop.skewYBy(-SKEW_STEP)}
      disabled={disabled || crop.params.skewYDeg <= -MAX_SKEW_DEG}
    />
    <span class="w-9 text-center text-xs tabular-nums">{crop.params.skewYDeg}°</span>
    <IconButton
      icon="plus"
      label="Skew Y +5°"
      iconClass="size-4"
      onclick={() => crop.skewYBy(SKEW_STEP)}
      disabled={disabled || crop.params.skewYDeg >= MAX_SKEW_DEG}
    />
  </span>

  <div class="mx-2 h-6 w-px bg-white-text/15"></div>

  <IconButton
    icon="flip-horizontal"
    label="Flip horizontally"
    active={crop.params.flipX}
    onclick={() => crop.toggleFlipX()}
    {disabled}
  />
  <IconButton
    icon="flip-vertical"
    label="Flip vertically"
    active={crop.params.flipY}
    onclick={() => crop.toggleFlipY()}
    {disabled}
  />

  <div class="mx-2 h-6 w-px bg-white-text/15"></div>

  <IconButton
    icon="reset"
    label="Reset transforms"
    onclick={() => crop.resetTransform()}
    disabled={disabled || isIdentityParams(crop.params)}
  />

  <div class="flex-1"></div>

  <OutlinePillButton
    class="px-4 py-2 text-sm"
    onclick={() => crop.pickImage()}
    disabled={crop.loading}
  >
    <Icon name="images" class="size-4" />Open image
  </OutlinePillButton>
</GlassPanel>
