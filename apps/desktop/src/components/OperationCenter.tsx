import { useEffect, useMemo, useRef, useState } from "react";
import { CheckCircle2, ChevronDown, CircleAlert, FileCheck2, LoaderCircle } from "lucide-react";
import gsap from "gsap";
import { useGSAP } from "@gsap/react";
import type { ProcessingJobSummary } from "../services/ai";
import { subscribeOperations, type OperationItem } from "../services/operations";

gsap.registerPlugin(useGSAP);

interface OperationCenterProps {
  processingJobs: ProcessingJobSummary[];
  processingRequestCount: number;
  onOpenProcessing: () => void;
}

const elapsedText = (startedAt: number) => {
  const seconds = Math.max(0, Math.round((Date.now() - startedAt) / 1000));
  if (seconds < 60) return `${seconds} 秒`;
  return `${Math.floor(seconds / 60)} 分 ${seconds % 60} 秒`;
};

export default function OperationCenter({
  processingJobs,
  processingRequestCount,
  onOpenProcessing,
}: OperationCenterProps) {
  const [operations, setOperations] = useState<OperationItem[]>([]);
  const [open, setOpen] = useState(false);
  const [, setClock] = useState(0);
  const rootRef = useRef<HTMLDivElement>(null);
  const requestStartedAt = useRef(Date.now());
  const previousRequestCount = useRef(processingRequestCount);

  useEffect(() => subscribeOperations(setOperations), []);
  useEffect(() => {
    if (processingRequestCount > 0 && previousRequestCount.current === 0) {
      requestStartedAt.current = Date.now();
    }
    previousRequestCount.current = processingRequestCount;
  }, [processingRequestCount]);
  useEffect(() => {
    if (!open) return;
    const close = (event: MouseEvent) => {
      if (!rootRef.current?.contains(event.target as Node)) setOpen(false);
    };
    window.addEventListener("mousedown", close);
    return () => window.removeEventListener("mousedown", close);
  }, [open]);
  useEffect(() => {
    if (!operations.some((item) => item.status === "running")) return;
    const timer = window.setInterval(() => setClock((value) => value + 1), 1000);
    return () => window.clearInterval(timer);
  }, [operations]);

  const externalOperations = useMemo<OperationItem[]>(() => processingJobs
    .filter((job) => job.status === "running" || job.importStatus === "running")
    .map((job) => ({
      id: `processing-${job.id}`,
      label: job.importStatus === "running"
        ? "AI 正在生成面试记录"
        : job.kind === "asr" ? "正在解析面试音频" : "正在解析面试文档",
      detail: job.progressMessage ?? (job.importStatus === "running" ? "正在还原问答与面试结构" : "材料处理仍在后台进行"),
      status: "running",
      startedAt: new Date(job.importStartedAt ?? job.createdAt).getTime(),
    })), [processingJobs]);

  const hiddenRequestCount = Math.max(0, processingRequestCount - externalOperations.length);
  const items = hiddenRequestCount > 0
    ? [...externalOperations, ...operations, {
      id: "processing-request",
      label: "正在准备面试材料",
      detail: `${hiddenRequestCount} 项请求正在启动`,
      status: "running" as const,
      startedAt: requestStartedAt.current,
    }]
    : [...externalOperations, ...operations];
  const runningCount = items.filter((item) => item.status === "running").length;
  const latest = items[0];

  useGSAP(() => {
    if (!open) return;
    const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    gsap.fromTo(
      ".operation-panel",
      { autoAlpha: 0, y: reduceMotion ? 0 : -8, scale: reduceMotion ? 1 : 0.98 },
      { autoAlpha: 1, y: 0, scale: 1, duration: reduceMotion ? 0 : 0.22, ease: "power2.out" },
    );
    gsap.from(".operation-item", {
      autoAlpha: 0,
      y: reduceMotion ? 0 : -5,
      stagger: reduceMotion ? 0 : 0.035,
      duration: reduceMotion ? 0 : 0.18,
      ease: "power2.out",
    });
  }, { scope: rootRef, dependencies: [open], revertOnUpdate: true });

  useEffect(() => {
    if (items.length === 0) setOpen(false);
  }, [items.length]);

  if (!latest) return null;

  return (
    <div className="operation-center" ref={rootRef}>
      <button
        type="button"
        className={`processing-global-status ${runningCount ? "is-running" : "is-finished"}`}
        onClick={() => setOpen((value) => !value)}
        aria-expanded={open}
      >
        {runningCount ? <span /> : latest.status === "error" ? <CircleAlert size={14} /> : <CheckCircle2 size={14} />}
        <FileCheck2 size={15} />
        <strong>{runningCount ? `${runningCount} 项正在执行` : latest.label}</strong>
        <ChevronDown className={open ? "is-open" : ""} size={13} />
      </button>
      {open && (
        <div className="operation-panel">
          <div className="operation-panel-head">
            <span><strong>执行状态</strong><small>{runningCount ? `${runningCount} 项任务正在后台执行` : "最近的任务状态"}</small></span>
            {externalOperations.length > 0 && <button type="button" onClick={onOpenProcessing}>查看面试材料</button>}
          </div>
          <div className="operation-list">
            {items.map((item) => (
              <div className={`operation-item operation-item--${item.status}`} key={item.id}>
                <span className="operation-item-icon">
                  {item.status === "running" ? <LoaderCircle className="spin" size={16} /> : item.status === "success" ? <CheckCircle2 size={16} /> : <CircleAlert size={16} />}
                </span>
                <span>
                  <strong>{item.label}</strong>
                  <small>{item.detail ?? (item.status === "running" ? "正在处理，请稍候" : item.status === "success" ? "已完成" : "执行失败")}</small>
                </span>
                <time>{item.status === "running" ? elapsedText(item.startedAt) : item.status === "success" ? "完成" : "失败"}</time>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
