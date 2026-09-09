type Tab = "convert" | "crop";

class UiStore {
  activeTab = $state<Tab>("convert");
  dragHover = $state(false);
}

const ui = new UiStore();

export { ui };
export type { Tab };
