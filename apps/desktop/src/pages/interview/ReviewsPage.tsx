import { useMemo, useState } from "react";
import { useSearchParams } from "react-router-dom";
import { ChevronRight, MessageSquareText, Mic2, Sparkles } from "lucide-react";
import { Badge, Card } from "../../components/ui";
import { useInterviewFlow, type InterviewSession } from "../../hooks/useInterviewFlow";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { hasLocalDatabase } from "../../services/applications";
import { parseDocument, transcribeAudio, type ProcessingJobResult } from "../../services/ai";

type Filter = "全部" | InterviewSession["type"];

export default function ReviewsPage() {
  const [params] = useSearchParams();
  const { applications, sessions } = useInterviewFlow();
  const [filter, setFilter] = useState<Filter>("全部");
  const [selectedId, setSelectedId] = useState(params.get("session") ?? sessions[0]?.id);
  const [processing, setProcessing] = useState(false);
  const [processingResult, setProcessingResult] = useState<ProcessingJobResult>();
  const [processingError, setProcessingError] = useState("");
  const filtered = useMemo(() => sessions.filter((item) => filter === "全部" || item.type === filter), [filter, sessions]);
  const selected = filtered.find((item) => item.id === selectedId) ?? filtered[0];
  const application = applications.find((item) => item.id === selected?.applicationId);
  const scored = selected?.questions.filter((item) => item.score !== undefined) ?? [];
  const averageScore = scored.length ? Math.round(scored.reduce((sum, item) => sum + (item.score ?? 0), 0) / scored.length) : undefined;

  const chooseAndProcess = async (kind: "document" | "audio") => {
    setProcessingError("");
    const path = await openDialog({ multiple: false, directory: false, filters: kind === "document" ? [{ name: "文档", extensions: ["pdf", "docx", "txt", "md"] }] : [{ name: "音频", extensions: ["mp3", "wav", "m4a", "webm", "mp4"] }] });
    if (!path) return;
    setProcessing(true);
    try {
      const result = kind === "document" ? await parseDocument(path, application?.id) : await transcribeAudio(path, application?.id);
      setProcessingResult(result);
    } catch (reason) { setProcessingError(String(reason)); } finally { setProcessing(false); }
  };

  return <div className="review-page-layout">
    <Card className="review-session-panel">
      <div className="review-filter">{(["全部", "真实面试", "模拟面试"] as Filter[]).map((value) => <button key={value} className={filter === value ? "active" : ""} onClick={() => setFilter(value)}>{value}<b>{value === "全部" ? sessions.length : sessions.filter((item) => item.type === value).length}</b></button>)}</div>
      <div className="session-list linked-sessions">{filtered.map((session) => {
        const related = applications.find((item) => item.id === session.applicationId);
        return <button className={selected?.id === session.id ? "active" : ""} key={session.id} onClick={() => setSelectedId(session.id)}><span className="company-logo">{related?.companyMark ?? "?"}</span><span><strong>{related?.company} · {session.round}</strong><small>{session.type} · {session.questions.length} 题 · {session.createdAt}</small></span><Badge tone={session.type === "真实面试" ? "blue" : "purple"}>{session.type}</Badge><ChevronRight size={16}/></button>;
      })}</div>
    </Card>

    <div className="review-record">
      <Card className="review-import-card"><div><strong>导入复盘材料</strong><p>PDF、DOCX 在本地提取文本；音频通过已配置的 ASR Provider 转写。</p></div><div><button className="button button--secondary" disabled={processing || !hasLocalDatabase} onClick={() => chooseAndProcess("document")}>解析文档</button><button className="button button--secondary" disabled={processing || !hasLocalDatabase} onClick={() => chooseAndProcess("audio")}><Mic2 size={15}/>{processing ? "处理中…" : "转写音频"}</button></div>{processingError && <p className="field-error">{processingError}</p>}{processingResult && <small>任务 {processingResult.id.slice(0, 8)} · {processingResult.status === "succeeded" ? "处理完成" : processingResult.status} · {processingResult.durationMs ?? 0} ms</small>}</Card>
      {selected && application ? <>
      <Card className="review-summary"><div><Badge tone={selected.type === "真实面试" ? "blue" : "purple"}>{selected.type}</Badge><h2>{application.company} · {application.role}</h2><p>{selected.round} · {selected.createdAt} · {selected.duration}</p></div><div className="review-score"><strong>{averageScore ?? "—"}</strong><span>平均分<small>{scored.length}/{selected.questions.length} 题已评价</small></span></div></Card>
      <div className="review-question-heading"><div><h3>逐题记录</h3><p>问题、回答、AI 评价与得分集中在同一处，便于回看上下文。</p></div><span>{selected.questions.length} 道题</span></div>
      <div className="review-question-list">{selected.questions.map((question, index) => <Card className="review-question-card" key={question.id}>
        <div className="review-question-title"><span>{index + 1}</span><div><Badge tone={question.source === "面经" ? "green" : question.source === "AI 简历题" ? "purple" : "blue"}>{question.source}</Badge><h3>{question.prompt}</h3></div>{question.score !== undefined && <strong className={`question-score ${question.score >= 80 ? "good" : question.score < 60 ? "weak" : ""}`}>{question.score}<small>/100</small></strong>}</div>
        <div className="answer-block"><div><MessageSquareText size={16}/><strong>回答</strong></div><p>{question.answer || "本题未作答"}</p></div>
        <div className="ai-evaluation"><div><Sparkles size={16}/><strong>AI 评价</strong></div><p>{question.evaluation ?? "完成本场面试后生成逐题评价与分数。"}</p></div>
      </Card>)}</div>
    </> : <Card><div className="interview-empty"><Mic2 size={32}/><h3>暂无面试记录</h3><p>完成模拟面试或导入真实面试后，会在这里生成逐题复盘。</p></div></Card>}</div>
  </div>;
}
