import { useCallback, useEffect, useMemo, useState } from "react";
import { Check, ChevronDown, ChevronRight, ChevronUp, ExternalLink, Inbox, Link2, Mail, RefreshCw, ShieldCheck } from "lucide-react";
import { Badge, Card, PageHeader } from "../components/ui";
import { hasLocalDatabase } from "../services/applications";
import { confirmEmailMatch, getEmailStats, ignoreEmail, listEmailMessages, rematchEmail, syncEmails, type EmailStats, type RecruitmentEmail } from "../services/emails";
import { showToast } from "../services/toast";
import { openExternalUrl } from "../services/external";

type Filter = "all" | "pending" | "confirmed" | "unmatched";
const emptyStats: EmailStats = { thisWeek: 0, pending: 0, confirmed: 0, unmatched: 0 };
const statusLabel = { unmatched: "未匹配", pending: "待确认", confirmed: "已更新流程", ignored: "已忽略" } as const;

export default function EmailsPage() {
  const [messages, setMessages] = useState<RecruitmentEmail[]>([]);
  const [stats, setStats] = useState(emptyStats);
  const [selectedId, setSelectedId] = useState("");
  const [filter, setFilter] = useState<Filter>("all");
  const [loading, setLoading] = useState(true);
  const [syncing, setSyncing] = useState(false);
  const [busy, setBusy] = useState(false);
  const [expanded, setExpanded] = useState(false);

  const load = useCallback(async () => {
    try {
      const [items, nextStats] = await Promise.all([listEmailMessages(), getEmailStats()]);
      setMessages(items); setStats(nextStats);
      setSelectedId((current) => items.some((item) => item.id === current) ? current : items[0]?.id ?? "");
    } catch (reason) { showToast(String(reason), "error"); }
    finally { setLoading(false); }
  }, []);
  useEffect(() => { void load(); }, [load]);
  useEffect(() => { setExpanded(false); }, [selectedId]);

  const filtered = useMemo(() => messages.filter((item) => filter === "all" || item.status === filter), [messages, filter]);
  useEffect(() => { setSelectedId((current) => filtered.some((item) => item.id === current) ? current : filtered[0]?.id ?? ""); }, [filtered]);
  const selected = filtered.find((item) => item.id === selectedId);

  async function sync() {
    if (syncing) return;
    setSyncing(true);
    try {
      const result = await syncEmails(); await load();
      window.dispatchEvent(new Event("email-index-changed"));
      showToast(`检查完成：读取 ${result.fetched} 封新邮件，识别 ${result.recognized} 封，匹配 ${result.matched} 封`);
    } catch (reason) { showToast(`邮件检查失败：${String(reason)}`, "error"); }
    finally { setSyncing(false); }
  }

  async function act(action: "confirm" | "ignore" | "rematch") {
    if (!selected || busy) return; setBusy(true);
    try {
      if (action === "confirm") await confirmEmailMatch(selected.id);
      if (action === "ignore") await ignoreEmail(selected.id);
      if (action === "rematch") await rematchEmail(selected.id);
      await load();
      window.dispatchEvent(new Event("email-index-changed"));
      showToast(action === "confirm" ? "已写入投递时间线并安全更新阶段" : action === "ignore" ? "已忽略该邮件" : "已重新识别邮件阶段并匹配当前投递");
    } catch (reason) { showToast(String(reason), "error"); }
    finally { setBusy(false); }
  }

  return <div className="page page-enter"><PageHeader title="招聘邮件" description="自动识别招聘邮件，一键更新投递进度" action={<button className="button button--secondary" disabled={syncing} onClick={sync}><RefreshCw className={syncing ? "spin" : ""} size={16}/>{syncing ? "正在检查…" : "立即检查"}</button>} />
    <div className="mail-stats"><Card><span className="stat-icon blue"><Inbox/></span><span><small>近 7 天招聘邮件</small><strong>{stats.thisWeek}</strong></span></Card><Card><span className="stat-icon orange"><Link2/></span><span><small>待确认匹配</small><strong>{stats.pending}</strong></span></Card><Card><span className="stat-icon green"><Check/></span><span><small>已更新流程</small><strong>{stats.confirmed}</strong></span></Card><div className="privacy-note"><ShieldCheck size={18}/><span><strong>你的邮件只在本地处理</strong><small>不会被上传，原邮件也不会被修改</small></span></div></div>
    <div className="email-layout"><Card className="email-list"><div className="email-filters"><button className={filter === "all" ? "active" : ""} onClick={() => setFilter("all")}>全部 <b>{messages.length}</b></button><button className={filter === "pending" ? "active" : ""} onClick={() => setFilter("pending")}>待确认 <b>{stats.pending}</b></button><button className={filter === "unmatched" ? "active" : ""} onClick={() => setFilter("unmatched")}>未匹配 <b>{stats.unmatched}</b></button><button className={filter === "confirmed" ? "active" : ""} onClick={() => setFilter("confirmed")}>已更新</button></div>
      {loading && <div className="settings-notice">正在读取本地邮件索引…</div>}
      {!loading && !filtered.length && <div className="email-empty"><Mail size={34}/><strong>暂无符合条件的招聘邮件</strong><small>{hasLocalDatabase ? "点击“立即检查”读取新邮件" : "当前为预览模式，展示的是示例数据"}</small></div>}
      {filtered.map((mail) => <button className={`email-item ${selectedId === mail.id ? "selected" : ""}`} key={mail.id} onClick={() => setSelectedId(mail.id)}><span className="company-logo">{mail.company?.[0] ?? "邮"}</span><span><span><strong>{mail.company ?? senderName(mail.sender)}</strong><time>{formatTime(mail.receivedAt)}</time></span><b>{mail.subject}</b><small>{mail.snippet || "邮件没有可显示的纯文本正文"}</small><span><Badge tone={mail.status === "confirmed" ? "green" : mail.status === "pending" ? "orange" : "gray"}>{statusLabel[mail.status]}</Badge><em>{mail.matchedApplicationId ? `${mail.confidence}% 匹配` : "等待匹配"}</em></span></span></button>)}</Card>
      <Card className="email-detail">{selected ? <><div className="detail-head"><div><Badge tone={selected.status === "confirmed" ? "green" : "blue"}>{selected.category}</Badge><h2>{selected.subject}</h2><p>{selected.sender} · {formatTime(selected.receivedAt)}</p></div></div><div className={`mail-body ${expanded ? "is-expanded" : ""}`}><div className="mail-body-content">{linkifyText(expanded ? selected.bodyText : selected.snippet || "邮件没有可显示的纯文本正文。")}</div>{expanded && selected.links.length > 0 && <div className="mail-link-list"><strong>邮件中的链接</strong>{selected.links.map((link, index) => <a key={link.url} href={link.url} onClick={(event) => { event.preventDefault(); void openExternalUrl(link.url).catch((reason) => showToast(String(reason), "error")); }}><ExternalLink size={13}/><span>{link.label || `邮件链接 ${index + 1}`}</span><small>{link.url}</small></a>)}</div>}<button type="button" className="mail-expand-button" onClick={() => setExpanded((value) => !value)}>{expanded ? <ChevronUp size={14}/> : <ChevronDown size={14}/>} {expanded ? "收起邮件全文" : "展开邮件全文"}</button></div>
        <div className="match-analysis"><div className="analysis-title"><span><Link2 size={17}/>匹配分析</span><strong>{selected.matchedApplicationId ? `${selected.confidence}% ${selected.confidence >= 75 ? "高" : "中"}置信度` : "尚未匹配"}</strong></div>
        {selected.matchedApplicationId ? <><div className="match-target"><span className="company-logo">{selected.company?.[0] ?? "投"}</span><span><strong>{selected.company} · {selected.role}</strong><small>当前阶段：{selected.currentStage ?? "未知"}</small></span><ChevronRight size={18}/></div><dl>{selected.reasons.map((reason, index) => <div key={reason}><dt>依据 {index + 1}</dt><dd>{reason}</dd></div>)}</dl></> : <div className="settings-notice">当前投递中没有足够接近的公司或岗位，请补充投递信息后重新匹配。</div>}</div>
        {selected.suggestedStage && <div className="update-suggestion"><strong>建议更新</strong><p>新增“{selected.category}”邮件事件{selected.currentStage !== selected.suggestedStage ? `，并将阶段安全推进至“${selected.suggestedStage}”；阶段更新可从时间线撤销` : "；当前阶段无需变更"}。</p></div>}
        <div className="detail-actions"><button className="button button--secondary" disabled={busy || selected.status === "confirmed"} onClick={() => act("ignore")}>忽略邮件</button><button className="button button--secondary" disabled={busy} onClick={() => act("rematch")}>重新识别与匹配</button><button className="button button--primary" disabled={busy || !selected.matchedApplicationId || selected.status === "confirmed"} onClick={() => act("confirm")}><Check size={16}/>{selected.status === "confirmed" ? "已更新流程" : "确认匹配并更新"}</button></div></> : <div className="email-empty"><Mail size={38}/><strong>选择一封邮件查看识别结果</strong></div>}</Card>
    </div>
  </div>;
}

