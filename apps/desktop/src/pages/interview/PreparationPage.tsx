import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Check, ChevronRight, ExternalLink, Link2, LoaderCircle, Pencil, PencilLine, Play, RefreshCw, Save, Sparkles, Trash2, X } from "lucide-react";
import { Badge, Card, CardHeader } from "../../components/ui";
import { useInterviewFlow } from "../../hooks/useInterviewFlow";
import { hasLocalDatabase } from "../../services/applications";
import { generateInterviewPreparation, getLatestInterviewPreparation, listApplicationAiCalls, type AiCallSummary, type StoredInterviewPreparation } from "../../services/ai";
import { requestAiSendConfirmation } from "../../services/settings";
import { openExternalUrl } from "../../services/external";

const isValidWebUrl = (value: string) => {
  try { return ["http:", "https:"].includes(new URL(value).protocol); } catch { return false; }
};

const formatImportedAt = (value: string) => {
  const parsed = new Date(value);
  return Number.isNaN(parsed.getTime()) ? value : parsed.toLocaleString("zh-CN", { dateStyle: "short", timeStyle: "short" });
};

export default function PreparationPage() {
  const navigate = useNavigate();
  const { eligibleApplications, experienceLinks, selectedApplicationId, setSelectedApplicationId, importExperienceLink, addManualExperience, analyzeExperienceLink, deleteExperienceSource, updateExperienceQuestions } = useInterviewFlow();
  const [url, setUrl] = useState("");
  const [urlTouched, setUrlTouched] = useState(false);
  const [importMode, setImportMode] = useState<"link" | "manual">("link");
  const [manualTitle, setManualTitle] = useState("");
  const [manualQuestions, setManualQuestions] = useState("");
  const [expandedLinkId, setExpandedLinkId] = useState<string>();
  const [importing, setImporting] = useState(false);
  const [analyzingId, setAnalyzingId] = useState<string>();
  const [deletingId, setDeletingId] = useState<string>();
  const [experienceError, setExperienceError] = useState("");
  const [questionEditor, setQuestionEditor] = useState<{ sourceId: string; index: number; value: string }>();
  const [questionSaving, setQuestionSaving] = useState("");
  const [preparation, setPreparation] = useState<StoredInterviewPreparation | null>(null);
  const [aiCalls, setAiCalls] = useState<AiCallSummary[]>([]);
  const [generating, setGenerating] = useState(false);
  const [aiError, setAiError] = useState("");
  const selected = eligibleApplications.find((item) => item.id === selectedApplicationId) ?? eligibleApplications[0];
  const links = experienceLinks.filter((item) => item.applicationId === selected?.id);
  const extractedCount = links.reduce((sum, item) => sum + item.questions.length, 0);
  const validUrl = isValidWebUrl(url.trim());
  const traceCall = preparation ? aiCalls.find((item) => item.id === preparation.aiCallId) : undefined;

  useEffect(() => {
    if (!selected?.id) { setPreparation(null); setAiCalls([]); return; }
    Promise.all([getLatestInterviewPreparation(selected.id), listApplicationAiCalls(selected.id)])
      .then(([latest, calls]) => { setPreparation(latest); setAiCalls(calls); setAiError(""); })
      .catch((reason) => setAiError(String(reason)));
  }, [selected?.id]);

  if (!selected) return <Card><div className="interview-empty"><Link2 size={32}/><h3>当前没有需要准备的岗位</h3><p>已拒绝或已获得 Offer 的投递不会出现在这里。</p></div></Card>;

  const submitLink = async () => {
    setUrlTouched(true);
    if (!validUrl || importing) return;
    setImporting(true); setExperienceError("");
    try {
      const created = await importExperienceLink(selected.id, url.trim());
      setExpandedLinkId(created.id);
      setUrl("");
      setUrlTouched(false);
      setAnalyzingId(created.id);
      await analyzeExperienceLink(created.id);
    } catch (reason) {
      setExperienceError(String(reason));
    } finally {
      setImporting(false);
      setAnalyzingId(undefined);
    }
  };

  const parsedManualQuestions = manualQuestions.split(/\n+/).map((item) => item.trim()).filter(Boolean);
  const submitManual = async () => {
    if (!parsedManualQuestions.length || importing) return;
    setImporting(true); setExperienceError("");
    try {
      const created = await addManualExperience(selected.id, manualTitle.trim(), parsedManualQuestions);
      setExpandedLinkId(created.id);
      setManualTitle("");
      setManualQuestions("");
    } catch (reason) {
      setExperienceError(String(reason));
    } finally {
      setImporting(false);
    }
  };

  const retryAnalysis = async (id: string) => {
    if (analyzingId) return;
    setAnalyzingId(id); setExperienceError("");
    try { await analyzeExperienceLink(id); }
    catch (reason) { setExperienceError(String(reason)); }
    finally { setAnalyzingId(undefined); }
  };

  const reanalyze = (id: string) => {
    if (!window.confirm("重新提取会覆盖这份来源中已经编辑过的问题，确定继续吗？")) return;
    retryAnalysis(id);
  };

  const removeExperience = async (id: string, title: string) => {
    if (!window.confirm(`确定删除“${title}”吗？删除后其中的面试题将不再用于模拟面试。`)) return;
    setDeletingId(id); setExperienceError("");
    try {
      await deleteExperienceSource(id);
      if (expandedLinkId === id) setExpandedLinkId(undefined);
      if (questionEditor?.sourceId === id) setQuestionEditor(undefined);
    } catch (reason) { setExperienceError(String(reason)); }
    finally { setDeletingId(undefined); }
  };

  const saveQuestion = async () => {
    if (!questionEditor || questionSaving) return;
    const value = questionEditor.value.trim();
    if (!value) { setExperienceError("问题内容不能为空；如需移除，请使用删除按钮。"); return; }
    const source = experienceLinks.find((item) => item.id === questionEditor.sourceId);
    if (!source) return;
    const savingKey = `${source.id}:${questionEditor.index}`;
    const questions = [...source.questions];
    questions[questionEditor.index] = value;
    setQuestionSaving(savingKey); setExperienceError("");
    try {
      await updateExperienceQuestions(source.id, questions);
      setQuestionEditor(undefined);
    } catch (reason) { setExperienceError(String(reason)); }
    finally { setQuestionSaving(""); }
  };

  const removeQuestion = async (sourceId: string, index: number, question: string) => {
    if (!window.confirm(`确定删除这道问题吗？\n\n${question}`)) return;
    const source = experienceLinks.find((item) => item.id === sourceId);
    if (!source) return;
    const savingKey = `${sourceId}:${index}`;
    setQuestionSaving(savingKey); setExperienceError("");
    try {
      await updateExperienceQuestions(sourceId, source.questions.filter((_, questionIndex) => questionIndex !== index));
      if (questionEditor?.sourceId === sourceId) setQuestionEditor(undefined);
    } catch (reason) { setExperienceError(String(reason)); }
    finally { setQuestionSaving(""); }
  };

  const renderExtractedQuestions = (link: (typeof links)[number]) => <>
    <div className="extracted-head"><strong>{link.source === "manual" ? "人工录入" : "提取到"} {link.questions.length} 道问题</strong>{link.url && <span className="extracted-head-actions"><button type="button" disabled={Boolean(analyzingId || questionSaving)} onClick={() => reanalyze(link.id)}><RefreshCw size={13}/>重新提取</button><a href={link.url} onClick={(event) => { event.preventDefault(); if (link.url) void openExternalUrl(link.url).catch((reason) => setExperienceError(String(reason))); }}>查看原帖<ExternalLink size={13}/></a></span>}</div>
    {link.questions.length ? <ol className="extracted-questions">{link.questions.map((question, index) => {
      const editing = questionEditor?.sourceId === link.id && questionEditor.index === index;
      const saving = questionSaving === `${link.id}:${index}`;
      return <li key={`${index}-${question}`}><Check size={14}/>{editing ? <div className="question-editor"><input autoFocus value={questionEditor.value} onChange={(event) => setQuestionEditor({ ...questionEditor, value: event.target.value })} onKeyDown={(event) => { if (event.key === "Enter") saveQuestion(); if (event.key === "Escape") setQuestionEditor(undefined); }}/><div className="question-editor-actions"><button type="button" title="保存" aria-label="保存修改" disabled={Boolean(questionSaving)} onClick={saveQuestion}>{saving ? <LoaderCircle className="spin" size={14}/> : <Save size={14}/>}</button><button type="button" title="取消" aria-label="取消修改" disabled={Boolean(questionSaving)} onClick={() => setQuestionEditor(undefined)}><X size={14}/></button></div></div> : <><span>{question}</span><span className="question-actions"><button type="button" title="编辑问题" aria-label={`编辑问题：${question}`} disabled={Boolean(questionSaving)} onClick={() => setQuestionEditor({ sourceId: link.id, index, value: question })}><Pencil size={13}/></button><button type="button" title="删除问题" aria-label={`删除问题：${question}`} disabled={Boolean(questionSaving)} onClick={() => removeQuestion(link.id, index, question)}>{saving ? <LoaderCircle className="spin" size={13}/> : <Trash2 size={13}/>}</button></span></>}</li>;
    })}</ol> : <div className="questions-empty"><p>这个来源中已没有问题，可以删除整个来源或重新导入。</p></div>}
  </>;

  const generatePreparation = async () => {
    setGenerating(true); setAiError("");
    try {
      const confirmed = !selected.resumeName || await requestAiSendConfirmation(`将把”${selected.resumeName}”和岗位信息发给 AI 服务，用来匹配简历和面试准备。是否继续？`);
      if (!confirmed) return;
      const generated = await generateInterviewPreparation(selected.id, confirmed);
      setPreparation(generated);
      setAiCalls(await listApplicationAiCalls(selected.id));
    } catch (reason) { setAiError(String(reason)); } finally { setGenerating(false); }
  };

  return <div className="prep-workspace">
    <Card className="prep-jobs">
      <div className="prep-jobs-head"><strong>进行中的岗位</strong><span>{eligibleApplications.length}</span></div>
      {eligibleApplications.map((application) => {
        const sourceCount = experienceLinks.filter((item) => item.applicationId === application.id).length;
        return <button key={application.id} className={application.id === selected.id ? "active" : ""} onClick={() => setSelectedApplicationId(application.id)}>
          <span className="company-logo">{application.companyMark}</span>
          <span><strong>{application.company}</strong><small>{application.role} · {application.stage}</small></span>
          <Badge tone={sourceCount ? "purple" : "gray"}>{sourceCount ? `${sourceCount} 份面经` : "待导入"}</Badge>
          <ChevronRight size={16}/>
        </button>;
      })}
    </Card>

    <div className="prep-detail">
      <div className="prep-position-head">
        <div><Badge tone={selected.stageTone}>{selected.stage}</Badge><h2>{selected.company} · {selected.role}</h2><p>{selected.city} · 面经只关联当前岗位，不会混到其他投递中</p></div>
        <button className="button button--primary" onClick={() => navigate("/mock-interview")}><Play size={16}/>用此岗位开始模拟</button>
      </div>

      <Card className="ai-preparation-card">
        <CardHeader title="AI 面试准备建议" subtitle="根据岗位信息和简历为你生成个性化的面试准备建议"/>
        <div className="ai-preparation-toolbar"><div><Sparkles size={18}/><span>{preparation ? `上次生成：${new Date(preparation.createdAt).toLocaleString("zh-CN")} · ${preparation.model}` : "尚未生成真实建议"}</span></div><button className="button button--primary" disabled={generating} onClick={generatePreparation}><Sparkles size={15}/>{generating ? "生成中…" : preparation ? "重新生成" : "生成准备建议"}</button></div>
        {aiError && <p className="field-error ai-preparation-error">{aiError}</p>}
        {!hasLocalDatabase && <p className="link-import-note">当前为预览模式，展示的是示例数据，不会涉及你的真实信息。</p>}
        {preparation && <div className="ai-preparation-result">
          <p className="ai-preparation-summary">{preparation.content.summary}</p>
          {preparation.content.resumeMatch && <section className="ai-predicted-questions"><h3>简历匹配分析</h3><article><b>✓</b><div><strong>{preparation.content.resumeMatch.summary}</strong><p>匹配优势：{preparation.content.resumeMatch.strengths.join("；") || "暂无明确优势"}</p><p>表述风险：{preparation.content.resumeMatch.risks.join("；") || "暂未发现"}</p><small>建议准备证据：{preparation.content.resumeMatch.evidenceToPrepare.join("；") || "暂无"}</small></div></article></section>}
          <div className="ai-preparation-columns"><section><h3>重点准备</h3>{preparation.content.focusAreas.map((item) => <article key={item.title}><Badge tone={item.priority === "high" ? "red" : item.priority === "medium" ? "orange" : "blue"}>{item.priority === "high" ? "高" : item.priority === "medium" ? "中" : "低"}</Badge><div><strong>{item.title}</strong><p>{item.reason}</p></div></article>)}</section><section><h3>行动计划</h3>{preparation.content.actionPlan.map((item) => <article key={item.action}><span className="ai-action-time">{item.estimatedMinutes} 分钟</span><div><strong>{item.action}</strong></div></article>)}</section></div>
          <section className="ai-predicted-questions"><h3>预测问题</h3>{preparation.content.predictedQuestions.map((item, index) => <article key={`${index}-${item.question}`}><b>{index + 1}</b><div><strong>{item.question}</strong><p>{item.rationale}</p><small>依据：{item.sourceBasis.join(" · ")}</small></div></article>)}</section>
          <div className="ai-trace"><span>生成来源：{preparation.sources.length} 项</span>{traceCall && <span>耗时 {traceCall.durationMs ?? 0} ms</span>}</div>
        </div>}
      </Card>

      <Card className="link-import-card">
        <div className="experience-import-tabs"><button className={importMode === "link" ? "active" : ""} onClick={() => setImportMode("link")}><Link2 size={15}/>网页链接</button><button className={importMode === "manual" ? "active" : ""} onClick={() => setImportMode("manual")}><PencilLine size={15}/>人工录入</button></div>
        {importMode === "link" ? <><CardHeader title="导入网页面经" subtitle="粘贴面经帖子的链接，系统会自动提取里面的面试问题"/><div className="link-import-form">
          <div className={`url-field ${urlTouched && !validUrl ? "invalid" : ""}`}><Link2 size={17}/><input value={url} onBlur={() => setUrlTouched(true)} onChange={(event) => setUrl(event.target.value)} onKeyDown={(event) => { if (event.key === "Enter") submitLink(); }} placeholder="https://www.nowcoder.com/discuss/..."/></div>
          <button className="button button--primary" disabled={importing || !validUrl} onClick={submitLink}>{importing && <LoaderCircle className="spin" size={15}/>} {importing ? "导入并分析中…" : "导入链接"}</button>
        </div>
        {urlTouched && url && !validUrl && <p className="field-error">请输入以 http:// 或 https:// 开头的有效网页地址</p>}
        <p className="link-import-note">链接内容只在本地保存；如果帖子需要登录才能看，请改用人工录入。</p></> : <><CardHeader title="人工录入面经" subtitle="把聊天记录或笔记里的面试题整理成题单"/><div className="manual-experience-form"><label><span>来源名称（可选）</span><input value={manualTitle} onChange={(event) => setManualTitle(event.target.value)} placeholder="例如：朋友分享的蚂蚁一面问题"/></label><label><span>面试问题</span><textarea rows={6} value={manualQuestions} onChange={(event) => setManualQuestions(event.target.value)} placeholder={"每行输入一道问题，例如：\n如何保证消息不重复消费？\n介绍一下你负责的订单系统。"}/><small>已识别 {parsedManualQuestions.length} 道题</small></label><button className="button button--primary" disabled={importing || !parsedManualQuestions.length} onClick={submitManual}>{importing ? "保存中…" : "保存人工面经"}</button></div></>}
      </Card>

      <Card className="experience-links-card">
        <CardHeader title="该岗位的面经来源" subtitle={`${links.length} 个来源 · 已提取 ${extractedCount} 道题`}/>
        {experienceError && <p className="field-error experience-operation-error">{experienceError}</p>}
        <div className="experience-link-list">{links.length ? links.map((link) => <article key={link.id} className={expandedLinkId === link.id ? "expanded" : ""}>
          <div className="experience-link-row"><button className="experience-link-main" onClick={() => setExpandedLinkId(expandedLinkId === link.id ? undefined : link.id)}>
              <span className="stat-icon blue">{link.source === "link" ? <Link2/> : <PencilLine/>}</span>
              <span><strong>{link.title}</strong><small>{link.source === "link" ? link.url : `人工录入 · ${link.questions.length} 道题`}</small><em>{formatImportedAt(link.importedAt)}</em></span>
              <Badge tone={link.status === "已提取" ? "green" : link.status === "分析失败" ? "red" : "orange"}>{analyzingId === link.id ? "分析中" : link.status}</Badge>
              <ChevronRight size={16}/>
            </button><button className="experience-delete-button" aria-label={`删除 ${link.title}`} title="删除该面经来源" disabled={deletingId === link.id} onClick={() => removeExperience(link.id, link.title)}>{deletingId === link.id ? <LoaderCircle className="spin" size={15}/> : <Trash2 size={15}/>}</button></div>
          {expandedLinkId === link.id && <div className="experience-link-detail">
            {link.status === "待分析" || analyzingId === link.id ? <div className="analysis-pending"><LoaderCircle className={analyzingId === link.id ? "spin" : ""} size={21}/><div><strong>{analyzingId === link.id ? "正在读取并分析网页" : "等待网页分析"}</strong><p>系统会读取原帖内容，自动提取其中的面试问题。</p></div>{analyzingId !== link.id && <button className="button button--secondary" onClick={() => retryAnalysis(link.id)}><Sparkles size={15}/>开始分析</button>}</div> : link.status === "分析失败" ? <div className="analysis-pending analysis-failed"><Sparkles size={21}/><div><strong>没有完成提取</strong><p>{link.errorMessage || "网页无法访问或未识别到面试问题。"}</p></div><button className="button button--secondary" onClick={() => retryAnalysis(link.id)}><Sparkles size={15}/>重试</button></div> : renderExtractedQuestions(link)}
          </div>}
        </article>) : <div className="experience-empty"><Link2 size={26}/><p>还没有面经来源，导入帖子或人工录入后即可用于模拟面试。</p></div>}</div>
      </Card>
    </div>
  </div>;
}
