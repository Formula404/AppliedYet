import type { CSSProperties } from "react";
import { ArrowUp, BarChart3, CircleDollarSign, Clock3, FileText, Plus, Star, Target } from "lucide-react";
import { Badge, Card, CardHeader, PageHeader } from "../components/ui";
import PreparationPage from "./interview/PreparationPage";
import MockInterviewPage from "./interview/MockInterviewPage";
import ReviewsPage from "./interview/ReviewsPage";
import SettingsPage from "./SettingsPage";

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

function Analytics(){return <><div className="analytics-kpis">{[["总投递","238","本月 +42"],["面试进入率","16.0%","较上月 +2.4%"],["Offer 转化率","2.9%","较上月 +0.6%"],["平均反馈时间","4.2 天","缩短 0.8 天"]].map(([a,b,c])=><Card key={a}><small>{a}</small><strong>{b}</strong><span>{c}</span></Card>)}</div><div className="analytics-progress-grid"><Card className="progress-card"><CardHeader title="本周进展"/><div className="trend-header"><strong>投递趋势</strong><span><i className="dot dot--blue"/>投递数 <i className="dot dot--green"/>面试数</span></div><div className="line-chart"><svg viewBox="0 0 420 115" preserveAspectRatio="none"><g className="grid-lines"><path d="M0 20H420M0 55H420M0 90H420"/></g><polyline className="line line-blue" points="0,82 60,55 120,51 180,40 240,67 300,49 360,42 420,24"/><polyline className="line line-green" points="0,98 60,88 120,80 180,69 240,91 300,73 360,71 420,83"/></svg><div>{["7/7","7/8","7/9","7/10","7/11","7/12","7/13"].map(value=><span key={value}>{value}</span>)}</div></div><div className="rate-row">{[["简历通过率","34.5",65],["面试通过率","36.2",67],["Offer率","18.4",42]].map(([label,value,degree])=><div key={label as string}><i className="ring" style={{"--ring":`${degree}%`} as CSSProperties}/><span>{label}<strong>{value}%</strong><small>较上周 <b>+{label==="简历通过率"?"5.2":"3.1"}%</b></small></span></div>)}</div></Card><Card className="funnel-card"><CardHeader title="投递转化漏斗"/><div className="funnel">{[["投递",238,"100%"],["测评/笔试",105,"44.1%"],["面试",38,"16.0%"],["Offer",7,"2.9%"]].map(([label,count,rate],index)=><div key={label as string}><span style={{width:`${100-index*14}%`}}>{label}<b>{count}</b></span><em>{rate}</em></div>)}</div><div className="funnel-total"><span>整体转化率</span><strong>2.9%</strong><b><ArrowUp size={12}/> 0.6%</b></div></Card></div><div className="two-grid analytics-grid"><Card><CardHeader title="近 8 周投递与面试"/><div className="bar-chart">{[42,68,50,78,61,88,72,95].map((h,i)=><div key={i}><span style={{height:`${h}%`}}/><i style={{height:`${h*.42}%`}}/><small>第{i+1}周</small></div>)}</div></Card><Card><CardHeader title="岗位方向分布"/><div className="distribution">{[["后端开发",50.4,120],["数据开发",17.6,42],["人工智能",11.8,28],["产品技术",7.6,18],["其他",12.6,30]].map(([n,p,c])=><div key={n as string}><span>{n}</span><i><b style={{width:`${p}%`}}/></i><strong>{c} <small>({p}%)</small></strong></div>)}</div></Card></div><Card><CardHeader title="行动建议"/><div className="insights"><span><TrendingIcon/><b>内推渠道的面试进入率最高<small>比招聘官网高 8.4 个百分点，可以优先寻找内推机会</small></b></span><span><Target/><b>系统设计题仍是高频薄弱项<small>最近 4 场面试中出现 3 次，建议安排专项训练</small></b></span><span><FileText/><b>“后端开发-2026.07”版本表现更好<small>仅表示统计关联，不代表由简历版本直接造成</small></b></span></div></Card></>}

function TrendingIcon(){return <span className="stat-icon green"><BarChart3/></span>}
