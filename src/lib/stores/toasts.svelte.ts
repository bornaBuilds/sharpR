type ToastKind = "info" | "success" | "error";

interface Toast {
  id: number;
  message: string;
  kind: ToastKind;
}

const DEFAULT_TTL_MS = 4000;

class ToastStore {
  list = $state<Toast[]>([]);
  #seq = 0;

  push(message: string, kind: ToastKind = "info", ttl = DEFAULT_TTL_MS): void {
    this.#seq += 1;
    const id = this.#seq;
    this.list.push({ id, message, kind });
    setTimeout(() => this.dismiss(id), ttl);
  }

  dismiss(id: number): void {
    this.list = this.list.filter((t) => t.id !== id);
  }
}

const toasts = new ToastStore();

export { toasts };
export type { Toast, ToastKind };
