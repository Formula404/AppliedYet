import { useState } from "react";
import { ArrowDown, ArrowUp, Check, ChevronLeft, ChevronRight, Clock3, Mail, MoreHorizontal, TrendingUp } from "lucide-react";
import { useNavigate } from "react-router-dom";
import { applications, calendarEvents, mails, todayTasks } from "../data/mock";
import { Badge, Card, CardHeader } from "../components/ui";

const kpis = [
  ["累计投递", "238", "+18", "blue"], ["进行中", "82", "34.5%", "blue"], ["待测评", "23", "9.7%", "orange"],
  ["待面试", "15", "6.3%", "purple"], ["Offer", "7", "2.9%", "green"], ["拒绝率", "61.3%", "-4.2%", "red"],
];
const weekdays = ["周一", "周二", "周三", "周四", "周五", "周六", "周日"];
const days = [...Array(3).fill(0).map((_, i) => ({ n: 28 + i, muted: true })), ...Array(31).fill(0).map((_, i) => ({ n: i + 1, muted: false })), { n: 1, muted: true }];

export default function HomePage() {
  const navigate = useNavigate();
  const [completed, setCompleted] = useState<string[]>([]);
  const [selectedDay, setSelectedDay] = useState(20);
  return <div className="dashboard page-enter">
    <div className="dashboard-top">
      <Card className="calendar-card">
        <div className="calendar-head"><div><strong>2026年7月</strong><button><ChevronLeft size={17}/></button><button><ChevronRight size={17}/></button><button className="today-button">今天</button></div><span>共 12 项日程</span></div>
        <div className="calendar-grid calendar-weekdays">{weekdays.map(d=><span key={d}>{d}</span>)}</div>
        <div className="calendar-grid calendar-days">{days.map((day,index)=><button key={index} className={`${day.muted ? "muted" : ""} ${day.n===13&&!day.muted ? "is-today" : ""} ${day.n===selectedDay&&!day.muted ? "selected" : ""}`} onClick={()=>!day.muted&&setSelectedDay(day.n)}><span>{day.n}</span>{!day.muted && calendarEvents[day.n]?.map((event,i)=><em className={`event event--${event.tone}`} key={i}>{event.label}</em>)}</button>)}</div>
        <div className="calendar-legend">{[["green","测评/笔试"],["blue","面试"],["orange","HR/沟通"],["teal","Offer相关"],["purple","复盘任务"],["gray","其他"]].map(([tone,label])=><span key={label}><i className={`dot dot--${tone}`}/>{label}</span>)}</div>
      </Card>
      <div className="dashboard-side">
        <div className="kpi-grid">{kpis.map(([label,value,delta,tone])=><button className={`kpi kpi--${tone}`} key={label} onClick={()=>navigate('/analytics')}><span>{label}</span><strong>{value}</strong><small>{label==="累计投递"?"较上周 ":""}<b className={delta.startsWith("-")?"good":""}>{delta}</b></small></button>)}</div>
        <div className="side-row">
          <Card className="funnel-card"><CardHeader title="投递转化漏斗" /><div className="funnel">{[["投递",238,"100%"],["测评/笔试",105,"44.1%"],["面试",38,"16.0%"],["Offer",7,"2.9%"]].map(([label,count,rate],i)=><button key={label} onClick={()=>navigate('/applications')}><span style={{width:`${100-i*14}%`}}>{label}<b>{count}</b></span><em>{rate}</em></button>)}</div><div className="funnel-total"><span>整体转化率</span><strong>2.9%</strong><b><ArrowUp size={12}/> 0.6%</b></div></Card>
          <Card className="tasks-card"><CardHeader title="今日待办" action="查看全部" /><div className="task-list">{todayTasks.map(task=><div className={completed.includes(task.id)?"task done":"task"} key={task.id}><button className={`task-check dot--${task.tone}`} onClick={()=>setCompleted([...completed,task.id])}>{completed.includes(task.id)&&<Check size={11}/>}</button><button className="task-main"><strong>{task.title}</strong><small>{task.relation}</small></button><time>{task.time}</time></div>)}</div></Card>
        </div>
      </div>
    </div>
    <div className="dashboard-bottom">
      <Card className="mail-card"><CardHeader title="最近招聘邮件" action="查看全部" /><div>{mails.slice(0,4).map((mail,i)=><button className="mail-row" key={mail.id} onClick={()=>navigate('/emails')}><span className={`company-logo logo-${i}`}>{mail.company[0]}</span><span><strong>{mail.company} · {mail.role}</strong><small>{mail.summary}</small></span><Badge tone={mail.status==="待确认"?"orange":"gray"}>{mail.type}</Badge><time>{mail.time}</time></button>)}</div></Card>
      <Card className="progress-card"><CardHeader title="本周进展" action="数据概览" /><div className="trend-header"><strong>投递趋势</strong><span><i className="dot dot--blue"/>投递数 <i className="dot dot--green"/>面试数</span></div><div className="line-chart"><svg viewBox="0 0 420 115" preserveAspectRatio="none"><g className="grid-lines"><path d="M0 20H420M0 55H420M0 90H420"/></g><polyline className="line line-blue" points="0,82 60,55 120,51 180,40 240,67 300,49 360,42 420,24"/><polyline className="line line-green" points="0,98 60,88 120,80 180,69 240,91 300,73 360,71 420,83"/></svg><div>{["7/7","7/8","7/9","7/10","7/11","7/12","7/13"].map(x=><span key={x}>{x}</span>)}</div></div><div className="rate-row">{[["简历通过率","34.5",65],["面试通过率","36.2",67],["Offer率","18.4",42]].map(([label,value,deg])=><div key={label as string}><i className="ring" style={{"--ring":`${deg}%`} as React.CSSProperties}/><span>{label}<strong>{value}%</strong><small>较上周 <b>+{label==="简历通过率"?"5.2":"3.1"}%</b></small></span></div>)}</div></Card>
      <Card className="focus-card"><CardHeader title="重点岗位" action="查看全部" /><div>{applications.slice(0,4).map(app=><button className="focus-row" key={app.id} onClick={()=>navigate('/applications')}><span className="company-logo">{app.companyMark}</span><span><strong>{app.company}</strong><small>{app.role} · {app.city}</small><i>投递　→　测评　→　<b>{app.stage}</b>　→　Offer</i></span><Badge tone={app.stageTone}>{app.stage}</Badge><time>{app.nextTime}</time></button>)}</div></Card>
    </div>
  </div>;
}
