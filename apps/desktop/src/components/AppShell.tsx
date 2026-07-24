import { useEffect, useRef, useState } from "react";
import { NavLink, Outlet, useLocation, useNavigate } from "react-router-dom";
import { BarChart3, Bell, BookOpenCheck, BriefcaseBusiness, CalendarDays, Check, ChevronDown, CircleDollarSign, Clock3, FileCheck2, Inbox, Menu, Mic2, Monitor, Moon, Plus, Search, Settings, Sparkles, Sun } from "lucide-react";
import gsap from "gsap";
import { useGSAP } from "@gsap/react";
import TitleBar from "./TitleBar";
import FeedbackCenter from "./FeedbackCenter";
import OperationCenter from "./OperationCenter";
import { useTheme } from "../hooks/useTheme";
import { useInterviewFlow } from "../hooks/useInterviewFlow";
import { startTaskNotificationScheduler } from "../services/notifications";
import { requestConfirmation, showError, showFeedback } from "../services/feedback";
import { trackOperation } from "../services/operations";
import { hasLocalDatabase } from "../services/applications";
import { listResumeProfiles } from "../services/resumes";
import { getActivitySummary, getDashboard, type ActivitySummary, type DashboardTask } from "../services/dashboard";
import { getEmailStats, listEmailMessages, syncEmails, type RecruitmentEmail } from "../services/emails";
import { getProviderSettings } from "../services/settings";
import { checkForUpdate } from "../services/updates";

