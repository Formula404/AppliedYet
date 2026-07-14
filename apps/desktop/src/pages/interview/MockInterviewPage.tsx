import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Check, ChevronLeft, ChevronRight, FileText, Mic2, Play, Sparkles } from "lucide-react";
import { Badge, Card, CardHeader } from "../../components/ui";
import { useInterviewFlow } from "../../hooks/useInterviewFlow";

export default function MockInterviewPage() {
  const navigate = useNavigate();
  const { eligibleApplications, experienceLinks, sessions, selectedApplicationId, setSelectedApplicationId, createMockSession, updateSessionAnswer, completeSession } = useInterviewFlow();
  const selected = eligibleApplications.find((item) => item.id === selectedApplicationId) ?? eligibleApplications[0];
  const [count, setCount] = useState(8);
  const [useExperience, setUseExperience] = useState(true);
  const [useAi, setUseAi] = useState(true);
  const [sessionId, setSessionId] = useState<string>();
  const [questionIndex, setQuestionIndex] = useState(0);
  const extractedLinks = experienceLinks.filter((item) => item.applicationId === selected?.id && item.status === "已提取");
  const experienceSelected = useExperience && extractedLinks.length > 0;
  const session = sessions.find((item) => item.id === sessionId);
  const question = session?.questions[questionIndex];

  if (!selected) return <Card><div className="interview-empty"><Mic2 size={32}/><h3>没有可模拟的岗位</h3><p>请先添加一个流程中的投递。</p></div></Card>;

  if (session && question) return <Card className="mock-runner">
    <div className="runner-head"><div><Badge tone="purple">模拟进行中</Badge><h2>{selected.company} · {selected.role}</h2></div><span>{questionIndex + 1} / {session.questions.length}</span></div>
    <div className="runner-progress"><i style={{ width: `${((questionIndex + 1) / session.questions.length) * 100}%` }}/></div>
    <div className="runner-question"><small>{question.source}</small><h3>{question.prompt}</h3><textarea rows={9} value={question.answer} onChange={(event) => updateSessionAnswer(session.id, question.id, event.target.value)} placeholder="记录你的完整回答。后端接入后可切换为语音回答与自动转写。"/><p>回答会随本场模拟保留，并在复盘中逐题展示。</p></div>
    <div className="runner-actions"><button className="button button--secondary" disabled={questionIndex === 0} onClick={() => setQuestionIndex((value) => value - 1)}><ChevronLeft size={16}/>上一题</button>{questionIndex < session.questions.length - 1 ? <button className="button button--primary" onClick={() => setQuestionIndex((value) => value + 1)}>下一题<ChevronRight size={16}/></button> : <button className="button button--primary" onClick={() => { completeSession(session.id); navigate(`/reviews?session=${session.id}`); }}><Check size={16}/>完成并查看逐题复盘</button>}</div>
  </Card>;

  const start = () => {
    const id = createMockSession({ applicationId: selected.id, questionCount: count, useExperience: experienceSelected, useAi });
    setSessionId(id);
  };

  return <div className="mock-layout">
    <Card className="mock-config"><CardHeader title="创建一场岗位模拟" subtitle={`预计 ${Math.max(10, count * 3)} 分钟`}/>
      <div className="form-grid"><label className="full"><span>目标岗位</span><select value={selected.id} onChange={(event) => setSelectedApplicationId(event.target.value)}>{eligibleApplications.map((application) => <option key={application.id} value={application.id}>{application.company} · {application.role}</option>)}</select></label></div>
      <div className="question-source"><button className={experienceSelected ? "selected" : ""} disabled={!extractedLinks.length} onClick={() => setUseExperience((value) => !value)}><FileText/><span><strong>网页面经题</strong><small>{extractedLinks.length ? `${extractedLinks.length} 个已分析链接` : "请先导入并分析面经链接"}</small></span><i>{experienceSelected && <Check size={14}/>}</i></button><button className={useAi ? "selected" : ""} onClick={() => setUseAi((value) => !value)}><Sparkles/><span><strong>AI 简历出题</strong><small>根据该投递关联的简历生成</small></span><i>{useAi && <Check size={14}/>}</i></button></div>
      <div className="question-count"><label><span>问题数量</span><input type="number" min="1" max="30" value={count} onChange={(event) => setCount(Math.max(1, Math.min(30, Number(event.target.value) || 1)))}/><small>题</small></label><div>{[5, 8, 12, 15].map((value) => <button key={value} className={count === value ? "active" : ""} onClick={() => setCount(value)}>{value} 题</button>)}</div></div>
      <button className="button button--primary large" disabled={!experienceSelected && !useAi} onClick={start}><Play size={17}/>生成并开始 {count} 题模拟</button>
    </Card>
    <div className="mock-side"><Card><CardHeader title="本场题源"/><div className="source-summary"><span><FileText/><b>{extractedLinks.reduce((sum, item) => sum + item.questions.length, 0)}<small>道网页面经题</small></b></span><span><Sparkles/><b>AI<small>简历动态出题</small></b></span></div><button className="button button--secondary" onClick={() => navigate("/preparation")}>管理面经链接</button></Card><Card><CardHeader title="最近面试记录"/><div className="recent-mocks">{sessions.slice(0, 3).map((item) => <div key={item.id}><span className="stat-icon purple"><Mic2/></span><span><strong>{item.type} · {item.round}</strong><small>{item.questions.length} 题 · {item.createdAt}</small></span><Badge tone={item.status === "复盘完成" ? "green" : item.status === "待复盘" ? "orange" : "purple"}>{item.status}</Badge></div>)}</div></Card></div>
  </div>;
}
