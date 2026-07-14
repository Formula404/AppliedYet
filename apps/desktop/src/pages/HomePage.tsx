import { useCallback, useEffect, useMemo, useState } from "react";
import { CalendarDays, Check, ChevronLeft, ChevronRight } from "lucide-react";
import { useNavigate } from "react-router-dom";
import { applications, calendarEvents, todayTasks } from "../data/mock";
import { Badge, Card, CardHeader } from "../components/ui";
import { getDashboard, type DashboardData, type DashboardEvent } from "../services/dashboard";
import { hasLocalDatabase, setApplicationTaskStatus } from "../services/applications";
import type { StatusTone } from "../types";

const weekdays = ["周一", "周二", "周三", "周四", "周五", "周六", "周日"];
const dateKey = (date: Date) => `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}-${String(date.getDate()).padStart(2, "0")}`;
const startOfDay = (date: Date) => new Date(date.getFullYear(), date.getMonth(), date.getDate());
const dateTime = (value: string) => new Date(value).toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit", hour12: false });
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

function mockDashboard(month: Date): DashboardData {
  const summary = {
    total: applications.length,
    active: applications.filter((item) => !item.stage.includes("拒绝") && !item.stage.toLowerCase().includes("offer")).length,
    assessments: applications.filter((item) => item.stage.includes("测评")).length,
    interviews: applications.filter((item) => item.stage.includes("面") || item.stage.includes("HR")).length,
    waiting: applications.filter((item) => item.stage.includes("等待")).length,
    offers: applications.filter((item) => item.stage.toLowerCase().includes("offer")).length,
    rejected: applications.filter((item) => item.stage.includes("拒绝")).length,
  };
  const today = startOfDay(new Date());
  const tasks = todayTasks.map((task) => ({
    id: task.id, applicationId: applications.find((item) => task.relation.includes(item.company))?.id || applications[0].id,
    title: task.title, company: task.relation.split(" · ")[0], role: task.relation.split(" · ")[1] || "个人任务",
    dueAt: new Date(today.getFullYear(), today.getMonth(), today.getDate(), ...task.time.split(":").map(Number) as [number, number]).toISOString(),
    priority: 2, status: "todo" as const, overdue: false, tone: task.tone,
  }));
  const events: DashboardEvent[] = month.getFullYear() === 2026 && month.getMonth() === 6
    ? Object.entries(calendarEvents).flatMap(([day, items]) => items.map((item, index) => ({
        id: `mock-${day}-${index}`, applicationId: applications.find((app) => item.label.includes(app.company))?.id || applications[0].id,
        title: item.label, company: item.label.split(" · ")[1] || "个人安排", role: "演示日程",
        scheduledAt: new Date(2026, 6, Number(day), 10).toISOString(), kind: "task" as const, tone: item.tone as StatusTone,
      }))) : [];
  return { summary, tasks, events };
}

