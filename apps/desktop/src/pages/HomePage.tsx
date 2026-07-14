import { useState } from "react";
import { CalendarDays, Check, ChevronLeft, ChevronRight } from "lucide-react";
import { calendarEvents, todayTasks } from "../data/mock";
import { Badge, Card, CardHeader } from "../components/ui";
import type { StatusTone } from "../types";

const weekdays = ["周一", "周二", "周三", "周四", "周五", "周六", "周日"];
const days = [
  ...Array(3).fill(0).map((_, index) => ({ n: 28 + index, muted: true })),
  ...Array(31).fill(0).map((_, index) => ({ n: index + 1, muted: false })),
  { n: 1, muted: true },
];

export default function HomePage() {
  const [completed, setCompleted] = useState<string[]>([]);
  const [selectedDay, setSelectedDay] = useState(13);
  const selectedEvents = calendarEvents[selectedDay] ?? [];

  const toggleTask = (id: string) => {
    setCompleted((current) => current.includes(id) ? current.filter((item) => item !== id) : [...current, id]);
  };

  return <div className="calendar-page page-enter">
    <div className="calendar-page-heading"><div><h1>日历</h1><p>集中查看面试、测评、沟通和复盘安排</p></div></div>
    <div className="calendar-layout">
      <Card className="calendar-card">
        <div className="calendar-head"><div><strong>2026年7月</strong><button><ChevronLeft size={17}/></button><button><ChevronRight size={17}/></button><button className="today-button" onClick={() => setSelectedDay(13)}>今天</button></div><span>共 12 项日程</span></div>
        <div className="calendar-grid calendar-weekdays">{weekdays.map((day) => <span key={day}>{day}</span>)}</div>
        <div className="calendar-grid calendar-days">{days.map((day, index) => <button key={index} className={`${day.muted ? "muted" : ""} ${day.n === 13 && !day.muted ? "is-today" : ""} ${day.n === selectedDay && !day.muted ? "selected" : ""}`} onClick={() => !day.muted && setSelectedDay(day.n)}><span>{day.n}</span>{!day.muted && calendarEvents[day.n]?.map((event, eventIndex) => <em className={`event event--${event.tone}`} key={eventIndex}>{event.label}</em>)}</button>)}</div>
        <div className="calendar-legend">{[["green", "测评/笔试"], ["blue", "面试"], ["orange", "HR/沟通"], ["teal", "Offer相关"], ["purple", "复盘任务"], ["gray", "其他"]].map(([tone, label]) => <span key={label}><i className={`dot dot--${tone}`}/>{label}</span>)}</div>
      </Card>

      <div className="calendar-side">
        <Card className="selected-day-card"><CardHeader title={`7月${selectedDay}日`} subtitle={selectedDay === 13 ? "今天" : "选中日期"}/><div className="selected-day-events">{selectedEvents.length ? selectedEvents.map((event) => <div key={event.label}><i className={`dot dot--${event.tone}`}/><span><strong>{event.label}</strong><small>时间待确认</small></span><Badge tone={event.tone as StatusTone}>日程</Badge></div>) : <div className="calendar-empty"><CalendarDays size={24}/><p>当天没有安排</p></div>}</div></Card>
        <Card className="tasks-card"><CardHeader title="今日待办"/><div className="task-list">{todayTasks.map((task) => <div className={completed.includes(task.id) ? "task done" : "task"} key={task.id}><button className={`task-check dot--${task.tone}`} onClick={() => toggleTask(task.id)}>{completed.includes(task.id) && <Check size={11}/>}</button><button className="task-main"><strong>{task.title}</strong><small>{task.relation}</small></button><time>{task.time}</time></div>)}</div></Card>
      </div>
    </div>
  </div>;
}
