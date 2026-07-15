import { useMemo, useState } from "react";
import { useSearchParams } from "react-router-dom";
import { ChevronRight, MessageSquareText, Mic2, Sparkles } from "lucide-react";
import { Badge, Card } from "../../components/ui";
import { useInterviewFlow, type InterviewSession } from "../../hooks/useInterviewFlow";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { hasLocalDatabase } from "../../services/applications";
import { parseDocument, transcribeAudio, type ProcessingJobResult } from "../../services/ai";

type Filter = "全部" | InterviewSession["type"];
const sessionTime = (value: string) => {
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString("zh-CN", { dateStyle: "short", timeStyle: "short" });
};

export default function ReviewsPage() {
  const [params] = useSearchParams();
  const { applications, sessions, reviewSession, importTranscript } = useInterviewFlow();
  const [filter, setFilter] = useState<Filter>("全部");
  const [selectedId, setSelectedId] = useState(params.get("session") ?? sessions[0]?.id);
  const [processing, setProcessing] = useState(false);
  const [processingResult, setProcessingResult] = useState<ProcessingJobResult>();
  const [processingError, setProcessingError] = useState("");
  const [reviewing, setReviewing] = useState(false);
  const [importApplicationId, setImportApplicationId] = useState("");
  const reviewableSessions = useMemo(() => sessions.filter((item) => item.status !== "进行中"), [sessions]);
  const filtered = useMemo(() => reviewableSessions.filter((item) => filter === "全部" || item.type === filter), [filter, reviewableSessions]);
  const selected = filtered.find((item) => item.id === selectedId) ?? filtered[0];
  const application = applications.find((item) => item.id === selected?.applicationId);
  const scored = selected?.questions.filter((item) => item.score !== undefined) ?? [];
  const averageScore = scored.length ? Math.round(scored.reduce((sum, item) => sum + (item.score ?? 0), 0) / scored.length) : undefined;

  const extractProcessingText = (result: ProcessingJobResult) => {
    const payload = result.result;
    if (typeof payload?.text === "string") return payload.text;
    const transcript = payload?.transcript;
    if (typeof transcript === "string") return transcript;
    if (transcript && typeof transcript === "object") {
      const value = transcript as { text?: unknown; segments?: Array<{ text?: unknown }> };
      if (typeof value.text === "string") return value.text;
      if (Array.isArray(value.segments)) return value.segments.map((item) => typeof item.text === "string" ? item.text : "").filter(Boolean).join("\n");
    }
    return "";
  };

  const chooseAndProcess = async (kind: "document" | "audio") => {
    setProcessingError("");
    const targetApplicationId = importApplicationId || application?.id || applications[0]?.id;
    if (!targetApplicationId) { setProcessingError("请先创建投递记录，再导入真实面试材料。"); return; }
    const path = await openDialog({ multiple: false, directory: false, filters: kind === "document" ? [{ name: "文档", extensions: ["pdf", "docx", "txt", "md"] }] : [{ name: "音频", extensions: ["mp3", "wav", "m4a", "webm", "mp4"] }] });
    if (!path) return;
    setProcessing(true);
    try {
      const result = kind === "document" ? await parseDocument(path, targetApplicationId) : await transcribeAudio(path, targetApplicationId);
      setProcessingResult(result);
      const text = extractProcessingText(result);
      if (!text.trim()) throw new Error("材料处理完成，但没有得到可用于整理问答的文字");
      if (!window.confirm("将把提取出的面试文字发送给已配置的 AI 服务，用于还原真实问答记录。是否继续？")) return;
      const imported = await importTranscript(targetApplicationId, text);
      setSelectedId(imported.id);
    } catch (reason) { setProcessingError(String(reason)); } finally { setProcessing(false); }
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
      <Card className="review-import-card"><div><strong>导入真实面试材料</strong><p>先在本地解析文档或通过 ASR 转写音频，再由 AI 忠实还原问题与回答，生成一条真实面试会话。</p></div><label><span>关联岗位</span><select value={importApplicationId || application?.id || applications[0]?.id || ""} onChange={(event) => setImportApplicationId(event.target.value)}>{applications.filter((item) => !item.archived).map((item) => <option key={item.id} value={item.id}>{item.company} · {item.role}</option>)}</select></label><div><button className="button button--secondary" disabled={processing || !hasLocalDatabase} onClick={() => chooseAndProcess("document")}>解析文档并导入</button><button className="button button--secondary" disabled={processing || !hasLocalDatabase} onClick={() => chooseAndProcess("audio")}><Mic2 size={15}/>{processing ? "处理中…" : "转写音频并导入"}</button></div>{processingError && <p className="field-error">{processingError}</p>}{processingResult && <small>处理任务 {processingResult.id.slice(0, 8)} · {processingResult.status === "succeeded" ? "文字提取完成" : processingResult.status} · {processingResult.durationMs ?? 0} ms</small>}</Card>
      {selected && application ? <>
      <Card className="review-summary"><div><Badge tone={selected.type === "真实面试" ? "blue" : "purple"}>{selected.type}</Badge><h2>{application.company} · {application.role}</h2><p>{selected.round} · {sessionTime(selected.createdAt)} · {selected.duration}</p>{selected.reviewSummary && <small className="review-overall-summary">{selected.reviewSummary}</small>}</div><div className="review-summary-actions"><div className="review-score"><strong>{averageScore ?? "—"}</strong><span>平均分<small>{scored.length}/{selected.questions.length} 题已评价</small></span></div>{selected.status !== "进行中" && <button className="button button--primary" disabled={reviewing} onClick={async () => { if (hasLocalDatabase && !window.confirm("将把本场问题与回答发送给已配置的 AI 服务进行逐题评价。是否继续？")) return; setReviewing(true); setProcessingError(""); try { await reviewSession(selected.id); } catch (reason) { setProcessingError(String(reason)); } finally { setReviewing(false); } }}><Sparkles size={14}/>{reviewing ? "复盘生成中…" : selected.status === "复盘完成" ? "重新生成复盘" : "生成 AI 复盘"}</button>}</div></Card>
      <div className="review-question-heading"><div><h3>逐题记录</h3><p>问题、回答、AI 评价与得分集中在同一处，便于回看上下文。</p></div><span>{selected.questions.length} 道题</span></div>
      <div className="review-question-list">{selected.questions.map((question, index) => <Card className="review-question-card" key={question.id}>
        <div className="review-question-title"><span>{index + 1}</span><div><Badge tone={question.source === "面经" ? "green" : question.source === "AI 简历题" ? "purple" : "blue"}>{question.source}</Badge><h3>{question.prompt}</h3></div>{question.score !== undefined && <strong className={`question-score ${question.score >= 80 ? "good" : question.score < 60 ? "weak" : ""}`}>{question.score}<small>/100</small></strong>}</div>
        <div className="answer-block"><div><MessageSquareText size={16}/><strong>回答</strong></div><p>{question.answer || "本题未作答"}</p></div>
        <div className="ai-evaluation"><div><Sparkles size={16}/><strong>AI 评价</strong></div><p>{question.evaluation ?? (selected.status === "进行中" ? "本场尚未完成。" : "点击上方“生成 AI 复盘”，根据真实回答生成评价与分数。")}</p></div>
      </Card>)}</div>
    </> : <Card><div className="interview-empty"><Mic2 size={32}/><h3>暂无面试记录</h3><p>完成模拟面试或导入真实面试后，会在这里生成逐题复盘。</p></div></Card>}</div>
  </div>;
}
