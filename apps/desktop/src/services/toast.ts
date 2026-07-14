export type ToastKind = "success" | "error" | "info";

export interface ToastPayload {
  message: string;
  kind?: ToastKind;
  duration?: number;
}

const TOAST_EVENT = "applied-yet:toast";

export function showToast(message: string, kind: ToastKind = "success", duration = 3200) {
  const normalizedMessage = message.trim();
  if (!normalizedMessage) return;
  window.dispatchEvent(new CustomEvent<ToastPayload>(TOAST_EVENT, { detail: { message: normalizedMessage, kind, duration } }));
}

export function subscribeToast(listener: (payload: ToastPayload) => void) {
  const handler = (event: Event) => listener((event as CustomEvent<ToastPayload>).detail);
  window.addEventListener(TOAST_EVENT, handler);
  return () => window.removeEventListener(TOAST_EVENT, handler);
}
