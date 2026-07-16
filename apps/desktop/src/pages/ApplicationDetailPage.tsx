import { useCallback, useEffect, useRef, useState, type FormEvent } from "react";
import {
  Archive, ArrowLeft, Bell, BriefcaseBusiness, CalendarClock, Check, Circle, Clock3, ExternalLink,
  FileText, History, MapPin, Pencil, Plus, RotateCcw, Trash2, X,
} from "lucide-react";
import { useNavigate, useParams } from "react-router-dom";
import { Badge, Card } from "../components/ui";
import { useInterviewFlow } from "../hooks/useInterviewFlow";
import {
  createApplicationEvent, createApplicationTask, deleteApplicationTask, getApplicationDetail, hasLocalDatabase,
  revertApplicationEvent, setApplicationTaskStatus, updateApplicationDetail, updateApplicationTask,
  updateApplicationEventTime,
  type CreateEventInput, type CreateTaskInput, type UpdateApplicationDetailInput,
} from "../services/applications";
import type { ApplicationDetail, ApplicationEvent, ApplicationTask, StatusTone } from "../types";
import { listResumeProfiles, type ResumeProfile } from "../services/resumes";
import { openExternalUrl } from "../services/external";

const stages = ["准备投递", "已投递", "在线测评", "笔试", "AI 面试", "HR 面试", "业务面试", "专业面试", "终面", "谈薪", "等待结果", "已获Offer", "进入人才库", "流程暂停", "流程结束", "主动放弃", "已拒绝"];
const optional = (value: FormDataEntryValue | null) => String(value || "").trim() || undefined;
const systemTimeZone = Intl.DateTimeFormat().resolvedOptions().timeZone;
const dateText = (value?: string) => {
  if (!value) return "未设置";
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString("zh-CN", { dateStyle: "medium", timeStyle: "short", timeZone: systemTimeZone });
};
const toUtc = (value?: string) => {
  if (!value) return undefined;
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? undefined : date.toISOString();
};
const safeExternalUrl = (value?: string) => {
  if (!value) return undefined;
  try {
    const url = new URL(value);
    return ["http:", "https:"].includes(url.protocol) ? url.href : undefined;
  } catch {
    return undefined;
  }
};
const toLocalInput = (value?: string) => {
  if (!value) return undefined;
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value.slice(0, 16);
  return new Date(date.getTime() - date.getTimezoneOffset() * 60_000).toISOString().slice(0, 16);
};
const stageTone = (stage: string): StatusTone => stage.includes("拒绝") ? "red" : stage.toLowerCase().includes("offer") ? "teal" : stage.includes("面") || stage.includes("HR") || stage.includes("复盘") ? "purple" : stage.includes("测评") || stage.includes("笔试") || stage.includes("沟通") || stage.includes("谈薪") ? "orange" : stage.includes("等待") ? "gray" : "blue";
const priorityText = (value: number) => value === 3 ? "高" : value === 2 ? "中" : "普通";
const sourceText: Record<string, string> = { manual: "手动", email: "邮件", ai: "AI", system: "系统" };
const reminderOptions = [
  { value: "5", label: "提前 5 分钟" }, { value: "15", label: "提前 15 分钟" },
  { value: "60", label: "提前 1 小时" }, { value: "1440", label: "提前 1 天" },
  { value: "4320", label: "提前 3 天" },
];
const reminderText = (dueAt?: string, remindAt?: string) => {
  if (!dueAt || !remindAt) return "";
  const minutes = Math.round((new Date(dueAt).getTime() - new Date(remindAt).getTime()) / 60_000);
  return reminderOptions.find((item) => Number(item.value) === minutes)?.label || `提前 ${minutes} 分钟`;
};
const reminderOffsetValue = (dueAt?: string, remindAt?: string) => {
  if (!dueAt || !remindAt) return "";
  const minutes = Math.round((new Date(dueAt).getTime() - new Date(remindAt).getTime()) / 60_000);
  return Number.isFinite(minutes) && minutes > 0 ? String(minutes) : "";
};