gsap.registerPlugin(useGSAP);

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
  const shellRef = useRef<HTMLDivElement>(null);
  const mainRef = useRef<HTMLElement>(null);
  const [collapsed, setCollapsed] = useState(false);
  const [searchOpen, setSearchOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [syncing, setSyncing] = useState(false);
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
  const notificationRequest = useRef(0);
  const location = useLocation();
  const navigate = useNavigate();
  const { mode, setMode } = useTheme();
  const {
    applications,
    applicationsLoading,
    applicationsError,
    processingJobs,
    processingJobsError,
    processingRequestCount,
  } = useInterviewFlow();
  const activeProcessingCount = Math.max(
    processingJobs.filter((job) => job.status === "running" || job.importStatus === "running").length,
    processingRequestCount,
  );
  const todayLabel = new Intl.DateTimeFormat("zh-CN", { year: "numeric", month: "long", day: "numeric", weekday: "short" }).format(currentDate);

  const refreshNotifications = async () => {
    const request = ++notificationRequest.current;
    setNotificationsLoading(true);
    const now = new Date();
    const todayStart = new Date(now.getFullYear(), now.getMonth(), now.getDate());
    try {
      const [result, messages, stats] = await Promise.all([
        getDashboard(new Date(now.getFullYear(), now.getMonth(), 1).toISOString(), new Date(now.getFullYear(), now.getMonth() + 1, 1).toISOString(), todayStart.toISOString(), new Date(now.getFullYear(), now.getMonth(), now.getDate() + 1).toISOString()),
        listEmailMessages(),
        getEmailStats(),
      ]);
      if (request === notificationRequest.current) { setNotificationTasks(result.tasks); setEmailMessages(messages); setEmailPending(stats.pending + stats.unmatched); }
    } catch { /* 保留上一次成功读取的数据 */ }
    finally { if (request === notificationRequest.current) setNotificationsLoading(false); }
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
  useEffect(() => () => { notificationRequest.current += 1; }, []);

  useEffect(() => {
    if (sessionStorage.getItem("applied-yet:update-checked")) return;
    sessionStorage.setItem("applied-yet:update-checked", "1");
    const timer = window.setTimeout(() => {
      void checkForUpdate().then(async (update) => {
        if (!update) return;
        const confirmed = await requestConfirmation({
          title: `发现新版本 v${update.version}`,
          message: `当前版本 v${update.currentVersion}。可以前往设置查看更新内容并安装。`,
          confirmLabel: "查看并更新",
          cancelLabel: "稍后",
          kind: "info",
        });
        if (confirmed) navigate("/settings?tab=updates");
      }).catch(() => undefined);
    }, 2500);
    return () => window.clearTimeout(timer);
  }, [navigate]);

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
    let schedulerRevision = 0;
    let disposed = false;
    const refreshIndex = () => Promise.all([listEmailMessages(), getEmailStats()]).then(([items, stats]) => { setEmailMessages(items); setEmailPending(stats.pending + stats.unmatched); }).catch(() => undefined);
    void refreshIndex();
    const configureScheduler = () => {
      const revision = ++schedulerRevision;
      if (timer !== undefined) { window.clearInterval(timer); timer = undefined; }
      void getProviderSettings().then((settings) => {
        if (disposed || revision !== schedulerRevision || !settings.email.enabled) return;
        const runAutomaticSync = () => {
          void trackOperation("自动检查招聘邮件", async (operation) => {
            operation.update("正在连接已启用的邮箱账户");
            const result = await syncEmails();
            await refreshIndex();
            if (result.failedCount > 0) {
              const failed = result.accounts.filter((account) => account.status === "failed");
              showFeedback({
                title: "邮件自动检查部分失败",
                message: failed.map((account) => `${account.account}（${account.reason ?? "未知原因"}）`).join("；"),
                kind: "error",
              });
            }
          }).catch((reason) => {
            void refreshIndex();
            showError(reason, "邮件自动检查异常");
          });
        };
        runAutomaticSync();
        timer = window.setInterval(runAutomaticSync, Math.max(1, settings.email.pollingMinutes) * 60_000);
      }).catch(() => undefined);
    };
    configureScheduler();
    const changed = () => { void refreshIndex(); };
    window.addEventListener("email-index-changed", changed);
    window.addEventListener("email-settings-changed", configureScheduler);
    return () => { disposed = true; schedulerRevision += 1; if (timer !== undefined) window.clearInterval(timer); window.removeEventListener("email-index-changed", changed); window.removeEventListener("email-settings-changed", configureScheduler); };
  }, []);

  useEffect(() => {
    const loadIdentity = () => listResumeProfiles().then((profiles) => {
      const profile = profiles.find((item) => item.isPrimary) ?? profiles[0];
      setSurname(profile ? surnameFromName(resumeName(profile.personalInfo)) : "");
    }).catch(() => setSurname(""));
    loadIdentity();
    window.addEventListener("resume-profile-changed", loadIdentity);
    return () => window.removeEventListener("resume-profile-changed", loadIdentity);
  }, []);

  useEffect(() => {
    if (applicationsLoading) return;
    getActivitySummary().then(setActivity).catch(() => setActivity(emptyActivity));
  }, [applications, applicationsLoading, location.pathname]);

  useEffect(() => {
    if (applicationsError) showError(applicationsError, "投递数据操作失败");
  }, [applicationsError]);

  useEffect(() => {
    if (processingJobsError) showError(processingJobsError, "材料处理状态读取失败");
  }, [processingJobsError]);

  useGSAP(() => {
    if (!notificationOpen || !notificationRef.current) return;
    const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    gsap.fromTo(
      notificationRef.current.querySelector(".notification-panel"),
      { autoAlpha: 0, y: reduceMotion ? 0 : -8, scale: reduceMotion ? 1 : 0.98 },
      { autoAlpha: 1, y: 0, scale: 1, duration: reduceMotion ? 0 : 0.22, ease: "power2.out" },
    );
  }, { scope: notificationRef, dependencies: [notificationOpen], revertOnUpdate: true });

  useGSAP(() => {
    const page = mainRef.current?.firstElementChild;
    if (!page) return;
    const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    gsap.fromTo(
      page,
      { autoAlpha: 0, y: reduceMotion ? 0 : 10 },
      { autoAlpha: 1, y: 0, duration: reduceMotion ? 0 : 0.32, ease: "power2.out", clearProps: "transform,opacity,visibility" },
    );
  }, { scope: shellRef, dependencies: [location.pathname], revertOnUpdate: true });

  useGSAP(() => {
    if (!searchOpen) return;
    const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    gsap.fromTo(
      ".command-backdrop",
      { autoAlpha: 0 },
      { autoAlpha: 1, duration: reduceMotion ? 0 : 0.16, ease: "power1.out" },
    );
    gsap.fromTo(
      ".command",
      { autoAlpha: 0, y: reduceMotion ? 0 : -12, scale: reduceMotion ? 1 : 0.98 },
      { autoAlpha: 1, y: 0, scale: 1, duration: reduceMotion ? 0 : 0.24, ease: "power2.out" },
    );
  }, { scope: shellRef, dependencies: [searchOpen], revertOnUpdate: true });

  const sync = async () => {
    if (syncing) return;
    setSyncing(true);
    try {
      await trackOperation("检查招聘邮件", async (operation) => {
        operation.update("正在连接邮箱并识别招聘流程");
        const result = await syncEmails();
        operation.update("正在刷新邮件索引");
        const [items, stats] = await Promise.all([listEmailMessages(), getEmailStats()]);
        setEmailMessages(items); setEmailPending(stats.pending + stats.unmatched);
        window.dispatchEvent(new Event("email-index-changed"));
        const failed = result.accounts.filter((account) => account.status === "failed");
        showFeedback({
          title: failed.length ? "邮件检查部分完成" : "邮件检查完成",
          message: failed.length
            ? `已检查 ${result.successCount} 个邮箱，${result.failedCount} 个失败：${failed.map((account) => `${account.account}（${account.reason ?? "未知原因"}）`).join("；")}`
            : `已识别 ${result.recognized} 封招聘邮件，其中 ${result.matched} 封匹配到投递`,
          kind: failed.length ? "warning" : "success",
        });
      });
    } catch (reason) {
      const [items, stats] = await Promise.all([listEmailMessages(), getEmailStats()]).catch(() => [undefined, undefined] as const);
      if (items && stats) { setEmailMessages(items); setEmailPending(stats.pending + stats.unmatched); }
      showError(reason, "邮件检查失败");
    }
    finally { setSyncing(false); }
  };

  const results = query ? [
    ...applications.filter((item) => `${item.company}${item.role}`.toLowerCase().includes(query.toLowerCase())).map((item) => ({ label: item.company, detail: item.role, to: `/applications/${item.id}` })),
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
  </> : <><h4>快速操作</h4><button onClick={()=>{navigate('/applications?new=1');setSearchOpen(false)}}><Plus size={16}/><span><strong>新增投递</strong><small>记录新的公司与岗位</small></span><kbd>Ctrl N</kbd></button><button onClick={()=>{navigate('/emails');setSearchOpen(false)}}><Inbox size={16}/><span><strong>查看待确认邮件</strong><small>{emailPending ? `${emailPending} 封邮件需要处理` : "暂无待确认邮件"}</small></span></button></>;

  return <div ref={shellRef} className={`app-shell ${collapsed ? "is-collapsed" : ""}`}>
    <TitleBar />
    <aside className="sidebar">
      <div className="brand">{brandImageFailed ? <span className="brand-mark brand-mark--fallback" aria-label="投了吗"><Check size={23}/></span> : <img className="brand-mark" src="/icon.png" alt="投了吗" onError={() => setBrandImageFailed(true)} />}<div className="brand-copy"><strong>投了吗</strong><span>Applied Yet?</span></div></div>
      <nav>{nav.map(([to, label, Icon]) => <NavLink key={to} to={to} end={to === "/"} className={({ isActive }) => `nav-item ${isActive ? "active" : ""}`}><Icon size={19} /><span>{label}</span>{label === "招聘邮件" && emailPending > 0 && <em>{emailPending}</em>}{label === "面试复盘" && activeProcessingCount > 0 && <em>{activeProcessingCount}</em>}</NavLink>)}</nav>
      <div className="streak-card"><span>连续记录</span><strong>{activity.streakDays} <small>天</small></strong><div><span>本周投递 {activity.thisWeekApplications}</span><b>{weekDifference === 0 ? "与上周持平" : `较上周 ${weekDifference > 0 ? "+" : ""}${weekDifference}`}</b></div><div className="mini-bars" title="最近 14 天的投递流程记录">{activity.dailyActivity.map((count,i)=><i key={i} style={{height: count ? Math.max(4, Math.round(count / maxActivity * 22)) : 2}} />)}</div></div>
      <button className="collapse-button" onClick={() => setCollapsed(!collapsed)} aria-label="折叠侧边栏"><Menu size={18} /></button>
    </aside>
    <div className="workspace">
      <header className="topbar">
        <button className="search-trigger" onClick={() => setSearchOpen(true)}><Search size={18}/><span>搜索公司、岗位、邮件、面试记录…</span><kbd>Ctrl K</kbd></button>
        <div className="top-actions">
          {!hasLocalDatabase && <span className="demo-mode-badge" title="当前为预览模式，数据仅供预览，刷新可重置">预览模式</span>}
          <OperationCenter processingJobs={processingJobs} processingRequestCount={processingRequestCount} onOpenProcessing={() => navigate("/reviews")} />
          <span className="today"><CalendarDays size={17}/>{todayLabel}</span>
          <button className="button button--secondary sync-button" disabled={syncing} onClick={sync}><span className={syncing ? "spin" : ""}>↻</span>{syncing ? "正在同步…" : "检查邮件"}</button>
          <button className="icon-button" aria-label="主题切换" title={`主题：${mode === "light" ? "浅色" : mode === "dark" ? "深色" : "跟随系统"}`} onClick={() => setMode(mode === "light" ? "dark" : mode === "dark" ? "system" : "light")}>{mode === "light" ? <Sun size={18}/> : mode === "dark" ? <Moon size={18}/> : <Monitor size={18}/>}</button>
          <div className="notification-anchor" ref={notificationRef}><button className={`icon-button ${notificationOpen ? "active" : ""}`} aria-label="通知" aria-expanded={notificationOpen} onClick={() => { const next=!notificationOpen;setNotificationOpen(next);if(next)void refreshNotifications(); }}><Bell size={19}/>{notificationCount>0&&<i/>}</button>{notificationOpen&&notificationContent}</div>
          <button className="user-button" onClick={() => navigate("/settings")} title="前往我的简历"><span>{surname || "我"}</span><b>{surname ? `${surname}同学` : "我的资料"}</b><ChevronDown size={14}/></button>
        </div>
      </header>
      <main ref={mainRef} key={location.pathname}><Outlet /></main>
    </div>
    <FeedbackCenter />
    {searchOpen && <div className="modal-backdrop command-backdrop" onMouseDown={() => setSearchOpen(false)}><div className="command" onMouseDown={(e)=>e.stopPropagation()}><div className="command-input"><Search size={20}/><input autoFocus value={query} onChange={(e)=>setQuery(e.target.value)} placeholder="搜索公司、岗位、邮件、面试记录…"/><kbd>ESC</kbd></div><div className="command-body">{searchContent}</div></div></div>}
  </div>;
}
