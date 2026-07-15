import { useEffect, useState, type CSSProperties } from "react";
import { ArrowUp, BarChart3, CircleDollarSign, Clock3, Plus, Star, Target } from "lucide-react";
import { Badge, Card, CardHeader, PageHeader } from "../components/ui";
import PreparationPage from "./interview/PreparationPage";
import MockInterviewPage from "./interview/MockInterviewPage";
import ReviewsPage from "./interview/ReviewsPage";
import SettingsPage from "./SettingsPage";
import { getAnalytics, type AnalyticsData } from "../services/dashboard";
import { hasLocalDatabase } from "../services/applications";

type Kind = "preparation" | "mock" | "reviews" | "questions" | "offers" | "analytics" | "settings";

const meta: Record<Kind, { title: string; description: string }> = {
  preparation: { title: "面试准备", description: "按每个流程中的岗位独立管理面经与训练入口" },
  mock: { title: "模拟面试", description: "组合导入面经与 AI 简历题，自由配置一场岗位模拟" },
  reviews: { title: "面试复盘", description: "承接真实面试与模拟训练，按岗位持续沉淀复盘" },
  questions: { title: "个人题库", description: "持续积累高频问题、最佳回答与 STAR 故事" },
  offers: { title: "Offer 管理", description: "客观比较选择维度，同时保留你的最终决定权" },
  analytics: { title: "数据分析", description: "从投递转化和面试表现中找到下一步行动" },
  settings: { title: "设置", description: "管理本地数据、外部服务、隐私权限与备份" },
};

export default function FeaturePages({ kind }: { kind: Kind }) {
  const page = meta[kind];
  return <div className="page page-enter"><PageHeader title={page.title} description={page.description} action={kind==="questions"||kind==="offers"?<button className="button button--primary"><Plus size={16}/>新建内容</button>:undefined}/>{kind==="preparation"&&<PreparationPage/>}{kind==="mock"&&<MockInterviewPage/>}{kind==="reviews"&&<ReviewsPage/>}{kind==="questions"&&<Questions/>}{kind==="offers"&&<Offers/>}{kind==="analytics"&&<Analytics/>}{kind==="settings"&&<SettingsPage/>}</div>;
}

function Questions(){const rows=[["如何保证分布式事务的一致性？","专业知识","5 次","熟悉"],["讲一个解决复杂问题的经历","行为面试","8 次","掌握"],["为什么选择我们公司？","岗位动机","6 次","待加强"],["项目中你最大的个人贡献是什么？","项目深挖","7 次","练习中"],["如何处理团队中的意见冲突？","行为面试","4 次","掌握"]];return <><div className="knowledge-tabs"><button className="active">问题库 <b>126</b></button><button>故事库 <b>18</b></button><button>薄弱项 <b>7</b></button></div><Card className="table-card"><table><thead><tr><th>问题</th><th>类型</th><th>出现次数</th><th>掌握程度</th><th>最近出现</th></tr></thead><tbody>{rows.map((r,i)=><tr key={r[0]}><td><strong>{r[0]}</strong><small>来源：{i%2?"真实面试":"面经 · 模拟面试"}</small></td><td><Badge tone={i%2?"purple":"blue"}>{r[1]}</Badge></td><td>{r[2]}</td><td><span className="mastery"><i style={{width:`${45+i*11}%`}}/></span>{r[3]}</td><td>{i+1} 天前</td></tr>)}</tbody></table></Card></>}

function Offers(){return <><div className="offer-summary"><Card><span className="stat-icon teal"><CircleDollarSign/></span><span><small>已收到 Offer</small><strong>3</strong></span></Card><Card><span className="stat-icon orange"><Clock3/></span><span><small>即将截止</small><strong>1</strong></span></Card><Card><span className="stat-icon blue"><Star/></span><span><small>当前首选</small><strong>蚂蚁集团</strong></span></Card></div><Card className="offer-compare"><CardHeader title="Offer 对比" action="调整权重"/><table><thead><tr><th>比较维度</th><th>权重</th><th>蚂蚁集团</th><th>腾讯科技</th><th>字节跳动</th></tr></thead><tbody>{[["综合得分","—","88","84","82"],["岗位匹配","25%","9.2","8.6","8.4"],["成长空间","20%","9.0","8.5","9.1"],["工作强度","15%","7.2","7.8","6.5"],["城市偏好","15%","9.0","8.0","7.5"],["薪资福利","25%","8.6","8.8","9.0"]].map((r,i)=><tr key={r[0]} className={i===0?"score-row":""}>{r.map((v,j)=><td key={j}>{v}</td>)}</tr>)}</tbody></table><p className="data-disclaimer">评分只根据你设置的权重汇总，最终选择由你决定。</p></Card></>}

const emptyAnalytics: AnalyticsData = { total: 0, thisMonth: 0, previousMonth: 0, assessments: 0, interviews: 0, offers: 0, averageFeedbackDays: null, daily: [], weekly: [], directions: [] };
const percent = (value: number, total: number) => total ? value / total * 100 : 0;
const displayPercent = (value: number) => `${value.toFixed(1)}%`;