function mockDetail(id: string, applications: ReturnType<typeof useInterviewFlow>["applications"]): ApplicationDetail | null {
  const app = applications.find((item) => item.id === id);
  if (!app) return null;
  const now = new Date();
  const at = (offset: number, hour = 10) => { const value = new Date(now); value.setDate(value.getDate() + offset); value.setHours(hour, 0, 0, 0); return value.toISOString(); };
  return {
    id, companyName: app.company, companyShortName: app.company, industry: "互联网 / 软件服务", companyType: "大型科技企业",
    website: "https://www.example.com", companyNotes: "重点关注技术成长、团队方向与工作地点；面试后及时补记问题和反馈。",
    positionTitle: app.role, department: "核心业务技术部", location: app.city, recruitmentType: "校招", jobCode: `DEMO-${id.toUpperCase()}-2026`,
    sourceUrl: "https://jobs.example.com/campus/demo", channel: "招聘官网", appliedAt: at(-12).slice(0, 10),
    jdRaw: "岗位职责\n1. 参与核心业务服务的设计、开发与稳定性建设；\n2. 负责高并发场景下的性能优化与故障治理；\n3. 与产品、测试和上下游团队协作推进项目落地。\n\n任职要求\n熟悉 Java、数据库、缓存与消息队列；具备良好的工程意识和问题分析能力。",
    priority: app.priority === "高" ? 3 : app.priority === "中" ? 2 : 1, currentStage: app.stage, nextAction: app.nextStep, nextActionDueAt: at(1, 14),
    resumeProfileId: app.resumeProfileId, resumeName: app.resumeName, resumeFileFormat: "pdf", resumeTargetDirection: id === "jd" ? "数据开发 / 实时计算" : "Java 后端 / 平台研发",
    createdAt: at(-12), updatedAt: at(0), archivedAt: app.archived ? at(0) : undefined,
    tasks: [
      { id: `task-${id}-1`, title: app.nextStep || "准备下一轮流程", description: "梳理岗位重点，并准备两个可量化的项目案例。", priority: 3, status: "doing", dueAt: at(1, 14), remindAt: at(1, 13), applicationStage: app.stage, sourceType: "manual", createdAt: at(-2) },
      { id: `task-${id}-2`, title: "完成公司与业务调研", description: "整理业务模式、核心产品与近期技术动态。", priority: 2, status: "done", dueAt: at(-1, 20), applicationStage: app.stage, sourceType: "manual", completedAt: at(-1, 19), createdAt: at(-5) },
    ],
    events: [
      { id: `event-${id}-stage`, eventType: "stage_changed", title: `流程更新为“${app.stage}”`, content: "由招聘邮件识别并经人工确认。", sourceType: "email", stageBefore: "已投递", stageAfter: app.stage, happenedAt: at(-2, 16), reversible: true },
      { id: `event-${id}-note`, eventType: "manual_note", title: "完成岗位信息整理", content: "已补充 JD、岗位方向和面试准备重点。", sourceType: "manual", happenedAt: at(-5, 21), reversible: false },
      { id: `mock-${id}`, eventType: "application_created", title: "创建投递", sourceType: "manual", happenedAt: at(-12, 11), reversible: false },
    ],
  };
}

