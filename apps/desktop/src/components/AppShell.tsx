import { useEffect, useRef, useState } from "react";
import { NavLink, Outlet, useLocation, useNavigate } from "react-router-dom";
import { AlertCircle, BarChart3, Bell, BookOpenCheck, BriefcaseBusiness, CalendarDays, Check, ChevronDown, CircleDollarSign, Clock3, FileCheck2, Inbox, Info, Menu, Mic2, Monitor, Moon, Plus, Search, Settings, Sparkles, Sun, X } from "lucide-react";
import TitleBar from "./TitleBar";
import { useTheme } from "../hooks/useTheme";
import { useInterviewFlow } from "../hooks/useInterviewFlow";
import { startTaskNotificationScheduler } from "../services/notifications";
import { subscribeToast, type ToastPayload } from "../services/toast";
import { hasLocalDatabase } from "../services/applications";
import { listResumeProfiles } from "../services/resumes";
import { getActivitySummary, getDashboard, type ActivitySummary, type DashboardTask } from "../services/dashboard";
import { getEmailStats, listEmailMessages, syncEmails, type RecruitmentEmail } from "../services/emails";
import { getProviderSettings } from "../services/settings";

const nav = [
  ["/", "日历", CalendarDays], ["/applications", "我的投递", BriefcaseBusiness], ["/emails", "招聘邮件", Inbox],
  ["/preparation", "面试准备", BookOpenCheck], ["/mock-interview", "模拟面试", Mic2], ["/reviews", "面试复盘", FileCheck2],
  ["/question-bank", "个人题库", Sparkles], ["/offers", "Offer", CircleDollarSign], ["/analytics", "数据分析", BarChart3], ["/settings", "设置", Settings],
] as const;

const emptyActivity: ActivitySummary = { streakDays: 0, thisWeekApplications: 0, previousWeekApplications: 0, dailyActivity: Array(14).fill(0) };
const resumeName = (personalInfo: string) => {
  try { const value = JSON.parse(personalInfo) as { name?: unknown }; if (typeof value.name === "string") return value.name.trim(); } catch { /* 兼容旧的纯文本个人信息 */ }
  return personalInfo.split(/[·|｜,，\n]/)[0]?.trim() ?? "";
};
const surnameFromName = (name: string) => {
  const compact = name.replace(/\s+/g, "");
  const compound = ["欧阳", "司马", "上官", "诸葛", "东方", "皇甫", "尉迟", "公孙", "慕容", "令狐", "宇文", "长孙", "司徒", "司空", "夏侯", "南宫"].find((item) => compact.startsWith(item));
  return compound ?? Array.from(compact)[0] ?? "";
};

