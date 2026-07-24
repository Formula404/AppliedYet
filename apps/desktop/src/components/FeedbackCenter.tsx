import { useEffect, useRef, useState } from "react";
import { AlertTriangle, CheckCircle2, Info, ShieldAlert, X, XCircle } from "lucide-react";
import gsap from "gsap";
import { useGSAP } from "@gsap/react";
import { subscribeFeedback, type FeedbackCenterRequest } from "../services/feedback";

gsap.registerPlugin(useGSAP);

const iconForRequest = (request: FeedbackCenterRequest) => {
  if (request.type === "confirmation" && request.kind === "danger") return <ShieldAlert size={27} />;
  if (request.kind === "success") return <CheckCircle2 size={27} />;
  if (request.kind === "error") return <XCircle size={27} />;
  if (request.kind === "info") return <Info size={27} />;
  return <AlertTriangle size={27} />;
};

export default function FeedbackCenter() {
  const [queue, setQueue] = useState<FeedbackCenterRequest[]>([]);
  const backdropRef = useRef<HTMLDivElement>(null);
  const current = queue[0];

  useEffect(() => subscribeFeedback((request) => {
    setQueue((items) => [...items, request]);
  }), []);

  const close = (confirmed = false) => {
    setQueue((items) => {
      const active = items[0];
      if (active?.type === "confirmation") active.resolve(confirmed);
      return items.slice(1);
    });
  };

  useEffect(() => {
    if (!current) return;
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") close(false);
      if (event.key === "Enter" && current.type === "confirmation") close(true);
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [current]);

  useGSAP(() => {
    if (!current || !backdropRef.current) return;
    const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    gsap.fromTo(
      backdropRef.current,
      { autoAlpha: 0 },
      { autoAlpha: 1, duration: reduceMotion ? 0 : 0.18, ease: "power2.out" },
    );
    gsap.fromTo(
      ".global-feedback-dialog",
      { autoAlpha: 0, y: reduceMotion ? 0 : 18, scale: reduceMotion ? 1 : 0.96 },
      { autoAlpha: 1, y: 0, scale: 1, duration: reduceMotion ? 0 : 0.3, ease: "back.out(1.35)" },
    );
  }, { scope: backdropRef, dependencies: [current?.id], revertOnUpdate: true });

  if (!current) return null;
  const isConfirmation = current.type === "confirmation";
  const visualKind = isConfirmation && current.kind === "danger" ? "error" : current.kind;

  return (
    <div
      ref={backdropRef}
      className="modal-backdrop status-modal-backdrop global-feedback-backdrop"
      onMouseDown={() => close(false)}
    >
      <div
        className={`dialog status-dialog global-feedback-dialog status-dialog--${visualKind}`}
        role={isConfirmation ? "alertdialog" : "dialog"}
        aria-modal="true"
        aria-labelledby={`feedback-title-${current.id}`}
        aria-describedby={`feedback-message-${current.id}`}
        onMouseDown={(event) => event.stopPropagation()}
      >
        {!isConfirmation && (
          <button type="button" className="status-dialog-close" onClick={() => close(false)} aria-label="关闭">
            <X size={18} />
          </button>
        )}
        <span className="status-dialog-icon">{iconForRequest(current)}</span>
        <h2 id={`feedback-title-${current.id}`}>{current.title}</h2>
        <p id={`feedback-message-${current.id}`}>{current.message}</p>
        {queue.length > 1 && <small className="feedback-queue-count">还有 {queue.length - 1} 条消息</small>}
        {isConfirmation ? (
          <div className="confirm-dialog-actions">
            <button type="button" className="button button--secondary" onClick={() => close(false)}>
              {current.cancelLabel}
            </button>
            <button
              type="button"
              className={`button ${current.kind === "danger" ? "button--danger" : "button--primary"}`}
              onClick={() => close(true)}
              autoFocus
            >
              {current.confirmLabel}
            </button>
          </div>
        ) : (
          <button type="button" className="button button--primary" onClick={() => close(false)} autoFocus>
            {current.confirmLabel ?? "知道了"}
          </button>
        )}
      </div>
    </div>
  );
}
