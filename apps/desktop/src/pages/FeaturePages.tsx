import { useEffect, useState, type CSSProperties } from "react";
import { ArrowUp, BarChart3, CircleDollarSign, Clock3, Plus, Target } from "lucide-react";
import { useNavigate } from "react-router-dom";
import { Badge, Card, CardHeader, PageHeader } from "../components/ui";
import PreparationPage from "./interview/PreparationPage";
import MockInterviewPage from "./interview/MockInterviewPage";
import ReviewsPage from "./interview/ReviewsPage";
import QuestionBankPage from "./interview/QuestionBankPage";
import SettingsPage from "./SettingsPage";
import { getAnalytics, type AnalyticsData } from "../services/dashboard";
import { useInterviewFlow } from "../hooks/useInterviewFlow";
import { showError } from "../services/feedback";

type Kind = "preparation" | "mock" | "reviews" | "questions" | "offers" | "analytics" | "settings";

const meta: Record<Kind, { title: string; description: string }> = {
  preparation: { title: "面试准备", description: "针对每个岗位准备面经和模拟练习" },
  mock: { title: "模拟面试", description: "用面经题和简历题自由组合一场模拟练习" },
  reviews: { title: "面试复盘", description: "回顾每场面试的表现，持续进步" },
  questions: { title: "个人题库", description: "整理高频问题，打磨你的最佳回答" },
  offers: { title: "Offer 管理", description: "全面比较 Offer，做出最适合你的选择" },
  analytics: { title: "数据分析", description: "用数据看清求职进展和优化方向" },
  settings: { title: "设置", description: "管理简历、外部服务和数据安全偏好" },
};

export default function FeaturePages({ kind }: { kind: Kind }) {
  const page = meta[kind];
  const navigate = useNavigate();
  return <div className="page page-enter"><PageHeader title={page.title} description={page.description} action={kind==="offers"?<button className="button button--primary" onClick={() => navigate("/applications?new=1")}><Plus size={16}/>新增投递</button>:undefined}/>{kind==="preparation"&&<PreparationPage/>}{kind==="mock"&&<MockInterviewPage/>}{kind==="reviews"&&<ReviewsPage/>}{kind==="questions"&&<QuestionBankPage/>}{kind==="offers"&&<Offers/>}{kind==="analytics"&&<Analytics/>}{kind==="settings"&&<SettingsPage/>}</div>;
}

function Offers(){
  const navigate = useNavigate();
  const { applications } = useInterviewFlow();
  const active = applications.filter((item) => !item.archived);
  const offers = active.filter((item) => item.stage.toLowerCase().includes("offer") || item.stage.includes("谈薪"));
  const negotiating = offers.filter((item) => item.stage.includes("谈薪") || item.nextStep.includes("谈薪")).length;
  const rate = active.length ? offers.length / active.length * 100 : 0;
  return <><div className="offer-summary"><Card><span className="stat-icon teal"><CircleDollarSign/></span><span><small>Offer / 谈薪中</small><strong>{offers.length}</strong></span></Card><Card><span className="stat-icon orange"><Clock3/></span><span><small>正在谈薪</small><strong>{negotiating}</strong></span></Card><Card><span className="stat-icon blue"><BarChart3/></span><span><small>Offer 转化率</small><strong>{rate.toFixed(1)}%</strong></span></Card></div><Card className="table-card offer-compare"><CardHeader title="Offer 投递" subtitle="在投递详情更新阶段和下一步行动后会自动同步到这里"/><table><thead><tr><th>公司 / 岗位</th><th>地点</th><th>当前阶段</th><th>优先级</th><th>下一步</th><th>最近更新</th><th>操作</th></tr></thead><tbody>{offers.map((item)=><tr key={item.id} onDoubleClick={() => navigate(`/applications/${item.id}`)}><td><span className="company-logo">{item.companyMark}</span><span><strong>{item.company}</strong><small>{item.role}</small></span></td><td>{item.city}</td><td><Badge tone={item.stageTone}>{item.stage}</Badge></td><td>{item.priority}</td><td>{item.nextStep}<small>{item.nextTime}</small></td><td>{item.updated}</td><td><button className="application-detail-link" onClick={() => navigate(`/applications/${item.id}`)}>查看详情</button></td></tr>)}</tbody></table>{!offers.length&&<div className="question-bank-empty">还没有进入谈薪或 Offer 阶段的投递哦。祝你尽早取得心仪的 Offer！</div>}</Card></>;
}

const emptyAnalytics: AnalyticsData = { total: 0, thisMonth: 0, previousMonth: 0, assessments: 0, interviews: 0, offers: 0, averageFeedbackDays: null, daily: [], weekly: [], directions: [] };
const percent = (value: number, total: number) => total ? value / total * 100 : 0;
const displayPercent = (value: number) => `${value.toFixed(1)}%`;

