import { useCallback, useEffect, useMemo, useState } from "react";
import { CalendarDays, Check, ChevronLeft, ChevronRight, CircleDot } from "lucide-react";
import { useNavigate } from "react-router-dom";
import { Badge, Card, CardHeader } from "../components/ui";
import { getDashboard, type DashboardData, type DashboardEvent } from "../services/dashboard";
import { hasLocalDatabase, setApplicationTaskStatus } from "../services/applications";
import type { StatusTone } from "../types";

const weekdays = ["周一", "周二", "周三", "周四", "周五", "周六", "周日"];
const systemTimeZone = Intl.DateTimeFormat().resolvedOptions().timeZone;
const dateKey = (date: Date) => `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}-${String(date.getDate()).padStart(2, "0")}`;
const startOfDay = (date: Date) => new Date(date.getFullYear(), date.getMonth(), date.getDate());
const dateTime = (value: string) => new Date(value).toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit", hour12: false, timeZone: systemTimeZone });
const eventKindLabel = (kind: DashboardEvent["kind"]) => kind === "task" ? "任务" : kind === "milestone" ? "流程节点" : "下一步";
const emptyDashboard = (): DashboardData => ({
  summary: { total: 0, active: 0, assessments: 0, interviews: 0, waiting: 0, offers: 0, rejected: 0 },
  tasks: [], events: [],
});

function buildCalendar(month: Date) {
  const first = new Date(month.getFullYear(), month.getMonth(), 1);
  const offset = (first.getDay() + 6) % 7;
  const count = new Date(month.getFullYear(), month.getMonth() + 1, 0).getDate();
  const cells = Math.ceil((offset + count) / 7) * 7;
  return Array.from({ length: cells }, (_, index) => {
    const date = new Date(month.getFullYear(), month.getMonth(), index - offset + 1);
    return { date, muted: date.getMonth() !== month.getMonth() };
  });
}

