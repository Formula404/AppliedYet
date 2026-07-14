import { useEffect, useState } from "react";
import { NavLink, Outlet, useLocation, useNavigate } from "react-router-dom";
import { BarChart3, Bell, BookOpenCheck, BriefcaseBusiness, CalendarDays, Check, ChevronDown, CircleDollarSign, FileCheck2, Inbox, Menu, Mic2, Monitor, Moon, Plus, Search, Settings, Sparkles, Sun, X } from "lucide-react";
import { mails } from "../data/mock";
import TitleBar from "./TitleBar";
import { useTheme } from "../hooks/useTheme";
import { useInterviewFlow } from "../hooks/useInterviewFlow";

const nav = [
  ["/", "日历", CalendarDays], ["/applications", "我的投递", BriefcaseBusiness], ["/emails", "招聘邮件", Inbox],
  ["/preparation", "面试准备", BookOpenCheck], ["/mock-interview", "模拟面试", Mic2], ["/reviews", "面试复盘", FileCheck2],
  ["/question-bank", "个人题库", Sparkles], ["/offers", "Offer", CircleDollarSign], ["/analytics", "数据分析", BarChart3], ["/settings", "设置", Settings],
] as const;

export default function AppShell() {
  const [collapsed, setCollapsed] = useState(false);
  const [searchOpen, setSearchOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [syncing, setSyncing] = useState(false);
  const [toast, setToast] = useState("");
  const location = useLocation();
  const navigate = useNavigate();
  const { mode, setMode } = useTheme();
  const { applications } = useInterviewFlow();

  useEffect(() => {
    const handler = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") { event.preventDefault(); setSearchOpen(true); }
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "n") { event.preventDefault(); navigate("/applications?new=1"); }
      if (event.key === "Escape") setSearchOpen(false);
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [navigate]);

  const sync = () => {
    if (syncing) return;
    setSyncing(true);
    window.setTimeout(() => { setSyncing(false); setToast("已发现 3 封招聘邮件，其中 1 封需要确认"); window.setTimeout(() => setToast(""), 3200); }, 1100);
  };

  const results = query ? [
    ...applications.filter((item) => `${item.company}${item.role}`.toLowerCase().includes(query.toLowerCase())).map((item) => ({ label: item.company, detail: item.role, to: "/applications" })),
    ...mails.filter((item) => `${item.company}${item.subject}`.toLowerCase().includes(query.toLowerCase())).map((item) => ({ label: item.subject, detail: item.company, to: "/emails" })),
  ] : [];

  return <div className={`app-shell ${collapsed ? "is-collapsed" : ""}`}>
    <TitleBar />
    <aside className="sidebar">
      <div className="brand"><img className="brand-mark" src="/icon.png" alt="投了吗" /><div className="brand-copy"><strong>投了吗</strong><span>Applied Yet?</span></div></div>
      <nav>{nav.map(([to, label, Icon]) => <NavLink key={to} to={to} end={to === "/"} className={({ isActive }) => `nav-item ${isActive ? "active" : ""}`}><Icon size={19} /><span>{label}</span>{label === "招聘邮件" && <em>2</em>}</NavLink>)}</nav>
      <div className="streak-card"><span>连续记录</span><strong>32 <small>天</small></strong><div><span>本周投递 18</span><b>较上周 +6</b></div><div className="mini-bars">{[4,8,5,13,8,16,12,18,10,15,22,13,17,9].map((h,i)=><i key={i} style={{height:h}} />)}</div></div>
      <button className="collapse-button" onClick={() => setCollapsed(!collapsed)} aria-label="折叠侧边栏"><Menu size={18} /></button>
    </aside>
    <div className="workspace">
      <header className="topbar">
        <button className="search-trigger" onClick={() => setSearchOpen(true)}><Search size={18}/><span>搜索公司、岗位、邮件、面试记录…</span><kbd>Ctrl K</kbd></button>
        <div className="top-actions"><span className="today"><CalendarDays size={17}/> 2026年7月13日&nbsp; 周一</span><button className="button button--secondary sync-button" onClick={sync}><span className={syncing ? "spin" : ""}>↻</span>{syncing ? "正在同步…" : "检查邮件"}</button><button className="icon-button" aria-label="主题切换" title={`主题：${mode === "light" ? "浅色" : mode === "dark" ? "深色" : "跟随系统"}`} onClick={() => setMode(mode === "light" ? "dark" : mode === "dark" ? "system" : "light")}>{mode === "light" ? <Sun size={18}/> : mode === "dark" ? <Moon size={18}/> : <Monitor size={18}/>}</button><button className="icon-button" aria-label="通知"><Bell size={19}/><i/></button><button className="user-button"><span>林</span><b>林同学</b><ChevronDown size={14}/></button></div>
      </header>
      <main key={location.pathname}><Outlet /></main>
    </div>
    <button className="floating-add" onClick={() => navigate("/applications?new=1")} title="新增投递（Ctrl + N）"><Plus size={22}/></button>
    {toast && <div className="toast"><Check size={17}/>{toast}<button onClick={() => setToast("")}><X size={15}/></button></div>}
    {searchOpen && <div className="modal-backdrop" onMouseDown={() => setSearchOpen(false)}><div className="command" onMouseDown={(e)=>e.stopPropagation()}><div className="command-input"><Search size={20}/><input autoFocus value={query} onChange={(e)=>setQuery(e.target.value)} placeholder="搜索公司、岗位、邮件、面试记录…"/><kbd>ESC</kbd></div><div className="command-body">{query ? (results.length ? results.map((result,index)=><button key={index} onClick={()=>{navigate(result.to);setSearchOpen(false)}}><Search size={16}/><span><strong>{result.label}</strong><small>{result.detail}</small></span><span>打开</span></button>) : <p>没有找到相关内容</p>) : <><h4>快速操作</h4><button onClick={()=>{navigate('/applications?new=1');setSearchOpen(false)}}><Plus size={16}/><span><strong>新增投递</strong><small>记录新的公司与岗位</small></span><kbd>Ctrl N</kbd></button><button onClick={()=>{navigate('/emails');setSearchOpen(false)}}><Inbox size={16}/><span><strong>查看待确认邮件</strong><small>2 封邮件需要处理</small></span></button></>}</div></div></div>}
  </div>;
}
