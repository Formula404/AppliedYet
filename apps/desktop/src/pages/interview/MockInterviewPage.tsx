import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Check, ChevronLeft, ChevronRight, FileText, LogOut, Mic2, Play, RotateCcw, Sparkles } from "lucide-react";
import { Badge, Card, CardHeader } from "../../components/ui";
import { useInterviewFlow } from "../../hooks/useInterviewFlow";
import { generateResumeQuestions } from "../../services/ai";
import { hasLocalDatabase } from "../../services/applications";
import { listQuestionBankItems, type QuestionBankItem } from "../../services/interviews";
import { requestAiSendConfirmation } from "../../services/settings";
import { showError, showSuccess } from "../../services/feedback";
import { trackOperation } from "../../services/operations";

const sessionTime = (value: string) => {
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString("zh-CN", { dateStyle: "short", timeStyle: "short" });
};

export default function MockInterviewPage() {
  const navigate = useNavigate();
  const { applications, eligibleApplications, experienceLinks, sessions, selectedApplicationId, setSelectedApplicationId, createMockSession, updateSessionAnswer, updateSessionProgress, completeSession } = useInterviewFlow();
  const selected = eligibleApplications.find((item) => item.id === selectedApplicationId) ?? eligibleApplications[0];
  const [count, setCount] = useState(8);
  const [useExperience, setUseExperience] = useState(true);
  const [useAi, setUseAi] = useState(true);
  const [useBank, setUseBank] = useState(false);
  const [bankItems, setBankItems] = useState<QuestionBankItem[]>([]);
  const [sessionId, setSessionId] = useState<string>();
  const [questionIndex, setQuestionIndex] = useState(0);
  const [generating, setGenerating] = useState(false);
  const [aiError, setAiError] = useState("");
  const extractedLinks = experienceLinks.filter((item) => item.applicationId === selected?.id && item.status === "已提取");
  const experienceSelected = useExperience && extractedLinks.length > 0;
  const session = sessions.find((item) => item.id === sessionId);
  const question = session?.questions[questionIndex];
  const sessionApplication = applications.find((item) => item.id === session?.applicationId);

  useEffect(() => { if (hasLocalDatabase) listQuestionBankItems({ pageSize: 100 }).then((page) => setBankItems(page.items)).catch(() => undefined); }, []);
  useEffect(() => { if (aiError) showError(aiError, "模拟面试操作失败"); }, [aiError]);

  if (!selected) return <Card><div className="interview-empty"><Mic2 size={32}/><h3>没有可模拟的岗位</h3><p>请先添加一个流程中的投递。</p></div></Card>;

  if (session && question) return <Card className="mock-runner">
    <div className="runner-head"><div><Badge tone="purple">模拟进行中</Badge><h2>{sessionApplication?.company} · {sessionApplication?.role}</h2></div><div className="runner-head-actions"><span>{questionIndex + 1} / {session.questions.length}</span><button className="button button--secondary" onClick={async () => { try { await trackOperation("保存模拟面试进度", () => updateSessionProgress(session.id, questionIndex)); setSessionId(undefined); showSuccess("当前题目与回答进度已保存。", "模拟面试已暂停"); } catch (reason) { setAiError(String(reason)); } }}><LogOut size={14}/>退出并保存</button></div></div>
    <div className="runner-progress"><i style={{ width: `${((questionIndex + 1) / session.questions.length) * 100}%` }}/></div>
    <div className="runner-question"><small>{question.source}</small><h3>{question.prompt}</h3><textarea rows={9} value={question.answer} onChange={(event) => { void updateSessionAnswer(session.id, question.id, event.target.value).catch((reason) => setAiError(String(reason))); }} placeholder="记录你的完整回答；内容会实时保存，可稍后继续。"/><p>退出后可从“最近会话”恢复到当前题。</p></div>
    <div className="runner-actions"><button className="button button--secondary" disabled={questionIndex === 0} onClick={() => { const next = questionIndex - 1; setQuestionIndex(next); void updateSessionProgress(session.id, next).catch((reason) => setAiError(String(reason))); }}><ChevronLeft size={16}/>上一题</button>{questionIndex < session.questions.length - 1 ? <button className="button button--primary" onClick={() => { const next = questionIndex + 1; setQuestionIndex(next); void updateSessionProgress(session.id, next).catch((reason) => setAiError(String(reason))); }}>下一题<ChevronRight size={16}/></button> : <button className="button button--primary" onClick={async () => { try { await trackOperation("完成模拟面试", () => completeSession(session.id), `${session.questions.length} 道问题`); showSuccess("本场回答已保存，可以开始复盘。", "模拟面试已完成"); navigate(`/reviews?session=${session.id}`); } catch (reason) { setAiError(String(reason)); } }}><Check size={16}/>完成并进入复盘</button>}</div>
  </Card>;

  const resumeReady = Boolean(selected.resumeProfileId);
  const start = async () => {
    setGenerating(true); setAiError("");
    try {
      let confirmed = false;
      if (useAi && resumeReady) {
        confirmed = await requestAiSendConfirmation(`将把“${selected.resumeName || "关联简历"}”和岗位信息发送给 AI，生成本场简历深挖题。是否继续？`);
        if (!confirmed) return;
      }
      const created = await trackOperation("生成模拟面试题目", async (operation) => {
        let generated: string[] | undefined;
        if (useAi && resumeReady) {
          operation.update("AI 正在生成简历深挖题");
          generated = (await generateResumeQuestions(selected.id, count, confirmed)).map((item) => item.question);
        }
        operation.update("正在组合本场题目与题源");
        return createMockSession({ applicationId: selected.id, questionCount: count, useExperience: experienceSelected, useAi: useAi && resumeReady, useBank, bankQuestions: bankItems.map((item) => item.prompt), resumeQuestions: generated });
      }, `${selected.company} · ${count} 题`);
      setSessionId(created.id);
      setQuestionIndex(created.currentQuestionIndex);
    } catch (reason) { setAiError(String(reason)); } finally { setGenerating(false); }
  };

  return <div className="mock-layout">
    <Card className="mock-config"><CardHeader title="创建一场岗位模拟" subtitle={`预计 ${Math.max(10, count * 3)} 分钟`}/>
      <div className="form-grid"><label className="full"><span>目标岗位</span><select value={selected.id} onChange={(event) => setSelectedApplicationId(event.target.value)}>{eligibleApplications.map((application) => <option key={application.id} value={application.id}>{application.company} · {application.role}</option>)}</select></label></div>
      <div className="question-source"><button className={experienceSelected ? "selected" : ""} disabled={!extractedLinks.length} onClick={() => setUseExperience((value) => !value)}><FileText/><span><strong>网页面经题</strong><small>{extractedLinks.length ? `${extractedLinks.length} 个已分析链接` : "请先导入并分析面经链接"}</small></span><i>{experienceSelected && <Check size={14}/>}</i></button><button className={useAi && resumeReady ? "selected" : ""} disabled={!resumeReady} onClick={() => setUseAi((value) => !value)}><Sparkles/><span><strong>AI 简历出题</strong><small>{resumeReady ? `基于 ${selected.resumeName || "关联简历"} 生成` : "请先在投递详情关联简历"}</small></span><i>{useAi && resumeReady && <Check size={14}/>}</i></button><button className={useBank && bankItems.length ? "selected" : ""} disabled={!bankItems.length} onClick={() => setUseBank((value) => !value)}><FileText/><span><strong>个人题库</strong><small>{bankItems.length ? `${bankItems.length} 道已沉淀问题` : "题库暂无问题"}</small></span><i>{useBank && bankItems.length > 0 && <Check size={14}/>}</i></button></div>
      <div className="question-count"><label><span>问题数量</span><input type="number" min="1" max="30" value={count} onChange={(event) => setCount(Math.max(1, Math.min(30, Number(event.target.value) || 1)))}/><small>题</small></label><div>{[5, 8, 12, 15].map((value) => <button key={value} className={count === value ? "active" : ""} onClick={() => setCount(value)}>{value} 题</button>)}</div></div>
      <button className="button button--primary large" disabled={generating || (!experienceSelected && !(useAi && resumeReady) && !(useBank && bankItems.length))} onClick={start}><Play size={17}/>{generating ? "正在生成简历问题…" : `生成并开始 ${count} 题模拟`}</button>
    </Card>
    <div className="mock-side"><Card><CardHeader title="本场题源"/><div className="source-summary"><span><FileText/><b>{extractedLinks.reduce((sum, item) => sum + item.questions.length, 0)}<small>道网页面经题</small></b></span><span><Sparkles/><b>AI<small>简历动态出题</small></b></span><span><FileText/><b>{bankItems.length}<small>道个人题库题</small></b></span></div><button className="button button--secondary" onClick={() => navigate("/preparation")}>管理面经链接</button></Card><Card><CardHeader title="最近会话" subtitle="可以接着上次的进度继续作答"/><div className="recent-mocks">{sessions.slice(0, 5).map((item) => <button key={item.id} onClick={() => { if (item.status === "进行中") { setSessionId(item.id); setQuestionIndex(Math.max(0, Math.min(item.currentQuestionIndex, item.questions.length - 1))); } else { navigate(`/reviews?session=${item.id}`); } }}><span className="stat-icon purple">{item.status === "进行中" ? <RotateCcw/> : <Mic2/>}</span><span><strong>{item.type} · {item.round}</strong><small>{item.questions.length} 题 · {sessionTime(item.createdAt)}</small></span><Badge tone={item.status === "复盘完成" ? "green" : item.status === "待复盘" ? "orange" : "purple"}>{item.status === "进行中" ? "继续作答" : item.status}</Badge></button>)}</div></Card></div>
  </div>;
}