export default function ApplicationDetailPage() {
  const { id = "" } = useParams();
  const navigate = useNavigate();
  const { applications, archiveApplication, refreshApplications, updateApplicationStage } = useInterviewFlow();
  const [detail, setDetail] = useState<ApplicationDetail | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [editing, setEditing] = useState(false);
  const [addingTask, setAddingTask] = useState(false);
  const [addingEvent, setAddingEvent] = useState(false);
  const [editingTask, setEditingTask] = useState<ApplicationTask | null>(null);
  const [editingEventTime, setEditingEventTime] = useState<ApplicationEvent | null>(null);
  const [saving, setSaving] = useState(false);
  const [changingStage, setChangingStage] = useState(false);
  const [taskError, setTaskError] = useState("");
  const [eventError, setEventError] = useState("");
  const [resumes, setResumes] = useState<ResumeProfile[]>([]);
  const loadRequest = useRef(0);

  const load = useCallback(async () => {
    const request = ++loadRequest.current;
    setLoading(true);
    setError("");
    try {
      const value = hasLocalDatabase ? await getApplicationDetail(id) : mockDetail(id, applications);
      if (request !== loadRequest.current) return;
      if (!value) throw new Error("投递记录不存在");
      setDetail(value);
    } catch (reason) {
      if (request === loadRequest.current) setError(String(reason));
    } finally {
      if (request === loadRequest.current) setLoading(false);
    }
  }, [applications, id]);

  useEffect(() => { load(); }, [load]);
  useEffect(() => () => { loadRequest.current += 1; }, []);
  useEffect(() => { listResumeProfiles().then(setResumes).catch((reason) => setError(String(reason))); }, []);

  const saveDetail = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!detail) return;
    const data = new FormData(event.currentTarget);
    const input: UpdateApplicationDetailInput = {
      companyName: String(data.get("companyName") || ""), companyShortName: optional(data.get("companyShortName")),
      industry: optional(data.get("industry")), companyType: optional(data.get("companyType")), website: optional(data.get("website")), companyNotes: optional(data.get("companyNotes")),
      positionTitle: String(data.get("positionTitle") || ""), department: optional(data.get("department")), location: optional(data.get("location")),
      recruitmentType: optional(data.get("recruitmentType")), jobCode: optional(data.get("jobCode")), sourceUrl: optional(data.get("sourceUrl")), jdRaw: optional(data.get("jdRaw")),
      appliedAt: optional(data.get("appliedAt")), channel: optional(data.get("channel")), priority: Number(data.get("priority")),
      currentStage: detail.currentStage, nextAction: optional(data.get("nextAction")), nextActionDueAt: toUtc(optional(data.get("nextActionDueAt"))),
      resumeProfileId: optional(data.get("resumeProfileId")),
    };
    setSaving(true);
    setError("");
    try {
      if (hasLocalDatabase) {
        setDetail(await updateApplicationDetail(id, input));
        await refreshApplications();
      } else {
        const changed = detail.currentStage !== input.currentStage;
        setDetail({ ...detail, ...input, updatedAt: new Date().toISOString(), events: [{
          id: `event-${Date.now()}`, eventType: changed ? "stage_changed" : "detail_updated",
          title: changed ? "更新投递阶段" : "更新岗位与投递资料", content: changed ? `${detail.currentStage} → ${input.currentStage}` : undefined,
          sourceType: "manual", stageBefore: changed ? detail.currentStage : undefined, stageAfter: changed ? input.currentStage : undefined,
          happenedAt: new Date().toISOString(), reversible: changed,
        }, ...detail.events] });
      }
      setEditing(false);
    } catch (reason) { setError(String(reason)); } finally { setSaving(false); }
  };

  const addTask = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!detail) return;
    setTaskError("");
    const data = new FormData(event.currentTarget);
    const dueAt = toUtc(optional(data.get("dueAt")));
    const reminderOffset = optional(data.get("reminderOffset"));
    if (reminderOffset && !dueAt) {
      setTaskError("设置提醒前请先选择任务截止时间");
      return;
    }
    const remindAt = dueAt && reminderOffset
      ? new Date(new Date(dueAt).getTime() - Number(reminderOffset) * 60_000).toISOString()
      : undefined;
    const input: CreateTaskInput = { title: String(data.get("title") || ""), description: optional(data.get("description")), priority: Number(data.get("priority")), dueAt, remindAt, applicationStage: optional(data.get("applicationStage")) };
    setSaving(true);
    try {
      if (hasLocalDatabase) {
        await createApplicationTask(id, input);
        await load();
      } else {
        const task: ApplicationTask = { id: `task-${Date.now()}`, ...input, status: "todo", sourceType: "manual", createdAt: new Date().toISOString() };
        const timeline: ApplicationEvent = { id: `event-${Date.now()}`, eventType: "task_created", title: "新增任务", content: task.title, sourceType: "manual", happenedAt: new Date().toISOString(), reversible: false };
        setDetail({ ...detail, tasks: [task, ...detail.tasks], events: [timeline, ...detail.events] });
      }
      setAddingTask(false);
    } catch (reason) { setTaskError(String(reason)); } finally { setSaving(false); }
  };

  const toggleTask = async (task: ApplicationTask) => {
    if (!detail) return;
    const status = task.status === "done" ? "todo" : "done";
    try {
      if (hasLocalDatabase) {
        await setApplicationTaskStatus(task.id, status);
        await load();
      } else {
        setDetail({ ...detail, tasks: detail.tasks.map((item) => item.id === task.id ? { ...item, status, completedAt: status === "done" ? new Date().toISOString() : undefined } : item) });
      }
    } catch (reason) { setError(String(reason)); }
  };

  const saveTaskEdit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!editingTask) return;
    setTaskError("");
    const data = new FormData(event.currentTarget);
    const dueAt = toUtc(optional(data.get("dueAt")));
    const reminderOffset = optional(data.get("reminderOffset"));
    if (reminderOffset && !dueAt) {
      setTaskError("设置提醒前请先选择任务截止时间");
      return;
    }
    const input: CreateTaskInput = {
      title: String(data.get("title") || ""), description: optional(data.get("description")), priority: Number(data.get("priority")), dueAt,
      remindAt: dueAt && reminderOffset ? new Date(new Date(dueAt).getTime() - Number(reminderOffset) * 60_000).toISOString() : undefined,
      applicationStage: optional(data.get("applicationStage")),
    };
    setSaving(true);
    try {
      if (hasLocalDatabase) {
        await updateApplicationTask(editingTask.id, input);
        await load();
      } else {
        setDetail((current) => current ? { ...current, tasks: current.tasks.map((task) => task.id === editingTask.id ? { ...task, ...input } : task) } : current);
      }
      setEditingTask(null);
    } catch (reason) { setTaskError(String(reason)); } finally { setSaving(false); }
  };

  const removeTask = async (task: ApplicationTask) => {
    if (!window.confirm(`确定删除任务“${task.title}”吗？`)) return;
    try {
      if (hasLocalDatabase) {
        await deleteApplicationTask(task.id);
        await load();
      } else {
        setDetail((current) => current ? { ...current, tasks: current.tasks.filter((item) => item.id !== task.id) } : current);
      }
    } catch (reason) { setError(String(reason)); }
  };

  const toggleArchived = async () => {
    const archived = !detail?.archivedAt;
    if (!detail || (archived && !window.confirm("归档后该投递将退出看板、首页统计和任务提醒。确定继续吗？"))) return;
    try {
      await archiveApplication(id, archived);
      if (hasLocalDatabase) await load();
      else setDetail({ ...detail, archivedAt: archived ? new Date().toISOString() : undefined });
    } catch (reason) { setError(String(reason)); }
  };

  const changeStage = async (stage: string) => {
    if (!detail || stage === detail.currentStage) return;
    setChangingStage(true);
    setError("");
    try {
      await updateApplicationStage(id, stage, stageTone(stage));
      if (hasLocalDatabase) await load();
      else setDetail({ ...detail, currentStage: stage, updatedAt: new Date().toISOString() });
    } catch (reason) {
      setError(String(reason));
    } finally {
      setChangingStage(false);
    }
  };

  const undoEvent = async (item: ApplicationEvent) => {
    if (!window.confirm(`撤销“${item.stageBefore} → ${item.stageAfter}”这次阶段变更吗？`)) return;
    const scrollPosition = window.scrollY;
    try {
      if (hasLocalDatabase) {
        setDetail(await revertApplicationEvent(item.id));
        await refreshApplications();
      } else if (detail && item.stageBefore) {
        setDetail({ ...detail, currentStage: item.stageBefore, events: detail.events.map((event) => event.id === item.id ? { ...event, revertedAt: new Date().toISOString() } : event) });
      }
      requestAnimationFrame(() => window.scrollTo({ top: scrollPosition, behavior: "instant" }));
    } catch (reason) { setError(String(reason)); }
  };

  const addEvent = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!detail) return;
    const data = new FormData(event.currentTarget);
    const input: CreateEventInput = { title: String(data.get("title") || ""), content: optional(data.get("content")), happenedAt: toUtc(optional(data.get("happenedAt"))) };
    setSaving(true);
    try {
      if (hasLocalDatabase) {
        await createApplicationEvent(id, input);
        await load();
      } else {
        const item: ApplicationEvent = { id: `event-${Date.now()}`, eventType: "manual_note", title: input.title, content: input.content, sourceType: "manual", happenedAt: input.happenedAt || new Date().toISOString(), reversible: false };
        setDetail({ ...detail, events: [item, ...detail.events] });
      }
      setAddingEvent(false);
    } catch (reason) { setError(String(reason)); } finally { setSaving(false); }
  };

  const saveEventTime = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!detail || !editingEventTime) return;
    const data = new FormData(event.currentTarget);
    const happenedAt = toUtc(optional(data.get("happenedAt")));
    if (!happenedAt) { setEventError("请选择状态实际发生时间"); return; }
    setSaving(true);
    setEventError("");
    try {
      if (hasLocalDatabase) {
        setDetail(await updateApplicationEventTime(editingEventTime.id, happenedAt));
      } else {
        setDetail({ ...detail, events: detail.events.map((item) => item.id === editingEventTime.id ? { ...item, happenedAt, updatedAt: new Date().toISOString() } : item) });
      }
      setEditingEventTime(null);
    } catch (reason) { setEventError(String(reason)); } finally { setSaving(false); }
  };

  if (loading) return <div className="page page-enter"><div className="detail-loading">正在读取岗位详情…</div></div>;
  if (!detail) return <div className="page page-enter"><button className="text-button" onClick={() => navigate("/applications")}><ArrowLeft size={15} />返回投递</button><div className="detail-error">{error || "投递记录不存在"}</div></div>;
  const websiteUrl = safeExternalUrl(detail.website);
  const sourceUrl = safeExternalUrl(detail.sourceUrl);

  return <div className="page page-enter application-detail-page">
    <div className="application-detail-header">
      <button className="detail-back" onClick={() => navigate("/applications")}><ArrowLeft size={17} />返回投递</button>
      <div className="detail-company-mark">{Array.from(detail.companyName)[0] ?? "?"}</div>
      <div><div className="detail-title-line"><h1>{detail.companyName} · {detail.positionTitle}</h1><Badge tone={stageTone(detail.currentStage)}>{detail.currentStage}</Badge><select className="detail-stage-select" aria-label="更新投递阶段" value={detail.currentStage} disabled={changingStage || Boolean(detail.archivedAt)} onChange={(event) => void changeStage(event.target.value)}>{!stages.includes(detail.currentStage) && <option>{detail.currentStage}</option>}{stages.map((stage) => <option key={stage}>{stage}</option>)}</select></div><p><MapPin size={14} />{detail.location || "地点未填写"}<span>·</span>{detail.department || "部门未填写"}<span>·</span>{priorityText(detail.priority)}优先级</p></div>
      <div className="detail-header-actions"><button className="button button--secondary" onClick={toggleArchived}>{detail.archivedAt ? <RotateCcw size={15} /> : <Archive size={15} />}{detail.archivedAt ? "恢复投递" : "归档投递"}</button><button className="button button--primary" onClick={() => setEditing(true)}><Pencil size={15} />编辑资料</button></div>
    </div>
    {error && <div className="detail-error">{error}</div>}

    <div className="application-detail-grid">
      <div className="application-detail-main">
        <Card className="detail-section-card">
          <div className="section-heading"><div><BriefcaseBusiness size={18} /><span><h2>岗位与投递</h2><small>核心信息与下一步行动</small></span></div></div>
          <dl className="detail-facts">
            <div><dt>投递渠道</dt><dd>{detail.channel || "未填写"}</dd></div><div><dt>投递日期</dt><dd>{detail.appliedAt || "未填写"}</dd></div>
            <div><dt>招聘类型</dt><dd>{detail.recruitmentType || "未填写"}</dd></div><div><dt>岗位编号</dt><dd>{detail.jobCode || "未填写"}</dd></div>
            <div><dt>行业 / 性质</dt><dd>{[detail.industry, detail.companyType].filter(Boolean).join(" · ") || "未填写"}</dd></div><div><dt>下一步</dt><dd>{detail.nextAction || "待安排"}</dd></div>
            <div><dt>下一步时间</dt><dd>{dateText(detail.nextActionDueAt)}</dd></div><div><dt>最近更新</dt><dd>{dateText(detail.updatedAt)}</dd></div>
          </dl>
          {(websiteUrl || sourceUrl) && <div className="detail-links">{websiteUrl && <a href={websiteUrl} onClick={(event) => { event.preventDefault(); void openExternalUrl(websiteUrl).catch((reason) => setError(String(reason))); }}>公司官网 <ExternalLink size={13} /></a>}{sourceUrl && <a href={sourceUrl} onClick={(event) => { event.preventDefault(); void openExternalUrl(sourceUrl).catch((reason) => setError(String(reason))); }}>招聘链接 <ExternalLink size={13} /></a>}</div>}
        </Card>

        <Card className="detail-section-card">
          <div className="section-heading"><div><FileText size={18} /><span><h2>关联简历</h2><small>关联的简历会用于 AI 面试准备和出题</small></span></div><button className="button button--secondary" onClick={() => setEditing(true)}><Pencil size={14} />更换</button></div>
          {detail.resumeProfileId ? <dl className="detail-facts"><div><dt>版本名称</dt><dd>{detail.resumeName || "历史简历"}</dd></div><div><dt>文件格式</dt><dd>{detail.resumeFileFormat?.toUpperCase() || "结构化资料"}</dd></div><div><dt>适用方向</dt><dd>{detail.resumeTargetDirection || "未设置"}</dd></div><div><dt>版本状态</dt><dd>{resumes.find((item) => item.id === detail.resumeProfileId)?.archivedAt ? "已归档（历史引用）" : "使用中"}</dd></div></dl> : <div className="detail-empty">尚未关联简历。关联后，AI 岗位准备和模拟面试会使用该版本。</div>}
        </Card>

        <Card className="detail-section-card">
          <div className="section-heading"><div><FileText size={18} /><span><h2>岗位描述</h2><small>填写后可获得更精准的 AI 面试准备建议</small></span></div></div>
          <div className={detail.jdRaw ? "jd-content" : "detail-empty"}>{detail.jdRaw || "尚未录入 JD 原文，可通过“编辑资料”补充。"}</div>
        </Card>

        <Card className="detail-section-card">
          <div className="section-heading"><div><History size={18} /><span><h2>事件时间线</h2><small>{detail.events.filter((item) => !item.revertedAt && item.eventType !== "event_reverted").length} 条记录</small></span></div><button type="button" className="button button--secondary" onClick={() => setAddingEvent(true)}><Plus size={14} />补记事件</button></div>
          <div className="event-timeline">{detail.events.some((item) => !item.revertedAt && item.eventType !== "event_reverted") ? detail.events.filter((item) => !item.revertedAt && item.eventType !== "event_reverted").map((item) => <div className="event-item" key={item.id}><span className={`event-dot event-dot--${item.sourceType}`} /><div><div className="event-title"><strong>{item.title}</strong><Badge tone="gray">{sourceText[item.sourceType] || item.sourceType}</Badge><time>{dateText(item.happenedAt)}</time>{item.eventType === "stage_changed" && <button type="button" className="event-time-edit" onClick={() => { setEventError(""); setEditingEventTime(item); }} aria-label="修改状态发生时间"><Pencil size={12} />时间</button>}{item.reversible && <button type="button" className="event-undo" onClick={() => undoEvent(item)}><RotateCcw size={12} />撤销</button>}</div>{item.content && <p>{item.content}</p>}{item.stageBefore && item.stageAfter && <div className="stage-change"><span>{item.stageBefore}</span><b>→</b><span>{item.stageAfter}</span></div>}</div></div>) : <div className="detail-empty">暂无事件记录</div>}</div>
        </Card>
      </div>

      <aside className="application-detail-side">
        <Card className="detail-section-card task-panel">
          <div className="section-heading"><div><CalendarClock size={18} /><span><h2>任务</h2><small>{detail.tasks.filter((item) => item.status !== "done").length} 项待完成</small></span></div><button className="icon-button" onClick={() => { setTaskError(""); setAddingTask(true); }} aria-label="新增任务"><Plus size={16} /></button></div>
          <div className="detail-task-list">{detail.tasks.length ? detail.tasks.map((task) => <div className={`detail-task ${task.status === "done" ? "is-done" : ""}`} key={task.id}><button className="detail-task-check" onClick={() => toggleTask(task)}>{task.status === "done" ? <Check size={14} /> : <Circle size={15} />}</button><div><div className="detail-task-title"><strong>{task.title}</strong><span className="detail-task-actions"><button onClick={() => { setTaskError(""); setEditingTask(task); }} aria-label="编辑任务"><Pencil size={12} /></button><button onClick={() => removeTask(task)} aria-label="删除任务"><Trash2 size={12} /></button></span></div>{task.description && <p>{task.description}</p>}<span><Badge tone={task.priority === 3 ? "red" : task.priority === 2 ? "orange" : "gray"}>{priorityText(task.priority)}</Badge>{task.applicationStage && <em>{task.applicationStage}</em>}</span></div><div className="detail-task-schedule"><time><Clock3 size={12} />{dateText(task.dueAt)}</time>{task.remindAt && <span><Bell size={11} />{reminderText(task.dueAt, task.remindAt)}</span>}</div></div>) : <div className="detail-empty">还没有任务<br />为这次投递安排一个明确的下一步吧</div>}</div>
          <button className="button button--secondary task-add-wide" onClick={() => { setTaskError(""); setAddingTask(true); }}><Plus size={14} />新增任务</button>
        </Card>
        {detail.companyNotes && <Card className="detail-section-card company-notes"><h2>公司备注</h2><p>{detail.companyNotes}</p></Card>}
      </aside>
    </div>

    {editing && <Modal title="编辑岗位与投递资料" description="修改信息后会同步更新记录" onClose={() => setEditing(false)}><form onSubmit={saveDetail}><div className="form-grid detail-edit-form">
      <label><span>公司名称 *</span><input name="companyName" required defaultValue={detail.companyName} /></label><label><span>公司简称</span><input name="companyShortName" defaultValue={detail.companyShortName} /></label>
      <label><span>行业</span><input name="industry" defaultValue={detail.industry} /></label><label><span>公司性质</span><input name="companyType" defaultValue={detail.companyType} /></label>
      <label><span>岗位名称 *</span><input name="positionTitle" required defaultValue={detail.positionTitle} /></label><label><span>部门</span><input name="department" defaultValue={detail.department} /></label>
      <label><span>地点</span><input name="location" defaultValue={detail.location} /></label><label><span>招聘类型</span><select name="recruitmentType" defaultValue={detail.recruitmentType}><option value="">未设置</option><option>校招</option><option>实习</option><option>社招</option></select></label>
      <label><span>岗位编号</span><input name="jobCode" defaultValue={detail.jobCode} /></label><label><span>投递渠道</span><input name="channel" defaultValue={detail.channel} /></label>
      <label><span>投递日期</span><input name="appliedAt" type="date" defaultValue={detail.appliedAt?.slice(0, 10)} /></label><label><span>优先级</span><select name="priority" defaultValue={detail.priority}><option value="3">高</option><option value="2">中</option><option value="1">普通</option></select></label>
      <label><span>下一步行动</span><input name="nextAction" defaultValue={detail.nextAction} placeholder="例如：准备技术一面" /></label><label><span>下一步时间</span><input name="nextActionDueAt" type="datetime-local" defaultValue={toLocalInput(detail.nextActionDueAt)} /></label>
      <label className="full"><span>关联简历</span><select name="resumeProfileId" defaultValue={detail.resumeProfileId || ""}><option value="">暂不关联</option>{detail.resumeProfileId && !resumes.some((item) => item.id === detail.resumeProfileId) && <option value={detail.resumeProfileId}>{detail.resumeName || "历史简历"}（历史引用）</option>}{resumes.filter((item) => !item.archivedAt || item.id === detail.resumeProfileId).map((resume) => <option key={resume.id} value={resume.id}>{resume.name}{resume.targetDirection ? ` · ${resume.targetDirection}` : ""}{resume.isPrimary ? "（默认）" : ""}</option>)}</select></label>
      <label><span>公司官网</span><input name="website" type="url" defaultValue={detail.website} /></label>
      <label className="full"><span>招聘链接</span><input name="sourceUrl" type="url" defaultValue={detail.sourceUrl} /></label><label className="full"><span>JD 原文</span><textarea name="jdRaw" rows={7} defaultValue={detail.jdRaw} /></label><label className="full"><span>公司备注</span><textarea name="companyNotes" rows={3} defaultValue={detail.companyNotes} /></label>
    </div><ModalActions saving={saving} onCancel={() => setEditing(false)} /></form></Modal>}

    {addingTask && <Modal title="新增任务" description={`关联到 ${detail.companyName} · ${detail.positionTitle}`} onClose={() => setAddingTask(false)}>{taskError && <div className="form-inline-error">{taskError}</div>}<form onSubmit={addTask}><div className="form-grid"><label className="full"><span>任务标题 *</span><input name="title" required autoFocus placeholder="例如：完成技术一面准备" /></label><label className="full"><span>描述</span><textarea name="description" rows={3} /></label><label><span>截止时间</span><input name="dueAt" type="datetime-local" /></label><label><span>提醒时间</span><select name="reminderOffset" defaultValue=""><option value="">不提醒</option>{reminderOptions.map((item) => <option key={item.value} value={item.value}>{item.label}</option>)}</select></label><label><span>优先级</span><select name="priority" defaultValue="2"><option value="3">高</option><option value="2">中</option><option value="1">普通</option></select></label><label><span>关联阶段</span><select name="applicationStage" defaultValue={detail.currentStage}><option value="">不限定</option>{!stages.includes(detail.currentStage) && <option>{detail.currentStage}</option>}{stages.map((stage) => <option key={stage}>{stage}</option>)}</select></label></div><ModalActions saving={saving} onCancel={() => setAddingTask(false)} /></form></Modal>}

    {editingTask && <Modal title="编辑任务" description="修改任务信息后会同步更新" onClose={() => setEditingTask(null)}>{taskError && <div className="form-inline-error">{taskError}</div>}<form onSubmit={saveTaskEdit}><div className="form-grid"><label className="full"><span>任务标题 *</span><input name="title" required autoFocus defaultValue={editingTask.title} /></label><label className="full"><span>描述</span><textarea name="description" rows={3} defaultValue={editingTask.description} /></label><label><span>截止时间</span><input name="dueAt" type="datetime-local" defaultValue={toLocalInput(editingTask.dueAt)} /></label><label><span>提醒时间</span><select name="reminderOffset" defaultValue={reminderOffsetValue(editingTask.dueAt, editingTask.remindAt)}><option value="">不提醒</option>{reminderOffsetValue(editingTask.dueAt, editingTask.remindAt) && !reminderOptions.some((item) => item.value === reminderOffsetValue(editingTask.dueAt, editingTask.remindAt)) && <option value={reminderOffsetValue(editingTask.dueAt, editingTask.remindAt)}>{reminderText(editingTask.dueAt, editingTask.remindAt)}</option>}{reminderOptions.map((item) => <option key={item.value} value={item.value}>{item.label}</option>)}</select></label><label><span>优先级</span><select name="priority" defaultValue={editingTask.priority}><option value="3">高</option><option value="2">中</option><option value="1">普通</option></select></label><label><span>关联阶段</span><select name="applicationStage" defaultValue={editingTask.applicationStage || ""}><option value="">不限定</option>{editingTask.applicationStage && !stages.includes(editingTask.applicationStage) && <option>{editingTask.applicationStage}</option>}{stages.map((stage) => <option key={stage}>{stage}</option>)}</select></label></div><ModalActions saving={saving} onCancel={() => setEditingTask(null)} /></form></Modal>}

    {addingEvent && <Modal title="补记事件" description="手动补充沟通、面试、备注等流程记录" onClose={() => setAddingEvent(false)}><form onSubmit={addEvent}><div className="form-grid"><label className="full"><span>事件标题 *</span><input name="title" required autoFocus placeholder="例如：HR 电话沟通" /></label><label className="full"><span>详细内容</span><textarea name="content" rows={4} /></label><label className="full"><span>发生时间</span><input name="happenedAt" type="datetime-local" defaultValue={toLocalInput(new Date().toISOString())} /></label></div><ModalActions saving={saving} onCancel={() => setAddingEvent(false)} /></form></Modal>}
    {editingEventTime && <Modal title="修改状态发生时间" description="调整事件发生的实际时间，方便你整理真实的时间线" onClose={() => setEditingEventTime(null)}>{eventError && <div className="form-inline-error">{eventError}</div>}<form onSubmit={saveEventTime}><div className="form-grid"><label className="full"><span>状态变更</span><input value={`${editingEventTime.stageBefore || "未知"} → ${editingEventTime.stageAfter || "未知"}`} disabled /></label><label className="full"><span>实际发生时间 *</span><input name="happenedAt" type="datetime-local" required autoFocus defaultValue={toLocalInput(editingEventTime.happenedAt)} /></label><p className="event-time-help">调整后的时间需要与前后顺序保持一致哦</p></div><ModalActions saving={saving} onCancel={() => setEditingEventTime(null)} /></form></Modal>}
  </div>;
}

function Modal({ title, description, onClose, children }: { title: string; description: string; onClose: () => void; children: React.ReactNode }) {
  useEffect(() => {
    const closeOnEscape = (event: KeyboardEvent) => { if (event.key === "Escape") onClose(); };
    window.addEventListener("keydown", closeOnEscape);
    return () => window.removeEventListener("keydown", closeOnEscape);
  }, [onClose]);
  return <div className="modal-backdrop application-detail-backdrop" onMouseDown={(event) => { if (event.target === event.currentTarget) onClose(); }}><div className="dialog application-detail-dialog" role="dialog" aria-modal="true" aria-label={title}><div className="dialog-head"><div><h2>{title}</h2><p>{description}</p></div><button type="button" aria-label="关闭" onClick={onClose}><X size={19} /></button></div>{children}</div></div>;
}

function ModalActions({ saving, onCancel }: { saving: boolean; onCancel: () => void }) {
  return <div className="dialog-actions"><button type="button" className="button button--secondary" onClick={onCancel} disabled={saving}>取消</button><button className="button button--primary" disabled={saving}>{saving ? "保存中…" : "保存"}</button></div>;
}
