import { useEffect, useMemo, useState } from "react";
import { useSearchParams } from "react-router-dom";
import { ChevronRight, Clock3, FileText, MessageSquareText, Mic2, Pencil, RefreshCw, Sparkles, Trash2, X } from "lucide-react";
import { Badge, Card } from "../../components/ui";
import { useInterviewFlow, type InterviewSession } from "../../hooks/useInterviewFlow";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { hasLocalDatabase } from "../../services/applications";
import type { ProcessingJobSummary } from "../../services/ai";
import { requestAiSendConfirmation } from "../../services/settings";

type Filter = "全部" | InterviewSession["type"];
interface TextEditorState { jobId: string; fileName: string; text: string; loading: boolean; saving: boolean }
const MAX_MATERIAL_CHARACTERS = 60_000;
const sessionTime = (value: string) => {
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString("zh-CN", { dateStyle: "short", timeStyle: "short" });
};
const fileName = (path: string) => path.split(/[\\/]/).pop() || path;
const formatDuration = (milliseconds: number) => {
  const seconds = Math.max(0, Math.round(milliseconds / 1000));
  if (seconds < 60) return `${seconds} 秒`;
  const minutes = Math.floor(seconds / 60);
  const remainder = seconds % 60;
  return remainder ? `${minutes} 分 ${remainder} 秒` : `${minutes} 分钟`;
};
const jobLabel = (job: ProcessingJobSummary) => {
  if (job.status === "running") return "文件处理中";
  if (job.status === "failed") return "文件处理失败";
  if (job.importStatus === "running") return "AI 正在还原问答";
  if (job.importStatus === "failed") return "生成面试记录失败";
  if (job.importStatus === "succeeded" && !job.interviewSessionId) return "关联面试记录已删除";
  if (job.importStatus === "succeeded") return "已生成面试记录";
  return "文字已提取，等待确认";
};
const jobTone = (job: ProcessingJobSummary): "blue" | "green" | "red" | "orange" | "gray" => {
  if (job.status === "failed" || job.importStatus === "failed") return "red";
  if (job.importStatus === "succeeded" && job.interviewSessionId) return "green";
  if (job.importStatus === "succeeded") return "gray";
  if (job.status === "running" || job.importStatus === "running") return "blue";
  return "orange";
};