export default function AppShell() {
  const [collapsed, setCollapsed] = useState(false);
  const [searchOpen, setSearchOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [syncing, setSyncing] = useState(false);
  const [toast, setToast] = useState("");
  const [toastKind, setToastKind] = useState<"success" | "error" | "info">("success");
  const [globalToast, setGlobalToast] = useState<ToastPayload | null>(null);
  const [brandImageFailed, setBrandImageFailed] = useState(false);
  const [currentDate, setCurrentDate] = useState(() => new Date());
  const [surname, setSurname] = useState("");
  const [activity, setActivity] = useState<ActivitySummary>(emptyActivity);
  const [emailMessages, setEmailMessages] = useState<RecruitmentEmail[]>([]);
  const [emailPending, setEmailPending] = useState(0);
  const [notificationOpen, setNotificationOpen] = useState(false);
  const [notificationTasks, setNotificationTasks] = useState<DashboardTask[]>([]);
  const [notificationsLoading, setNotificationsLoading] = useState(false);
  const notificationRef = useRef<HTMLDivElement>(null);
  const globalToastTimer = useRef<number>();
  const location = useLocation();
  const navigate = useNavigate();
  const { mode, setMode } = useTheme();
  const { applications, applicationsLoading } = useInterviewFlow();
  const todayLabel = new Intl.DateTimeFormat("zh-CN", { year: "numeric", month: "long", day: "numeric", weekday: "short" }).format(currentDate);

  const refreshNotifications = async () => {
    setNotificationsLoading(true);
    const now = new Date();
    const todayStart = new Date(now.getFullYear(), now.getMonth(), now.getDate());
    try {
      const [result, messages, stats] = await Promise.all([
        getDashboard(new Date(now.getFullYear(), now.getMonth(), 1).toISOString(), new Date(now.getFullYear(), now.getMonth() + 1, 1).toISOString(), todayStart.toISOString(), new Date(now.getFullYear(), now.getMonth(), now.getDate() + 1).toISOString()),
        listEmailMessages(),
        getEmailStats(),
      ]);
      setNotificationTasks(result.tasks); setEmailMessages(messages); setEmailPending(stats.pending);
    } catch { /* 保留上一次成功读取的数据 */ }
    finally { setNotificationsLoading(false); }
  };

  useEffect(() => {
    const handler = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") { event.preventDefault(); setSearchOpen(true); }
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "n") { event.preventDefault(); navigate("/applications?new=1"); }
      if (event.key === "Escape") setSearchOpen(false);
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [navigate]);

  useEffect(() => {
    const timer = window.setInterval(() => setCurrentDate(new Date()), 30_000);
    return () => window.clearInterval(timer);
  }, []);

  useEffect(() => startTaskNotificationScheduler(), []);

  useEffect(() => { void refreshNotifications(); }, [location.pathname]);

  useEffect(() => {
    if (!notificationOpen) return;
    const close = (event: MouseEvent) => { if (!notificationRef.current?.contains(event.target as Node)) setNotificationOpen(false); };
    window.addEventListener("mousedown", close);
    return () => window.removeEventListener("mousedown", close);
  }, [notificationOpen]);

  useEffect(() => {
    if (!hasLocalDatabase) return;
    let timer: number | undefined;
    const refreshIndex = () => Promise.all([listEmailMessages(), getEmailStats()]).then(([items, stats]) => { setEmailMessages(items); setEmailPending(stats.pending); }).catch(() => undefined);
    void refreshIndex();
    const configureScheduler = () => {
      if (timer !== undefined) { window.clearInterval(timer); timer = undefined; }
      void getProviderSettings().then((settings) => {
        if (!settings.email.enabled) return;
        timer = window.setInterval(() => { void syncEmails().then(refreshIndex).catch(() => undefined); }, Math.max(1, settings.email.pollingMinutes) * 60_000);
      }).catch(() => undefined);
    };
    configureScheduler();
    const changed = () => { void refreshIndex(); };
    window.addEventListener("email-index-changed", changed);
    window.addEventListener("email-settings-changed", configureScheduler);
    return () => { if (timer !== undefined) window.clearInterval(timer); window.removeEventListener("email-index-changed", changed); window.removeEventListener("email-settings-changed", configureScheduler); };
  }, []);

  useEffect(() => {
    const loadIdentity = () => listResumeProfiles().then((profiles) => {
      const profile = profiles.find((item) => item.isPrimary) ?? profiles[0];
      setSurname(profile ? surnameFromName(resumeName(profile.personalInfo)) : "");
    }).catch(() => setSurname(""));
    loadIdentity();
    window.addEventListener("resume-profile-changed", loadIdentity);
    return () => window.removeEventListener("resume-profile-changed", loadIdentity);
  }, [location.pathname]);

  useEffect(() => {
    if (applicationsLoading) return;
    getActivitySummary().then(setActivity).catch(() => setActivity(emptyActivity));
  }, [applications, applicationsLoading, location.pathname]);

  useEffect(() => {
    const unsubscribe = subscribeToast((payload) => {
      if (globalToastTimer.current !== undefined) window.clearTimeout(globalToastTimer.current);
      setGlobalToast(payload);
      globalToastTimer.current = window.setTimeout(() => {
        setGlobalToast(null);
        globalToastTimer.current = undefined;
      }, payload.duration ?? 3200);
    });
    return () => {
      unsubscribe();
      if (globalToastTimer.current !== undefined) window.clearTimeout(globalToastTimer.current);
    };
  }, []);

  const sync = async () => {
    if (syncing) return;
    setSyncing(true);
    try {
      const result = await syncEmails();
      const [items, stats] = await Promise.all([listEmailMessages(), getEmailStats()]);
      setEmailMessages(items); setEmailPending(stats.pending);
      window.dispatchEvent(new Event("email-index-changed"));
      setToastKind("success"); setToast(`已识别 ${result.recognized} 封招聘邮件，其中 ${result.matched} 封匹配到投递`);
      window.setTimeout(() => setToast(""), 3200);
    } catch (reason) { setToastKind("error"); setToast(`邮件检查失败：${String(reason)}`); window.setTimeout(() => setToast(""), 4200); }
    finally { setSyncing(false); }
  };

  const results = query ? [
    ...applications.filter((item) => `${item.company}${item.role}`.toLowerCase().includes(query.toLowerCase())).map((item) => ({ label: item.company, detail: item.role, to: "/applications" })),
    ...emailMessages.filter((item) => `${item.company ?? ""}${item.subject}`.toLowerCase().includes(query.toLowerCase())).map((item) => ({ label: item.subject, detail: item.company ?? item.sender, to: "/emails" })),
  ] : [];
  const weekDifference = activity.thisWeekApplications - activity.previousWeekApplications;
  const maxActivity = Math.max(1, ...activity.dailyActivity);
  const attentionEmails = emailMessages.filter((mail) => mail.status === "pending" || mail.status === "unmatched");
  const pendingTaskCount = notificationTasks.filter((task) => task.status !== "done").length;
  const notificationCount = pendingTaskCount + attentionEmails.length;

  const notificationContent = <div className="notification-panel">
    <div className="notification-head"><span><strong>通知与待办</strong><small>{notificationCount} 项需要处理</small></span><button onClick={()=>{setNotificationOpen(false);navigate("/")}}>查看日历</button></div>
    <div className="notification-list">{notificationsLoading?<p>正在读取通知…</p>:<>
      {notificationTasks.length>0&&<><div className="notification-section-title">今日待办</div>{notificationTasks.map(task=><button key={`task-${task.id}`} className={task.status==="done"?"done":""} onClick={()=>{setNotificationOpen(false);navigate(`/applications/${task.applicationId}`)}}><span className={`notification-mark ${task.overdue?"overdue":""}`}>{task.status==="done"?<Check size={12}/>:<Clock3 size={12}/>}</span><span><strong>{task.title}</strong><small>{task.company} · {task.role}</small></span><time>{task.overdue?"已逾期":task.status==="done"?"已完成":new Date(task.dueAt).toLocaleTimeString("zh-CN",{hour:"2-digit",minute:"2-digit",hour12:false})}</time></button>)}</>}
      {attentionEmails.length>0&&<><div className="notification-section-title">招聘邮件</div>{attentionEmails.map(mail=><button key={`email-${mail.id}`} onClick={()=>{setNotificationOpen(false);navigate("/emails")}}><span className={`notification-mark notification-mark--email ${mail.status==="unmatched"?"unmatched":""}`}><Inbox size={12}/></span><span><strong>{mail.subject}</strong><small>{mail.status==="pending"?`待确认 · ${mail.company??mail.sender}`:`未匹配 · ${mail.company??mail.sender}`}</small></span><time>{new Date(mail.receivedAt).toLocaleDateString("zh-CN",{month:"numeric",day:"numeric"})}</time></button>)}</>}
      {!notificationTasks.length&&!attentionEmails.length&&<p><Check size={18}/> 暂无待处理通知</p>}
    </>}</div>
  </div>;

  const searchContent = query ? <>
    {applicationsLoading && <p>正在读取本地投递…</p>}
    {results.length ? results.map((result,index)=><button key={index} onClick={()=>{navigate(result.to);setSearchOpen(false)}}><Search size={16}/><span><strong>{result.label}</strong><small>{result.detail}</small></span><span>打开</span></button>) : !applicationsLoading && <p>没有找到相关内容</p>}
  </> : <><h4>快速操作</h4><button onClick={()=>{navigate('/applications?new=1');setSearchOpen(false)}}><Plus size={16}/><span><strong>新增投递</strong><small>记录新的公司与岗位</small></span><kbd>Ctrl N</kbd></button><button onClick={()=>{navigate('/emails');setSearchOpen(false)}}><Inbox size={16}/><span><strong>查看待确认邮件</strong><small>2 封邮件需要处理</small></span></button></>;

  return <div className={`app-shell ${collapsed ? "is-collapsed" : ""}`}>
    <TitleBar />
    <aside className="sidebar">
      <div className="brand">{brandImageFailed ? <span className="brand-mark brand-mark--fallback" aria-label="投了吗"><Check size={23}/></span> : <img className="brand-mark" src="/icon.png" alt="投了吗" onError={() => setBrandImageFailed(true)} />}<div className="brand-copy"><strong>投了吗</strong><span>Applied Yet?</span></div></div>
      <nav>{nav.map(([to, label, Icon]) => <NavLink key={to} to={to} end={to === "/"} className={({ isActive }) => `nav-item ${isActive ? "active" : ""}`}><Icon size={19} /><span>{label}</span>{label === "招聘邮件" && emailPending > 0 && <em>{emailPending}</em>}</NavLink>)}</nav>
      <div className="streak-card"><span>连续记录</span><strong>{activity.streakDays} <small>天</small></strong><div><span>本周投递 {activity.thisWeekApplications}</span><b>{weekDifference === 0 ? "与上周持平" : `较上周 ${weekDifference > 0 ? "+" : ""}${weekDifference}`}</b></div><div className="mini-bars" title="最近 14 天的投递流程记录">{activity.dailyActivity.map((count,i)=><i key={i} style={{height: count ? Math.max(4, Math.round(count / maxActivity * 22)) : 2}} />)}</div></div>
      <button className="collapse-button" onClick={() => setCollapsed(!collapsed)} aria-label="折叠侧边栏"><Menu size={18} /></button>
    </aside>
    <div className="workspace">
      <header className="topbar">
        <button className="search-trigger" onClick={() => setSearchOpen(true)}><Search size={18}/><span>搜索公司、岗位、邮件、面试记录…</span><kbd>Ctrl K</kbd></button>
        <div className="top-actions">
          {!hasLocalDatabase && <span className="demo-mode-badge" title="与桌面应用数据库完全隔离，刷新页面可恢复预置数据">演示数据</span>}
          <span className="today"><CalendarDays size={17}/>{todayLabel}</span>
          <button className="button button--secondary sync-button" disabled={syncing} onClick={sync}><span className={syncing ? "spin" : ""}>↻</span>{syncing ? "正在同步…" : "检查邮件"}</button>
          <button className="icon-button" aria-label="主题切换" title={`主题：${mode === "light" ? "浅色" : mode === "dark" ? "深色" : "跟随系统"}`} onClick={() => setMode(mode === "light" ? "dark" : mode === "dark" ? "system" : "light")}>{mode === "light" ? <Sun size={18}/> : mode === "dark" ? <Moon size={18}/> : <Monitor size={18}/>}</button>
          <div className="notification-anchor" ref={notificationRef}><button className={`icon-button ${notificationOpen ? "active" : ""}`} aria-label="通知" aria-expanded={notificationOpen} onClick={() => { const next=!notificationOpen;setNotificationOpen(next);if(next)void refreshNotifications(); }}><Bell size={19}/>{notificationCount>0&&<i/>}</button>{notificationOpen&&notificationContent}</div>
          <button className="user-button" onClick={() => navigate("/settings")} title="前往我的简历"><span>{surname || "我"}</span><b>{surname ? `${surname}同学` : "我的资料"}</b><ChevronDown size={14}/></button>
        </div>
      </header>
      <main key={location.pathname}><Outlet /></main>
    </div>
    <button className="floating-add" onClick={() => navigate("/applications?new=1")} title="新增投递（Ctrl + N）"><Plus size={22}/></button>
    {(toast || globalToast) && (() => { const kind = globalToast?.kind ?? toastKind; return <div className={`toast toast--${kind}`}>{kind === "error" ? <AlertCircle size={17}/> : kind === "info" ? <Info size={17}/> : <Check size={17}/>} {globalToast?.message ?? toast}<button onClick={() => { setToast(""); setGlobalToast(null); }}><X size={15}/></button></div>; })()}
    {searchOpen && <div className="modal-backdrop" onMouseDown={() => setSearchOpen(false)}><div className="command" onMouseDown={(e)=>e.stopPropagation()}><div className="command-input"><Search size={20}/><input autoFocus value={query} onChange={(e)=>setQuery(e.target.value)} placeholder="搜索公司、岗位、邮件、面试记录…"/><kbd>ESC</kbd></div><div className="command-body">{searchContent}</div></div></div>}
  </div>;
}