function Analytics(){
  const [data,setData]=useState<AnalyticsData>(emptyAnalytics);
  const [loading,setLoading]=useState(true);
  const [error,setError]=useState("");
  useEffect(()=>{let disposed=false;setLoading(true);setError("");getAnalytics().then((value)=>{if(!disposed)setData(value)}).catch((reason)=>{if(!disposed)setError(String(reason))}).finally(()=>{if(!disposed)setLoading(false)});return()=>{disposed=true}},[]);
  useEffect(()=>{if(error&&!loading)showError(error,"数据分析读取失败")},[error,loading]);
  const monthDelta=data.thisMonth-data.previousMonth;
  const assessmentRate=percent(data.assessments,data.total), interviewRate=percent(data.interviews,data.total), offerRate=percent(data.offers,data.total);
  const stageRates:[[string,number,number],[string,number,number],[string,number,number]]=[["测评进入率",assessmentRate,assessmentRate],["面试进入率",percent(data.interviews,data.assessments),percent(data.interviews,data.assessments)],["Offer率",percent(data.offers,data.interviews),percent(data.offers,data.interviews)]];
  const maxDaily=Math.max(1,...data.daily.flatMap(item=>[item.applications,item.interviews]));
  const points=(key:"applications"|"interviews")=>data.daily.map((item,index)=>`${data.daily.length===1?210:index*420/(data.daily.length-1)},${105-item[key]/maxDaily*85}`).join(" ");
  const maxWeekly=Math.max(1,...data.weekly.flatMap(item=>[item.applications,item.interviews]));
  const topDirection=data.directions[0];
  const busiest=data.weekly.reduce<(typeof data.weekly)[number]|undefined>((best,item)=>!best||item.applications>best.applications?item:best,undefined);
  const funnel:[[string,number],[string,number],[string,number],[string,number]]=[["投递",data.total],["测评/笔试",data.assessments],["面试",data.interviews],["Offer",data.offers]];
  return <>{loading&&<div className="dashboard-loading">正在汇总本地投递数据…</div>}<div className="analytics-kpis">{[["总投递",String(data.total),`本月 ${monthDelta>=0?"+":""}${monthDelta}`],["面试进入率",displayPercent(interviewRate),`${data.interviews} 份进入面试`],["Offer 转化率",displayPercent(offerRate),`${data.offers} 份获得 Offer`],["平均反馈时间",data.averageFeedbackDays===null?"—":`${data.averageFeedbackDays.toFixed(1)} 天`,"基于首次流程反馈"]].map(([a,b,c])=><Card key={a}><small>{a}</small><strong>{b}</strong><span>{c}</span></Card>)}</div><div className="analytics-progress-grid"><Card className="progress-card"><CardHeader title="近 7 天进展"/><div className="trend-header"><strong>投递趋势</strong><span><i className="dot dot--blue"/>投递数 <i className="dot dot--green"/>面试数</span></div><div className="line-chart"><svg viewBox="0 0 420 115" preserveAspectRatio="none"><g className="grid-lines"><path d="M0 20H420M0 55H420M0 90H420"/></g><polyline className="line line-blue" points={points("applications")}/><polyline className="line line-green" points={points("interviews")}/></svg><div>{data.daily.map(value=><span key={value.label}>{value.label}</span>)}</div></div><div className="rate-row">{stageRates.map(([label,value,degree])=><div key={label}><i className="ring" style={{"--ring":`${Math.min(100,degree)}%`} as CSSProperties}/><span>{label}<strong>{displayPercent(value)}</strong><small>来自真实流程记录</small></span></div>)}</div></Card><Card className="funnel-card"><CardHeader title="投递转化漏斗"/><div className="funnel">{funnel.map(([label,count])=><div key={label}><span style={{width:`${Math.max(56,percent(count,data.total))}%`}}><span className="funnel-label">{label}</span><b>{count}</b></span><em>{displayPercent(percent(count,data.total))}</em></div>)}</div><div className="funnel-total"><span>整体转化率</span><strong>{displayPercent(offerRate)}</strong><b><ArrowUp size={12}/>{data.offers} 个</b></div></Card></div><div className="two-grid analytics-grid"><Card><CardHeader title="近 8 周投递与面试"/><div className="bar-chart">{data.weekly.map((item)=><div key={item.label} title={`${item.label}：投递 ${item.applications}，面试 ${item.interviews}`}><span style={{height:`${item.applications/maxWeekly*100}%`}}/><i style={{height:`${item.interviews/maxWeekly*100}%`}}/><small>{item.label}</small></div>)}</div></Card><Card><CardHeader title="岗位方向分布"/><div className="distribution">{data.directions.length?data.directions.map((item)=>{const rate=percent(item.count,data.total);return <div key={item.name}><span title={item.name}>{item.name}</span><i><b style={{width:`${rate}%`}}/></i><strong>{item.count} <small>({rate.toFixed(1)}%)</small></strong></div>}):<div className="analytics-empty">暂无岗位数据</div>}</div></Card></div><Card><CardHeader title="数据洞察"/><div className="insights"><span><TrendingIcon/><b>本月新增 {data.thisMonth} 份投递<small>{monthDelta===0?"与上月持平":`较上月${monthDelta>0?"增加":"减少"} ${Math.abs(monthDelta)} 份`}</small></b></span><span><Target/><b>{topDirection?`${topDirection.name}是主要方向`:"尚未形成主要岗位方向"}<small>{topDirection?`共 ${topDirection.count} 份，占 ${displayPercent(percent(topDirection.count,data.total))}`:"新增投递后会自动形成分布"}</small></b></span><span><TrendingIcon/><b>{busiest?`${busiest.label} 起的一周投递最多`:"暂无周趋势"}<small>{busiest?`该周新增 ${busiest.applications} 份投递，进入面试 ${busiest.interviews} 份`:"记录投递后即可查看趋势"}</small></b></span></div></Card></>}

function TrendingIcon(){return <span className="stat-icon green"><BarChart3/></span>}