function senderName(sender: string) { return sender.split("<")[0]?.replace(/[\"']/g, "").trim() || "招聘邮件"; }
function formatTime(value: string) { const date = new Date(value); return Number.isNaN(date.getTime()) ? value : new Intl.DateTimeFormat("zh-CN", { month: "numeric", day: "numeric", hour: "2-digit", minute: "2-digit" }).format(date); }
function linkifyText(text: string) {
  const pattern = /(https?:\/\/[^\s<>"']+|mailto:[^\s<>"']+)/gi;
  const nodes: Array<string | JSX.Element> = []; let offset = 0;
  for (const match of text.matchAll(pattern)) {
    const start = match.index ?? 0; let url = match[0];
    const trailing = url.match(/[),.;!?，。；！？）】]+$/)?.[0] ?? "";
    if (trailing) url = url.slice(0, -trailing.length);
    if (start > offset) nodes.push(text.slice(offset, start));
    nodes.push(<a key={`${start}-${url}`} href={url} onClick={(event) => { event.preventDefault(); void openExternalUrl(url).catch((reason) => showToast(String(reason), "error")); }}>{url}<ExternalLink size={11}/></a>);
    if (trailing) nodes.push(trailing);
    offset = start + match[0].length;
  }
  if (offset < text.length) nodes.push(text.slice(offset));
  return nodes;
}
