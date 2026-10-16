import { mount } from "svelte";
import App from "./App.svelte";
import "./app.css";
import { converter } from "$lib/stores/converter.svelte";
import { crop } from "$lib/stores/crop.svelte";
import { posters } from "$lib/stores/posters.svelte";
import { ui } from "$lib/stores/ui.svelte";
import { video } from "$lib/stores/video.svelte";

const target = document.getElementById("app");
if (!target) {
  throw new Error("Root element #app not found");
}

if (import.meta.env.DEV) {
  // Debug hook for the browser preview / devtools console.
  Object.assign(window, { sharpr: { converter, crop, posters, ui, video } });
}

export default mount(App, { target });