export default function HomePage() {
  const navigate = useNavigate();
  const [today, setToday] = useState(() => startOfDay(new Date()));
  const [month, setMonth] = useState(() => new Date(today.getFullYear(), today.getMonth(), 1));
  const [selectedDate, setSelectedDate] = useState(today);
  const [data, setData] = useState<DashboardData>(() => hasLocalDatabase ? emptyDashboard() : mockDashboard(month));
  const [loading, setLoading] = useState(hasLocalDatabase);
  const [error, setError] = useState("");

  useEffect(() => {
    const timer = window.setInterval(() => {
      const current = startOfDay(new Date());
      setToday((previous) => dateKey(previous) === dateKey(current) ? previous : current);
    }, 30_000);
    return () => window.clearInterval(timer);
  }, []);

  const load = useCallback(async () => {
    if (!hasLocalDatabase) {
      setData(mockDashboard(month));
      return;
    }
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
    <div className="calendar-page-heading"><div><h1>日历</h1><p>集中查看面试、测评、沟通和复盘安排</p></div>{loading && <span className="dashboard-loading">正在同步本地数据…</span>}</div>
    {error && <div className="detail-error">首页数据读取失败：{error}</div>}
    <div className="home-kpis">{kpis.map(([label, value, tone, note]) => <button key={label} className={`home-kpi home-kpi--${tone}`} onClick={() => navigate("/applications")}><span>{label}</span><strong>{value}</strong><small>{note}</small></button>)}</div>
    <div className="calendar-layout">
      <Card className="calendar-card">
        <div className="calendar-head"><div><strong>{month.getFullYear()}年{month.getMonth() + 1}月</strong><button onClick={() => changeMonth(-1)} aria-label="上个月"><ChevronLeft size={17}/></button><button onClick={() => changeMonth(1)} aria-label="下个月"><ChevronRight size={17}/></button><button className="today-button" onClick={backToday}>今天</button></div><span>共 {data.events.length} 项日程</span></div>
        <div className="calendar-grid calendar-weekdays">{weekdays.map((day) => <span key={day}>{day}</span>)}</div>
        <div className="calendar-grid calendar-days" style={{ gridTemplateRows: `repeat(${cells.length / 7}, minmax(77px, 1fr))` }}>{cells.map(({ date, muted }) => {
          const key = dateKey(date); const isToday = key === dateKey(today); const selected = key === dateKey(selectedDate);
          return <button key={key} className={`${muted ? "muted" : ""} ${isToday ? "is-today" : ""} ${selected ? "selected" : ""}`} onClick={() => { setSelectedDate(date); if (muted) setMonth(new Date(date.getFullYear(), date.getMonth(), 1)); }}><span>{date.getDate()}</span>{eventsByDay[key]?.slice(0, 3).map((event) => <em className={`event event--${event.tone}`} key={event.id}>{event.title} · {event.company}</em>)}</button>;
        })}</div>
        <div className="calendar-legend">{[["orange", "测评/沟通"], ["purple", "面试/复盘"], ["teal", "Offer相关"], ["gray", "其他"]].map(([tone, label]) => <span key={label}><i className={`dot dot--${tone}`}/>{label}</span>)}</div>
      </Card>

      <div className="calendar-side">
        <Card className="selected-day-card"><CardHeader title={`${selectedDate.getMonth() + 1}月${selectedDate.getDate()}日`} subtitle={dateKey(selectedDate) === dateKey(today) ? "今天" : "选中日期"}/><div className="selected-day-events">{selectedEvents.length ? selectedEvents.map((event) => <button type="button" className="selected-day-event" key={event.id} onClick={() => navigate(`/applications/${event.applicationId}`)}><i className={`dot dot--${event.tone}`}/><span><strong>{event.title}</strong><small>{dateTime(event.scheduledAt)} · {event.company} · {event.role}</small></span><Badge tone={event.tone}>{event.kind === "task" ? "任务" : "下一步"}</Badge></button>) : <div className="calendar-empty"><CalendarDays size={24}/><p>当天没有安排</p></div>}</div></Card>
        <Card className="tasks-card"><CardHeader title="今日待办" subtitle={data.tasks.some((task) => task.overdue) ? "含逾期任务" : `${data.tasks.filter((task) => task.status !== "done").length} 项`}/><div className="task-list">{data.tasks.length ? data.tasks.map((task) => <div className={`task ${task.status === "done" ? "done" : ""}`} key={task.id}><button className="task-check" onClick={() => toggleTask(task)} aria-label={task.status === "done" ? "恢复为未完成" : "标记为已完成"}>{task.status === "done" && <Check size={11}/>}</button><button className="task-main" onClick={() => navigate(`/applications/${task.applicationId}`)}><strong>{task.title}</strong><small>{task.company} · {task.role}{task.overdue ? " · 已逾期" : task.status === "done" ? " · 已完成" : ""}</small></button><time>{dateTime(task.dueAt)}</time></div>) : <div className="calendar-empty"><Check size={22}/><p>今天没有待办任务</p></div>}</div></Card>
      </div>
    </div>
  </div>;
}