function Analytics(){
  const [data,setData]=useState<AnalyticsData>(emptyAnalytics);
  const [loading,setLoading]=useState(true);
  const [error,setError]=useState("");
  useEffect(()=>{setLoading(true);getAnalytics().then(setData).catch((reason)=>setError(String(reason))).finally(()=>setLoading(false))},[]);
  const monthDelta=data.thisMonth-data.previousMonth;
  const assessmentRate=percent(data.assessments,data.total), interviewRate=percent(data.interviews,data.total), offerRate=percent(data.offers,data.total);
  const stageRates:[[string,number,number],[string,number,number],[string,number,number]]=[["测评进入率",assessmentRate,assessmentRate],["面试进入率",percent(data.interviews,data.assessments),percent(data.interviews,data.assessments)],["Offer率",percent(data.offers,data.interviews),percent(data.offers,data.interviews)]];
  const maxDaily=Math.max(1,...data.daily.flatMap(item=>[item.applications,item.interviews]));
  const points=(key:"applications"|"interviews")=>data.daily.map((item,index)=>`${data.daily.length===1?210:index*420/(data.daily.length-1)},${105-item[key]/maxDaily*85}`).join(" ");
  const maxWeekly=Math.max(1,...data.weekly.flatMap(item=>[item.applications,item.interviews]));
  const topDirection=data.directions[0];
  const busiest=data.weekly.reduce<(typeof data.weekly)[number]|undefined>((best,item)=>!best||item.applications>best.applications?item:best,undefined);
  const funnel:[[string,number],[string,number],[string,number],[string,number]]=[["投递",data.total],["测评/笔试",data.assessments],["面试",data.interviews],["Offer",data.offers]];
  return <>{loading&&<div className="dashboard-loading">正在汇总本地投递数据…</div>}{error&&<div className="detail-error">数据分析读取失败：{error}</div>}<div className="analytics-kpis">{[["总投递",String(data.total),`本月 ${monthDelta>=0?"+":""}${monthDelta}`],["面试进入率",displayPercent(interviewRate),`${data.interviews} 份进入面试`],["Offer 转化率",displayPercent(offerRate),`${data.offers} 份获得 Offer`],["平均反馈时间",data.averageFeedbackDays===null?"—":`${data.averageFeedbackDays.toFixed(1)} 天`,"基于首次流程反馈"]].map(([a,b,c])=><Card key={a}><small>{a}</small><strong>{b}</strong><span>{c}</span></Card>)}</div><div className="analytics-progress-grid"><Card className="progress-card"><CardHeader title="近 7 天进展"/><div className="trend-header"><strong>投递趋势</strong><span><i className="dot dot--blue"/>投递数 <i className="dot dot--green"/>面试数</span></div><div className="line-chart"><svg viewBox="0 0 420 115" preserveAspectRatio="none"><g className="grid-lines"><path d="M0 20H420M0 55H420M0 90H420"/></g><polyline className="line line-blue" points={points("applications")}/><polyline className="line line-green" points={points("interviews")}/></svg><div>{data.daily.map(value=><span key={value.label}>{value.label}</span>)}</div></div><div className="rate-row">{stageRates.map(([label,value,degree])=><div key={label}><i className="ring" style={{"--ring":`${Math.min(100,degree)}%`} as CSSProperties}/><span>{label}<strong>{displayPercent(value)}</strong><small>来自真实流程记录</small></span></div>)}</div></Card><Card className="funnel-card"><CardHeader title="投递转化漏斗"/><div className="funnel">{funnel.map(([label,count])=><div key={label}><span style={{width:`${Math.max(56,percent(count,data.total))}%`}}><span className="funnel-label">{label}</span><b>{count}</b></span><em>{displayPercent(percent(count,data.total))}</em></div>)}</div><div className="funnel-total"><span>整体转化率</span><strong>{displayPercent(offerRate)}</strong><b><ArrowUp size={12}/>{data.offers} 个</b></div></Card></div><div className="two-grid analytics-grid"><Card><CardHeader title="近 8 周投递与面试"/><div className="bar-chart">{data.weekly.map((item)=><div key={item.label} title={`${item.label}：投递 ${item.applications}，面试 ${item.interviews}`}><span style={{height:`${item.applications/maxWeekly*100}%`}}/><i style={{height:`${item.interviews/maxWeekly*100}%`}}/><small>{item.label}</small></div>)}</div></Card><Card><CardHeader title="岗位方向分布"/><div className="distribution">{data.directions.length?data.directions.map((item)=>{const rate=percent(item.count,data.total);return <div key={item.name}><span title={item.name}>{item.name}</span><i><b style={{width:`${rate}%`}}/></i><strong>{item.count} <small>({rate.toFixed(1)}%)</small></strong></div>}):<div className="analytics-empty">暂无岗位数据</div>}</div></Card></div><Card><CardHeader title="数据洞察"/><div className="insights"><span><TrendingIcon/><b>本月新增 {data.thisMonth} 份投递<small>{monthDelta===0?"与上月持平":`较上月${monthDelta>0?"增加":"减少"} ${Math.abs(monthDelta)} 份`}</small></b></span><span><Target/><b>{topDirection?`${topDirection.name}是主要方向`:"尚未形成主要岗位方向"}<small>{topDirection?`共 ${topDirection.count} 份，占 ${displayPercent(percent(topDirection.count,data.total))}`:"新增投递后会自动形成分布"}</small></b></span><span><TrendingIcon/><b>{busiest?`${busiest.label} 起的一周投递最多`:"暂无周趋势"}<small>{busiest?`该周新增 ${busiest.applications} 份投递，进入面试 ${busiest.interviews} 份`:"记录投递后即可查看趋势"}</small></b></span></div></Card></>}

function TrendingIcon(){return <span className="stat-icon green"><BarChart3/></span>}
