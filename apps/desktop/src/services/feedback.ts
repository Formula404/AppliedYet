export type FeedbackKind = "success" | "error" | "info" | "warning";

export interface FeedbackOptions {
  title?: string;
  message: string;
  kind?: FeedbackKind;
  confirmLabel?: string;
}

export interface ConfirmationOptions {
  title: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  kind?: "warning" | "danger" | "info";
}

export interface FeedbackRequest extends FeedbackOptions {
  id: string;
  type: "feedback";
}

export interface ConfirmationRequest extends ConfirmationOptions {
  id: string;
  type: "confirmation";
  resolve: (confirmed: boolean) => void;
}

export type FeedbackCenterRequest = FeedbackRequest | ConfirmationRequest;

const FEEDBACK_EVENT = "applied-yet:feedback";
let requestSequence = 0;
let lastFeedbackKey = "";
let lastFeedbackAt = 0;

const defaultTitles: Record<FeedbackKind, string> = {
  success: "操作成功",
  error: "操作失败",
  info: "提示",
  warning: "请注意",
};

const nextRequestId = () => `feedback-${Date.now()}-${++requestSequence}`;

export function showFeedback(options: FeedbackOptions | string, kind: FeedbackKind = "info", _legacyDuration?: number) {
  const normalized = typeof options === "string" ? { message: options, kind } : options;
  const message = normalized.message.trim();
  if (!message) return;
  const resolvedKind = normalized.kind ?? "info";
  const feedbackKey = `${resolvedKind}:${message}`;
  const now = Date.now();
  if (feedbackKey === lastFeedbackKey && now - lastFeedbackAt < 800) return;
  lastFeedbackKey = feedbackKey;
  lastFeedbackAt = now;
  const request: FeedbackRequest = {
    ...normalized,
    id: nextRequestId(),
    type: "feedback",
    kind: resolvedKind,
    title: normalized.title?.trim() || defaultTitles[resolvedKind],
    message,
  };
  window.dispatchEvent(new CustomEvent<FeedbackCenterRequest>(FEEDBACK_EVENT, { detail: request }));
}

export function showSuccess(message: string, title = "操作成功") {
  showFeedback({ title, message, kind: "success" });
}

export function showError(reason: unknown, title = "操作失败") {
  showFeedback({ title, message: String(reason), kind: "error" });
}

export function showInfo(message: string, title = "提示") {
  showFeedback({ title, message, kind: "info" });
}

export function requestConfirmation(options: ConfirmationOptions | string) {
  const normalized: ConfirmationOptions = typeof options === "string"
    ? { title: "确认操作", message: options }
    : options;
  return new Promise<boolean>((resolve) => {
    const request: ConfirmationRequest = {
      ...normalized,
      id: nextRequestId(),
      type: "confirmation",
      kind: normalized.kind ?? "warning",
      confirmLabel: normalized.confirmLabel ?? "确认",
      cancelLabel: normalized.cancelLabel ?? "取消",
      resolve,
    };
    window.dispatchEvent(new CustomEvent<FeedbackCenterRequest>(FEEDBACK_EVENT, { detail: request }));
  });
}

export function subscribeFeedback(listener: (request: FeedbackCenterRequest) => void) {
  const handler = (event: Event) => listener((event as CustomEvent<FeedbackCenterRequest>).detail);
  window.addEventListener(FEEDBACK_EVENT, handler);
  return () => window.removeEventListener(FEEDBACK_EVENT, handler);
}
