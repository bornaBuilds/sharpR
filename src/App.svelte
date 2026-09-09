<script lang="ts">
import ConvertTab from "$lib/components/convert/ConvertTab.svelte";
import CropTab from "$lib/components/crop/CropTab.svelte";
import AppHeader from "$lib/components/layout/AppHeader.svelte";
import DropOverlay from "$lib/components/layout/DropOverlay.svelte";
import Toasts from "$lib/components/layout/Toasts.svelte";
import { converter } from "$lib/stores/converter.svelte";
import { crop } from "$lib/stores/crop.svelte";
import { toasts } from "$lib/stores/toasts.svelte";
import { ui } from "$lib/stores/ui.svelte";
import { registerDragDrop } from "$lib/tauri/drag-drop";
import { isSupportedPath } from "$lib/utils/paths";

function handleDrop(paths: string[]): void {
  if (ui.activeTab === "convert") {
    void converter.addPaths(paths);
    return;
  }
  const image = paths.find(isSupportedPath);
  if (!image) {
    toasts.push("Drop a single image file to crop it (folders go to Convert)");
    return;
  }
  if (paths.length > 1) {
    toasts.push("Only the first image was opened in the crop editor");
  }
  void crop.open({ kind: "path", path: image });
}

$effect(() => {
  let unlisten: (() => void) | undefined;
  let disposed = false;
  registerDragDrop({
    onHover: () => {
      ui.dragHover = true;
    },
    onLeave: () => {
      ui.dragHover = false;
    },
    onDrop: handleDrop,
  })
    .then((fn) => {
      if (disposed) {
        fn();
      } else {
        unlisten = fn;
      }
    })
    .catch(() => {
      // Not running inside Tauri (browser preview): drops are unavailable.
    });
  return () => {
    disposed = true;
    unlisten?.();
  };
});
</script>

<div class="flex h-full flex-col">
  <AppHeader />
  <main class="relative min-h-0 flex-1 px-6 pb-6">
    <div class="h-full" hidden={ui.activeTab !== "convert"}>
      <ConvertTab />
    </div>
    <div class="h-full" hidden={ui.activeTab !== "crop"}>
      <CropTab />
    </div>
  </main>
  <DropOverlay />
  <Toasts />
</div>