export default function HomePage() {
  const navigate = useNavigate();
  const [today, setToday] = useState(() => startOfDay(new Date()));
  const [month, setMonth] = useState(() => new Date(today.getFullYear(), today.getMonth(), 1));
  const [selectedDate, setSelectedDate] = useState(today);
  const [data, setData] = useState<DashboardData>(emptyDashboard);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  useEffect(() => {
    const timer = window.setInterval(() => {
      const current = startOfDay(new Date());
      setToday((previous) => dateKey(previous) === dateKey(current) ? previous : current);
    }, 30_000);
    return () => window.clearInterval(timer);
  }, []);

  const load = useCallback(async () => {
    setLoading(true);
    setError("");
    const monthStart = new Date(month.getFullYear(), month.getMonth(), 1);
    const monthEnd = new Date(month.getFullYear(), month.getMonth() + 1, 1);
    const todayEnd = new Date(today.getFullYear(), today.getMonth(), today.getDate() + 1);
    try {
      setData(await getDashboard(monthStart.toISOString(), monthEnd.toISOString(), today.toISOString(), todayEnd.toISOString()));
    } catch (reason) {
      setError(String(reason));
    } finally {
      setLoading(false);
    }
  }, [month, today]);

  useEffect(() => { load(); }, [load]);

  const cells = useMemo(() => buildCalendar(month), [month]);
  const eventsByDay = useMemo(() => data.events.reduce<Record<string, DashboardEvent[]>>((result, item) => {
    const key = dateKey(new Date(item.scheduledAt));
    (result[key] ||= []).push(item);
    return result;
  }, {}), [data.events]);
  const selectedEvents = eventsByDay[dateKey(selectedDate)] || [];
  const kpis: Array<[string, number, StatusTone, string]> = [
    ["累计投递", data.summary.total, "blue", "全部记录"], ["进行中", data.summary.active, "blue", "仍在流程"],
    ["待测评", data.summary.assessments, "orange", "测评/笔试"], ["面试中", data.summary.interviews, "purple", "各轮面试"],
    ["等待结果", data.summary.waiting, "gray", "等待反馈"], ["Offer", data.summary.offers, "teal", "已获 Offer"],
    ["已拒绝", data.summary.rejected, "red", "流程结束"],
  ];

  const changeMonth = (offset: number) => {
    const next = new Date(month.getFullYear(), month.getMonth() + offset, 1);
    setMonth(next);
    setSelectedDate(next);
  };
  const backToday = () => {
    setMonth(new Date(today.getFullYear(), today.getMonth(), 1));
    setSelectedDate(today);
  };
  const toggleTask = async (task: DashboardData["tasks"][number]) => {
    const nextStatus = task.status === "done" ? "todo" : "done";
    try {
      if (hasLocalDatabase) {
        await setApplicationTaskStatus(task.id, nextStatus);
        await load();
      } else {
        setData((current) => ({ ...current, tasks: current.tasks.map((item) => item.id === task.id ? { ...item, status: nextStatus } : item) }));
      }
    } catch (reason) { setError(String(reason)); }
  };

  return <div className="calendar-page page-enter">
    <div className="calendar-page-heading"><div><h1>日历</h1><p>集中查看待办任务、投递创建与流程变更</p></div>{loading && <span className="dashboard-loading">正在同步本地数据…</span>}</div>
    {error && <div className="detail-error">首页数据读取失败：{error}</div>}
    <div className="home-kpis">{kpis.map(([label, value, tone, note]) => <button key={label} className={`home-kpi home-kpi--${tone}`} onClick={() => navigate("/applications")}><span>{label}</span><strong>{value}</strong><small>{note}</small></button>)}</div>
    <div className="calendar-layout">
      <Card className="calendar-card">
        <div className="calendar-head"><div><strong>{month.getFullYear()}年{month.getMonth() + 1}月</strong><button onClick={() => changeMonth(-1)} aria-label="上个月"><ChevronLeft size={17}/></button><button onClick={() => changeMonth(1)} aria-label="下个月"><ChevronRight size={17}/></button><button className="today-button" onClick={backToday}>今天</button></div><span>{data.events.filter((item) => item.kind === "task").length} 项任务 · {data.events.filter((item) => item.kind === "milestone").length} 个流程节点</span></div>
        <div className="calendar-grid calendar-weekdays">{weekdays.map((day) => <span key={day}>{day}</span>)}</div>
        <div className="calendar-grid calendar-days" style={{ gridTemplateRows: `repeat(${cells.length / 7}, minmax(77px, 1fr))` }}>{cells.map(({ date, muted }) => {
          const key = dateKey(date); const isToday = key === dateKey(today); const selected = key === dateKey(selectedDate);
          return <button key={key} className={`${muted ? "muted" : ""} ${isToday ? "is-today" : ""} ${selected ? "selected" : ""}`} onClick={() => { setSelectedDate(date); if (muted) setMonth(new Date(date.getFullYear(), date.getMonth(), 1)); }}><span>{date.getDate()}</span>{eventsByDay[key]?.slice(0, 3).map((event) => <em className={`event event--${event.tone} event--type-${event.kind}`} key={event.id}>{event.title} · {event.company}</em>)}{(eventsByDay[key]?.length || 0) > 3 && <small className="calendar-more">+{eventsByDay[key].length - 3}</small>}</button>;
        })}</div>
        <div className="calendar-legend calendar-type-legend"><span><i className="calendar-type-mark calendar-type-mark--task"/>待办任务</span><span><i className="calendar-type-mark calendar-type-mark--milestone"/>流程节点</span><span><i className="calendar-type-mark calendar-type-mark--next"/>下一步安排</span></div>
      </Card>

      <div className="calendar-side">
        <Card className="selected-day-card"><CardHeader title={`${selectedDate.getMonth() + 1}月${selectedDate.getDate()}日`} subtitle={dateKey(selectedDate) === dateKey(today) ? "今天" : "选中日期"}/><div className="selected-day-events">{selectedEvents.length ? selectedEvents.map((event) => <button type="button" className={`selected-day-event selected-day-event--${event.kind}`} key={event.id} onClick={() => navigate(`/applications/${event.applicationId}`)}><span className="calendar-event-type-icon">{event.kind === "task" ? <Check size={14}/> : event.kind === "milestone" ? <CircleDot size={15}/> : <ChevronRight size={15}/>}</span><span><strong>{event.title}</strong><small>{event.title === "新增投递" ? "投递日期" : dateTime(event.scheduledAt)} · {event.company} · {event.role}</small></span><Badge tone={event.tone}>{eventKindLabel(event.kind)}</Badge></button>) : <div className="calendar-empty"><CalendarDays size={24}/><p>当天没有任务或流程记录</p></div>}</div></Card>
        <Card className="tasks-card"><CardHeader title="今日待办" subtitle={data.tasks.some((task) => task.overdue) ? "含逾期任务" : `${data.tasks.filter((task) => task.status !== "done").length} 项`}/><div className="task-list">{data.tasks.length ? data.tasks.map((task) => <div className={`task ${task.status === "done" ? "done" : ""}`} key={task.id}><button className="task-check" onClick={() => toggleTask(task)} aria-label={task.status === "done" ? "恢复为未完成" : "标记为已完成"}>{task.status === "done" && <Check size={11}/>}</button><button className="task-main" onClick={() => navigate(`/applications/${task.applicationId}`)}><strong>{task.title}</strong><small>{task.company} · {task.role}{task.overdue ? " · 已逾期" : task.status === "done" ? " · 已完成" : ""}</small></button><time>{dateTime(task.dueAt)}</time></div>) : <div className="calendar-empty"><Check size={22}/><p>今天没有待办任务</p></div>}</div></Card>
      </div>
    </div>
  </div>;
}
