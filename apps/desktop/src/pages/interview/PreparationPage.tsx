import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Check, ChevronRight, ExternalLink, Link2, PencilLine, Play, Sparkles } from "lucide-react";
import { Badge, Card, CardHeader } from "../../components/ui";
import { useInterviewFlow } from "../../hooks/useInterviewFlow";
import { hasLocalDatabase } from "../../services/applications";
import { generateInterviewPreparation, getLatestInterviewPreparation, listApplicationAiCalls, type AiCallSummary, type StoredInterviewPreparation } from "../../services/ai";

const isValidWebUrl = (value: string) => {
  try { return ["http:", "https:"].includes(new URL(value).protocol); } catch { return false; }
};

export default function PreparationPage() {
  const navigate = useNavigate();
  const { eligibleApplications, experienceLinks, selectedApplicationId, setSelectedApplicationId, importExperienceLink, addManualExperience, analyzeExperienceLink } = useInterviewFlow();
  const [url, setUrl] = useState("");
  const [urlTouched, setUrlTouched] = useState(false);
  const [importMode, setImportMode] = useState<"link" | "manual">("link");
  const [manualTitle, setManualTitle] = useState("");
  const [manualQuestions, setManualQuestions] = useState("");
  const [expandedLinkId, setExpandedLinkId] = useState<string>();
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

  const submitLink = () => {
    setUrlTouched(true);
    if (!validUrl) return;
    const id = importExperienceLink(selected.id, url.trim());
    setExpandedLinkId(id);
    setUrl("");
    setUrlTouched(false);
  };

  const parsedManualQuestions = manualQuestions.split(/\n+/).map((item) => item.trim()).filter(Boolean);
  const submitManual = () => {
    if (!parsedManualQuestions.length) return;
    const id = addManualExperience(selected.id, manualTitle.trim(), parsedManualQuestions);
    setExpandedLinkId(id);
    setManualTitle("");
    setManualQuestions("");
  };

  const generatePreparation = async () => {
    if (hasLocalDatabase && selected.resumeName && !window.confirm(`将把“${selected.resumeName}”的结构化内容和当前岗位 JD 发送给已配置的 AI 服务，用于简历匹配与面试准备。是否继续？`)) return;
    setGenerating(true); setAiError("");
    try {
      const generated = await generateInterviewPreparation(selected.id);
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
        <div><Badge tone={selected.stageTone}>{selected.stage}</Badge><h2>{selected.company} · {selected.role}</h2><p>{selected.city} · 面经链接及提取题目仅归属于当前投递</p></div>
        <button className="button button--primary" onClick={() => navigate("/mock-interview")}><Play size={16}/>用此岗位开始模拟</button>
      </div>

      <Card className="ai-preparation-card">
        <CardHeader title="AI 面试准备建议" subtitle="使用当前投递的 JD、关联简历与岗位上下文，结果和来源会保存在本地"/>
        <div className="ai-preparation-toolbar"><div><Sparkles size={18}/><span>{preparation ? `上次生成：${new Date(preparation.createdAt).toLocaleString("zh-CN")} · ${preparation.model}` : "尚未生成真实建议"}</span></div><button className="button button--primary" disabled={generating} onClick={generatePreparation}><Sparkles size={15}/>{generating ? "生成中…" : preparation ? "重新生成" : "生成准备建议"}</button></div>
        {aiError && <p className="field-error ai-preparation-error">{aiError}</p>}
        {!hasLocalDatabase && <p className="link-import-note">浏览器演示模式使用预置 AI 结果，不会发送任何真实简历或岗位数据。</p>}
        {preparation && <div className="ai-preparation-result">
          <p className="ai-preparation-summary">{preparation.content.summary}</p>
          {preparation.content.resumeMatch && <section className="ai-predicted-questions"><h3>简历匹配分析</h3><article><b>✓</b><div><strong>{preparation.content.resumeMatch.summary}</strong><p>匹配优势：{preparation.content.resumeMatch.strengths.join("；") || "暂无明确优势"}</p><p>表述风险：{preparation.content.resumeMatch.risks.join("；") || "暂未发现"}</p><small>建议准备证据：{preparation.content.resumeMatch.evidenceToPrepare.join("；") || "暂无"}</small></div></article></section>}
          <div className="ai-preparation-columns"><section><h3>重点准备</h3>{preparation.content.focusAreas.map((item) => <article key={item.title}><Badge tone={item.priority === "high" ? "red" : item.priority === "medium" ? "orange" : "blue"}>{item.priority === "high" ? "高" : item.priority === "medium" ? "中" : "低"}</Badge><div><strong>{item.title}</strong><p>{item.reason}</p></div></article>)}</section><section><h3>行动计划</h3>{preparation.content.actionPlan.map((item) => <article key={item.action}><span className="ai-action-time">{item.estimatedMinutes} 分钟</span><div><strong>{item.action}</strong></div></article>)}</section></div>
          <section className="ai-predicted-questions"><h3>预测问题</h3>{preparation.content.predictedQuestions.map((item, index) => <article key={`${index}-${item.question}`}><b>{index + 1}</b><div><strong>{item.question}</strong><p>{item.rationale}</p><small>依据：{item.sourceBasis.join(" · ")}</small></div></article>)}</section>
          <div className="ai-trace"><span>调用 ID：{preparation.aiCallId.slice(0, 8)}</span><span>来源项：{preparation.sources.length}</span>{traceCall && <span>{traceCall.attempts} 次尝试 · {traceCall.durationMs ?? 0} ms</span>}</div>
        </div>}
      </Card>

      <Card className="link-import-card">
        <div className="experience-import-tabs"><button className={importMode === "link" ? "active" : ""} onClick={() => setImportMode("link")}><Link2 size={15}/>网页链接</button><button className={importMode === "manual" ? "active" : ""} onClick={() => setImportMode("manual")}><PencilLine size={15}/>人工录入</button></div>
        {importMode === "link" ? <><CardHeader title="导入网页面经" subtitle="粘贴公开帖子链接，后续由网页分析服务自动提取题目"/><div className="link-import-form">
          <div className={`url-field ${urlTouched && !validUrl ? "invalid" : ""}`}><Link2 size={17}/><input value={url} onBlur={() => setUrlTouched(true)} onChange={(event) => setUrl(event.target.value)} onKeyDown={(event) => { if (event.key === "Enter") submitLink(); }} placeholder="https://www.nowcoder.com/discuss/..."/></div>
          <button className="button button--primary" onClick={submitLink}>导入链接</button>
        </div>
        {urlTouched && url && !validUrl && <p className="field-error">请输入以 http:// 或 https:// 开头的有效网页地址</p>}
        <p className="link-import-note">这里只保存链接和提取结果，不上传本地文件。当前“分析”使用前端演示题目。</p></> : <><CardHeader title="人工录入面经" subtitle="适合整理聊天记录、评论区内容或无法直接访问的帖子"/><div className="manual-experience-form"><label><span>来源名称（可选）</span><input value={manualTitle} onChange={(event) => setManualTitle(event.target.value)} placeholder="例如：朋友分享的蚂蚁一面问题"/></label><label><span>面试问题</span><textarea rows={6} value={manualQuestions} onChange={(event) => setManualQuestions(event.target.value)} placeholder={"每行输入一道问题，例如：\n如何保证消息不重复消费？\n介绍一下你负责的订单系统。"}/><small>已识别 {parsedManualQuestions.length} 道题</small></label><button className="button button--primary" disabled={!parsedManualQuestions.length} onClick={submitManual}>保存人工面经</button></div></>}
      </Card>

      <Card className="experience-links-card">
        <CardHeader title="该岗位的面经来源" subtitle={`${links.length} 个链接 · 已提取 ${extractedCount} 道题`}/>
        <div className="experience-link-list">{links.length ? links.map((link) => <article key={link.id} className={expandedLinkId === link.id ? "expanded" : ""}>
          <button className="experience-link-main" onClick={() => setExpandedLinkId(expandedLinkId === link.id ? undefined : link.id)}>
            <span className="stat-icon blue">{link.source === "link" ? <Link2/> : <PencilLine/>}</span>
            <span><strong>{link.title}</strong><small>{link.source === "link" ? link.url : `人工录入 · ${link.questions.length} 道题`}</small><em>{link.importedAt}</em></span>
            <Badge tone={link.status === "已提取" ? "green" : link.status === "分析失败" ? "red" : "orange"}>{link.status}</Badge>
            <ChevronRight size={16}/>
          </button>
          {expandedLinkId === link.id && <div className="experience-link-detail">
            {link.status === "待分析" ? <div className="analysis-pending"><Sparkles size={21}/><div><strong>等待网页分析</strong><p>后端接入后将在这里抓取正文、识别面试问题并去重。</p></div><button className="button button--secondary" onClick={() => analyzeExperienceLink(link.id)}><Sparkles size={15}/>演示分析并提取</button></div> : <><div className="extracted-head"><strong>{link.source === "manual" ? "人工录入" : "提取到"} {link.questions.length} 道问题</strong>{link.url && <a href={link.url} target="_blank" rel="noreferrer">查看原帖<ExternalLink size={13}/></a>}</div><ol className="extracted-questions">{link.questions.map((question) => <li key={question}><Check size={14}/><span>{question}</span></li>)}</ol></>}
          </div>}
        </article>) : <div className="experience-empty"><Link2 size={26}/><p>还没有面经链接，导入帖子后即可分析题目并用于模拟面试。</p></div>}</div>
      </Card>
    </div>
  </div>;
}