export default function ReviewsPage() {
  const [params] = useSearchParams();
  const {
    applications, sessions, reviewSession, deleteSession, importProcessingJob, processInterviewMaterial,
    processingJobs: jobs, processingJobsLoading: jobsLoading, processingJobsError: jobsError,
    processingJobsHasMore, processingRequestCount, refreshProcessingJobs, loadMoreProcessingJobs,
    getProcessingJobText, updateProcessingJobText, deleteProcessingJob,
  } = useInterviewFlow();
  const [filter, setFilter] = useState<Filter>("全部");
  const [selectedId, setSelectedId] = useState(params.get("session") ?? sessions[0]?.id);
  const [choosingFile, setChoosingFile] = useState(false);
  const [processingError, setProcessingError] = useState("");
  const [reviewing, setReviewing] = useState(false);
  const [deletingSession, setDeletingSession] = useState(false);
  const [importApplicationId, setImportApplicationId] = useState("");
  const [editor, setEditor] = useState<TextEditorState>();
  const [now, setNow] = useState(Date.now());
  const reviewApplications = applications;
  const reviewableSessions = useMemo(() => sessions.filter((item) => item.status !== "进行中"), [sessions]);
  const filtered = useMemo(() => reviewableSessions.filter((item) => filter === "全部" || item.type === filter), [filter, reviewableSessions]);
  const selected = filtered.find((item) => item.id === selectedId) ?? filtered[0];
  const application = applications.find((item) => item.id === selected?.applicationId);
  const scored = selected?.questions.filter((item) => item.score !== undefined) ?? [];
  const averageScore = scored.length ? Math.round(scored.reduce((sum, item) => sum + (item.score ?? 0), 0) / scored.length) : undefined;
  const hasRunningJob = jobs.some((job) => job.status === "running" || job.importStatus === "running");
  const busy = choosingFile || processingRequestCount > 0 || hasRunningJob;

  useEffect(() => {
    if (!busy) return;
    setNow(Date.now());
    const timer = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(timer);
  }, [busy]);

  const estimatedDuration = (job: ProcessingJobSummary) => {
    if (job.importStatus === "running") {
      const history = jobs
        .filter((item) => item.importStatus === "succeeded" && item.importStartedAt && item.importCompletedAt)
        .map((item) => new Date(item.importCompletedAt as string).getTime() - new Date(item.importStartedAt as string).getTime())
        .filter((value) => Number.isFinite(value) && value > 0)
        .sort((a, b) => a - b);
      return history[Math.floor(history.length / 2)] ?? 90_000;
    }
    const rateHistory = jobs
      .filter((item) => item.kind === job.kind && item.status === "succeeded" && item.durationMs && item.sourceSizeBytes)
      .map((item) => (item.durationMs as number) / (item.sourceSizeBytes as number))
      .filter((value) => Number.isFinite(value) && value > 0)
      .sort((a, b) => a - b);
    if (job.sourceSizeBytes && rateHistory.length) {
      return Math.max(1000, job.sourceSizeBytes * (rateHistory[Math.floor(rateHistory.length / 2)] as number));
    }
    return job.kind === "asr" ? 120_000 : 10_000;
  };

  const chooseAndProcess = async (kind: "document" | "audio") => {
    setProcessingError("");
    const targetApplicationId = reviewApplications.find((item) => item.id === importApplicationId)?.id
      ?? reviewApplications.find((item) => item.id === application?.id)?.id
      ?? reviewApplications[0]?.id;
    if (!targetApplicationId) {
      setProcessingError("请先创建投递记录，再导入真实面试材料。");
      return;
    }
    setChoosingFile(true);
    try {
      const path = await openDialog({
        multiple: false,
        directory: false,
        filters: kind === "document"
          ? [{ name: "文档", extensions: ["pdf", "docx", "txt", "md"] }]
          : [{ name: "音频", extensions: ["mp3", "wav", "m4a", "webm", "mp4"] }],
      });
      if (!path) return;
      await processInterviewMaterial(kind, path, targetApplicationId);
    } catch (reason) {
      setProcessingError(String(reason));
    } finally {
      setChoosingFile(false);
    }
  };

  const runImport = async (jobId: string) => {
    setProcessingError("");
    const confirmed = await requestAiSendConfirmation("将把你确认过的面试文字发给 AI 服务，帮你还原问答记录。是否继续？");
    if (!confirmed) return;
    try {
      const imported = await importProcessingJob(jobId, confirmed);
      setFilter("全部");
      setSelectedId(imported.id);
    } catch (reason) {
      setProcessingError(String(reason));
    }
  };

  const openEditor = async (job: ProcessingJobSummary) => {
    setEditor({ jobId: job.id, fileName: fileName(job.sourcePath), text: "", loading: true, saving: false });
    try {
      const text = await getProcessingJobText(job.id);
      setEditor({ jobId: job.id, fileName: fileName(job.sourcePath), text, loading: false, saving: false });
    } catch (reason) {
      setEditor(undefined);
      setProcessingError(String(reason));
    }
  };

  const saveEditor = async () => {
    if (!editor || editor.loading || editor.saving) return;
    setEditor({ ...editor, saving: true });
    try {
      await updateProcessingJobText(editor.jobId, editor.text);
      setEditor(undefined);
    } catch (reason) {
      setEditor({ ...editor, saving: false });
      setProcessingError(String(reason));
    }
  };

  const removeJob = async (job: ProcessingJobSummary) => {
    if (!window.confirm(`确定删除“${fileName(job.sourcePath)}”的处理记录和转写文字吗？${job.interviewSessionId ? "已经生成的面试记录会保留。" : ""}`)) return;
    try {
      await deleteProcessingJob(job.id);
    } catch (reason) {
      setProcessingError(String(reason));
    }
  };

  const viewSession = (id: string) => {
    setFilter("全部");
    setSelectedId(id);
  };

  const removeSession = async (session: InterviewSession) => {
    if (!window.confirm(`确定删除这场“${session.round}”面试记录吗？逐题回答和 AI 评价会一并删除，但原材料处理记录会保留。`)) return;
    setDeletingSession(true);
    try {
      await deleteSession(session.id);
      setSelectedId(undefined);
      await refreshProcessingJobs();
    } catch (reason) {
      setProcessingError(String(reason));
    } finally {
      setDeletingSession(false);
    }
  };

  return <div className="review-page-layout">
    <Card className="review-session-panel">
      <div className="review-filter">{(["全部", "真实面试", "模拟面试"] as Filter[]).map((value) => <button key={value} className={filter === value ? "active" : ""} onClick={() => setFilter(value)}>{value}<b>{value === "全部" ? reviewableSessions.length : reviewableSessions.filter((item) => item.type === value).length}</b></button>)}</div>
      <div className="session-list linked-sessions">{filtered.map((session) => {
        const related = applications.find((item) => item.id === session.applicationId);
        return <button className={selected?.id === session.id ? "active" : ""} key={session.id} onClick={() => setSelectedId(session.id)}><span className="company-logo">{related?.companyMark ?? "?"}</span><span><strong>{related?.company} · {session.round}</strong><small>{session.type} · {session.questions.length} 题 · {sessionTime(session.createdAt)}</small></span><Badge tone={session.type === "真实面试" ? "blue" : "purple"}>{session.type}</Badge><ChevronRight size={16}/></button>;
      })}</div>
    </Card>

    <div className="review-record">
      <Card className="review-import-card">
        <div><strong>第一步：提取面试文字</strong><p>解析完成后不会自动发送给 AI。请先检查、编辑文字，再决定是否生成面试记录。</p></div>
        <label><span>关联岗位（拒绝或已归档投递也可复盘）</span><select value={reviewApplications.find((item) => item.id === importApplicationId)?.id ?? reviewApplications.find((item) => item.id === application?.id)?.id ?? reviewApplications[0]?.id ?? ""} onChange={(event) => setImportApplicationId(event.target.value)}>{reviewApplications.map((item) => <option key={item.id} value={item.id}>{item.company} · {item.role}{item.archived ? " · 已归档" : ""}{item.stage.includes("拒绝") ? " · 已拒绝" : ""}</option>)}</select></label>
        <div><button className="button button--secondary" disabled={busy || !hasLocalDatabase || !reviewApplications.length} onClick={() => void chooseAndProcess("document")}><FileText size={15}/>解析文档</button><button className="button button--secondary" disabled={busy || !hasLocalDatabase || !reviewApplications.length} onClick={() => void chooseAndProcess("audio")}><Mic2 size={15}/>转写音频</button></div>
        {processingError && <p className="field-error">{processingError}</p>}
      </Card>

      <Card className="review-processing-card">
        <div className="review-processing-heading"><div><strong>第二步：确认文字并生成记录</strong><p>状态保存在本地；切换页面时可通过侧边栏查看运行数量，完成或失败会全局通知。</p></div><button className="text-button" disabled={jobsLoading} onClick={() => void refreshProcessingJobs()}><RefreshCw size={14}/>{jobsLoading ? "刷新中" : "刷新"}</button></div>
        {jobsError && <p className="field-error">{jobsError}</p>}
        {!jobsLoading && !jobs.length && <div className="review-processing-empty">还没有上传过面试材料</div>}
        <div className="review-processing-list">{jobs.map((job) => {
          const related = applications.find((item) => item.id === job.applicationId);
          const activeSince = job.importStatus === "running" ? job.importStartedAt : job.createdAt;
          const elapsed = Math.max(0, now - new Date(activeSince ?? job.createdAt).getTime());
          const estimate = estimatedDuration(job);
          const remaining = Math.max(0, estimate - elapsed);
          const isRunning = job.status === "running" || job.importStatus === "running";
          const error = job.errorMessage ?? job.importErrorMessage;
          const editable = job.status === "succeeded" && !["running", "succeeded"].includes(job.importStatus);
          return <div className="review-processing-item" key={job.id}>
            <div className={`processing-kind ${job.kind === "asr" ? "is-audio" : ""}`}>{job.kind === "asr" ? <Mic2 size={17}/> : <FileText size={17}/>}</div>
            <div className="processing-detail">
              <div><strong title={job.sourcePath}>{fileName(job.sourcePath)}</strong><Badge tone={jobTone(job)}>{jobLabel(job)}</Badge></div>
              <small>{related ? `${related.company} · ${related.role}` : "关联投递已删除"} · {sessionTime(job.createdAt)}{job.characterCount ? ` · ${job.characterCount.toLocaleString()} 字` : ""}</small>
              {isRunning && <p className="processing-progress"><span><i/></span><Clock3 size={13}/>已用时 {formatDuration(elapsed)} · {remaining > 0 ? `按同类任务估算还需约 ${formatDuration(remaining)}` : "已超过历史估算，服务仍在处理"}</p>}
              {error && <p className="field-error">{error}</p>}
              {job.textPreview && job.status === "succeeded" && <p className="processing-preview">{job.textPreview}{job.characterCount && job.characterCount > 240 ? "…" : ""}</p>}
            </div>
            <div className="processing-actions">
              {editable && <button className="text-button" onClick={() => void openEditor(job)}><Pencil size={13}/>查看/编辑文字</button>}
              {editable && <button className="button button--secondary" disabled={busy} onClick={() => void runImport(job.id)}>{job.importStatus === "failed" ? "重试生成" : "生成面试记录"}</button>}
              {job.importStatus === "succeeded" && job.interviewSessionId && <button className="text-button" onClick={() => viewSession(job.interviewSessionId as string)}>查看记录<ChevronRight size={14}/></button>}
              {!isRunning && <button className="icon-button danger-text" title="删除处理记录" onClick={() => void removeJob(job)}><Trash2 size={14}/></button>}
            </div>
          </div>;
        })}</div>
        {processingJobsHasMore && <button className="review-processing-more" onClick={loadMoreProcessingJobs}>加载更早的处理记录</button>}
      </Card>

      {selected && application ? <>
        <Card className="review-summary"><div><Badge tone={selected.type === "真实面试" ? "blue" : "purple"}>{selected.type}</Badge><h2>{application.company} · {application.role}</h2><p>{selected.round} · {sessionTime(selected.createdAt)} · {selected.duration}</p>{selected.reviewSummary && <small className="review-overall-summary">{selected.reviewSummary}</small>}</div><div className="review-summary-actions"><div className="review-score"><strong>{averageScore ?? "—"}</strong><span>平均分<small>{scored.length}/{selected.questions.length} 题已评价</small></span></div>{selected.status !== "进行中" && <button className="button button--primary" disabled={reviewing || deletingSession} onClick={async () => { setReviewing(true); setProcessingError(""); try { const confirmed = await requestAiSendConfirmation("将把本场问题与回答发给 AI 服务进行逐题评价。是否继续？"); if (confirmed) await reviewSession(selected.id, confirmed); } catch (reason) { setProcessingError(String(reason)); } finally { setReviewing(false); } }}><Sparkles size={14}/>{reviewing ? "复盘生成中…" : selected.status === "复盘完成" ? "重新生成复盘" : "生成 AI 复盘"}</button>}<button className="icon-button danger-text" title="删除面试记录" disabled={deletingSession || reviewing} onClick={() => void removeSession(selected)}><Trash2 size={15}/></button></div></Card>
        <div className="review-question-heading"><div><h3>逐题记录</h3><p>问题和回答放在一起，方便回看每道题的表现</p></div><span>{selected.questions.length} 道题</span></div>
        <div className="review-question-list">{selected.questions.map((question, index) => <Card className="review-question-card" key={question.id}>
          <div className="review-question-title"><span>{index + 1}</span><div><Badge tone={question.source === "面经" ? "green" : question.source === "AI 简历题" ? "purple" : "blue"}>{question.source}</Badge><h3>{question.prompt}</h3></div>{question.score !== undefined && <strong className={`question-score ${question.score >= 80 ? "good" : question.score < 60 ? "weak" : ""}`}>{question.score}<small>/100</small></strong>}</div>
          <div className="answer-block"><div><MessageSquareText size={16}/><strong>回答</strong></div><p>{question.answer || "本题未作答"}</p></div>
          <div className="ai-evaluation"><div><Sparkles size={16}/><strong>AI 评价</strong></div><p>{question.evaluation ?? (selected.status === "进行中" ? "本场尚未完成。" : "点击上方“生成 AI 复盘”，根据真实回答生成评价与分数。")}</p></div>
        </Card>)}</div>
      </> : <Card><div className="interview-empty"><Mic2 size={32}/><h3>暂无面试记录</h3><p>完成模拟面试，或从上方已确认的材料生成真实面试记录后，会在这里展示逐题内容。</p></div></Card>}
    </div>

    {editor && <div className="modal-backdrop" onMouseDown={() => !editor.saving && setEditor(undefined)}><div className="material-editor-modal" onMouseDown={(event) => event.stopPropagation()}>
      <div className="material-editor-head"><div><strong>检查面试文字</strong><small>{editor.fileName}</small></div><button className="icon-button" disabled={editor.saving} onClick={() => setEditor(undefined)}><X size={16}/></button></div>
      {editor.loading ? <div className="material-editor-loading">正在读取完整文字…</div> : <><textarea value={editor.text} maxLength={MAX_MATERIAL_CHARACTERS} onChange={(event) => setEditor({ ...editor, text: event.target.value })}/><div className="material-editor-footer"><span className={editor.text.length >= MAX_MATERIAL_CHARACTERS ? "danger-text" : ""}>{editor.text.length.toLocaleString()} / {MAX_MATERIAL_CHARACTERS.toLocaleString()} 字</span><div><button className="button button--secondary" disabled={editor.saving} onClick={() => setEditor(undefined)}>取消</button><button className="button button--primary" disabled={editor.saving || !editor.text.trim()} onClick={() => void saveEditor()}>{editor.saving ? "保存中…" : "保存文字"}</button></div></div></>}
    </div></div>}
  </div>;
}
