import { useCallback, useEffect, useMemo, useState } from "react";
import { CalendarPlus, Check, ChevronDown, ChevronRight, ChevronUp, ExternalLink, Inbox, Link2, Mail, Pencil, Plus, RefreshCw, ShieldCheck, X } from "lucide-react";
import { Badge, Card, PageHeader } from "../components/ui";
import { NewApplicationDialog } from "../components/NewApplicationDialog";
import { hasLocalDatabase } from "../services/applications";
import { attachEmailToApplication, confirmEmailMatch, createApplicationFromEmail, createEmailCalendarTask, getEmailStats, ignoreEmail, listEmailMessages, rematchEmail, reviewEmail, syncEmails, type EmailStats, type RecruitmentEmail } from "../services/emails";
import { showToast } from "../services/toast";
import { openExternalUrl } from "../services/external";
import { useInterviewFlow } from "../hooks/useInterviewFlow";

type Filter = "all" | "pending" | "confirmed" | "unmatched" | "ignored";
const emptyStats: EmailStats = { thisWeek: 0, pending: 0, confirmed: 0, unmatched: 0 };
const statusLabel = { unmatched: "未匹配", pending: "待确认", confirmed: "已处理", ignored: "已忽略" } as const;
const categories = ["投递反馈 · 投递成功", "测评邀请", "笔试邀请", "面试邀请", "结果通知 · 进入下一轮", "结果通知 · 流程进展", "结果通知 · Offer", "结果通知 · 未通过", "HR 沟通", "招聘邮件", "待人工判断"];
const stages = ["已投递", "等待结果", "在线测评", "笔试", "面试中", "HR 面试", "已获Offer", "已拒绝", "进入人才库", "主动放弃"];

export default function EmailsPage() {
  const [messages, setMessages] = useState<RecruitmentEmail[]>([]);
  const [stats, setStats] = useState(emptyStats);
  const [selectedId, setSelectedId] = useState("");
  const [filter, setFilter] = useState<Filter>("all");
  const [loading, setLoading] = useState(true);
  const [syncing, setSyncing] = useState(false);
  const [busy, setBusy] = useState(false);
  const [expanded, setExpanded] = useState(false);
  const [adding, setAdding] = useState(false);
  const [reviewing, setReviewing] = useState(false);
  const { applications, createApplication } = useInterviewFlow();

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
  const detectedSchedule = useMemo(() => selected ? extractEmailSchedule(selected) : undefined, [selected]);

  async function sync() {
    if (syncing) return;
    setSyncing(true);
    try {
      const result = await syncEmails(); await load();
      window.dispatchEvent(new Event("email-index-changed"));
      const failed = result.accounts.filter((account) => account.status === "failed");
      showToast(
        failed.length
          ? `已完成 ${result.successCount} 个邮箱的检查，${result.failedCount} 个失败：${failed.map((account) => `${account.account}（${account.reason ?? "未知原因"}）`).join("；")}`
          : `检查完成：读取 ${result.fetched} 封邮件，识别 ${result.recognized} 封，匹配 ${result.matched} 封`,
        failed.length ? "error" : "success",
      );
    } catch (reason) { await load(); window.dispatchEvent(new Event("email-index-changed")); showToast(`邮件检查失败：${String(reason)}`, "error"); }
    finally { setSyncing(false); }
  }

  async function saveReview(applicationId: string, category: string, suggestedStage?: string) {
    if (!selected || busy) return;
    setBusy(true);
    try {
      await reviewEmail(selected.id, applicationId, category, suggestedStage);
      setReviewing(false);
      await load();
      window.dispatchEvent(new Event("email-index-changed"));
      showToast("已按你的选择更新关联、分类和建议阶段");
    } catch (reason) {
      showToast(String(reason), "error");
    } finally {
      setBusy(false);
    }
  }

  async function createCalendarTask() {
    if (!selected || !detectedSchedule || busy) return;
    setBusy(true);
    try {
      const remindAt = new Date(detectedSchedule.scheduledAt.getTime() - 30 * 60_000).toISOString();
      await createEmailCalendarTask(selected.id, detectedSchedule.title, detectedSchedule.scheduledAt.toISOString(), remindAt);
      await load();
      window.dispatchEvent(new Event("application-index-changed"));
      showToast("已创建日历任务，并设置提前 30 分钟提醒");
    } catch (reason) {
      showToast(String(reason), "error");
    } finally {
      setBusy(false);
    }
  }

  async function act(action: "confirm" | "ignore" | "rematch") {
    if (!selected || busy) return; setBusy(true);
    try {
      if (action === "confirm") await confirmEmailMatch(selected.id);
      if (action === "ignore") await ignoreEmail(selected.id);
      if (action === "rematch") await rematchEmail(selected.id);
      await load();
      window.dispatchEvent(new Event("email-index-changed"));
      if (action === "confirm") window.dispatchEvent(new Event("application-index-changed"));
      showToast(action === "confirm" ? "已写入投递时间线并安全更新阶段" : action === "ignore" ? "已忽略该邮件" : "已重新识别邮件阶段并匹配当前投递");
    } catch (reason) { showToast(String(reason), "error"); }
    finally { setBusy(false); }
  }

  return <div className="page page-enter"><PageHeader title="招聘邮件" description={stats.lastSyncedAt ? `自动识别招聘邮件 · 上次同步 ${formatTime(stats.lastSyncedAt)}` : "自动识别招聘邮件，一键更新投递进度"} action={<button className="button button--secondary" disabled={syncing} onClick={sync}><RefreshCw className={syncing ? "spin" : ""} size={16}/>{syncing ? "正在检查…" : "立即检查"}</button>} />
    <div className="mail-stats"><Card><span className="stat-icon blue"><Inbox/></span><span><small>近 7 天招聘邮件</small><strong>{stats.thisWeek}</strong></span></Card><Card><span className="stat-icon orange"><Link2/></span><span><small>待处理</small><strong>{stats.pending + stats.unmatched}</strong></span></Card><Card><span className="stat-icon green"><Check/></span><span><small>已处理</small><strong>{stats.confirmed}</strong></span></Card><div className="privacy-note"><ShieldCheck size={18}/><span><strong>你的邮件只在本地处理</strong><small>不会被上传，原邮件也不会被修改</small></span></div></div>
    <div className="email-layout"><Card className="email-list"><div className="email-filters"><button className={filter === "all" ? "active" : ""} onClick={() => setFilter("all")}>全部 <b>{messages.length}</b></button><button className={filter === "pending" ? "active" : ""} onClick={() => setFilter("pending")}>待确认 <b>{stats.pending}</b></button><button className={filter === "unmatched" ? "active" : ""} onClick={() => setFilter("unmatched")}>未匹配 <b>{stats.unmatched}</b></button><button className={filter === "confirmed" ? "active" : ""} onClick={() => setFilter("confirmed")}>已处理</button><button className={filter === "ignored" ? "active" : ""} onClick={() => setFilter("ignored")}>已忽略</button></div>
      {loading && <div className="settings-notice">正在读取本地邮件索引…</div>}
      {!loading && !filtered.length && <div className="email-empty"><Mail size={34}/><strong>暂无符合条件的招聘邮件</strong><small>{hasLocalDatabase ? "点击“立即检查”读取新邮件" : "当前为预览模式，展示的是示例数据"}</small></div>}
      {filtered.map((mail) => <button className={`email-item ${selectedId === mail.id ? "selected" : ""}`} key={mail.id} onClick={() => setSelectedId(mail.id)}><span className="company-logo">{mail.company?.[0] ?? "邮"}</span><span><span><strong>{mail.company ?? senderName(mail.sender)}</strong><time>{formatTime(mail.receivedAt)}</time></span><b>{mail.subject}</b><small>{mail.snippet || "邮件没有可显示的纯文本正文"}</small><span><Badge tone={mail.status === "confirmed" ? "green" : mail.status === "pending" ? "orange" : "gray"}>{statusLabel[mail.status]}</Badge><em>{mail.matchedApplicationId ? isManualMatch(mail) ? "人工关联" : `匹配分 ${mail.confidence}` : "等待匹配"}</em></span></span></button>)}</Card>
      <Card className="email-detail">{selected ? <><div className="detail-head"><div><Badge tone={selected.status === "confirmed" ? "green" : "blue"}>{selected.category}</Badge><h2>{selected.subject}</h2><p>{selected.sender} · {formatTime(selected.receivedAt)}</p></div></div><div className={`mail-body ${expanded ? "is-expanded" : ""}`}><div className="mail-body-content">{linkifyText(expanded ? selected.bodyText : selected.snippet || "邮件没有可显示的纯文本正文。")}</div>{expanded && selected.links.length > 0 && <div className="mail-link-list"><strong>邮件中的链接</strong>{selected.links.map((link, index) => <a key={link.url} href={link.url} onClick={(event) => { event.preventDefault(); void openExternalUrl(link.url).catch((reason) => showToast(String(reason), "error")); }}><ExternalLink size={13}/><span>{link.label || `邮件链接 ${index + 1}`}</span><small>{link.url}</small></a>)}</div>}<button type="button" className="mail-expand-button" onClick={() => setExpanded((value) => !value)}>{expanded ? <ChevronUp size={14}/> : <ChevronDown size={14}/>} {expanded ? "收起邮件全文" : "展开邮件全文"}</button></div>
        <div className="match-analysis"><div className="analysis-title"><span><Link2 size={17}/>匹配分析</span><strong>{selected.matchedApplicationId ? isManualMatch(selected) ? "人工确认关联" : `匹配分 ${selected.confidence} · ${selected.confidence >= 90 ? "明确" : "需复核"}` : "存在歧义或尚未匹配"}</strong></div>
        {selected.matchedApplicationId ? <><div className="match-target"><span className="company-logo">{selected.company?.[0] ?? "投"}</span><span><strong>{selected.company} · {selected.role}</strong><small>当前阶段：{selected.currentStage ?? "未知"}</small></span><ChevronRight size={18}/></div><dl>{selected.reasons.map((reason, index) => <div key={reason}><dt>依据 {index + 1}</dt><dd>{reason}</dd></div>)}</dl></> : <div className="settings-notice">当前投递中没有足够接近的记录。你可以将邮件直接加入投递流程，或先完善已有投递后重新匹配。</div>}</div>
        {selected.suggestedStage && <div className="update-suggestion"><strong>建议更新</strong><p>按邮件接收时间新增“{selected.category}”事件，并以当时的历史阶段判断是否进入“{selected.suggestedStage}”；如果时间线上已有更晚节点，当前阶段会保留较新的状态。</p></div>}
        {detectedSchedule && <div className="update-suggestion"><strong><CalendarPlus size={16}/>识别到日程</strong><p>{detectedSchedule.title}：{detectedSchedule.scheduledAt.toLocaleString("zh-CN", { dateStyle: "medium", timeStyle: "short" })}</p><button type="button" className="button button--secondary" disabled={!hasLocalDatabase || busy || selected.status !== "confirmed" || !selected.matchedApplicationId || selected.calendarTaskCreated || detectedSchedule.scheduledAt.getTime() <= Date.now()} onClick={createCalendarTask}>{!hasLocalDatabase ? "桌面版可创建" : selected.calendarTaskCreated ? "已加入日历" : detectedSchedule.scheduledAt.getTime() <= Date.now() ? "时间已过" : selected.status === "confirmed" ? "创建日历与提醒" : "确认邮件后可创建"}</button></div>}
        <div className="detail-actions">{selected.status === "ignored" ? <button className="button button--secondary" disabled={busy} onClick={() => act("rematch")}><RefreshCw size={16}/>恢复处理</button> : <button className="button button--secondary" disabled={busy || selected.status === "confirmed"} onClick={() => act("ignore")}>移出待办</button>}<button className="button button--secondary" disabled={busy || selected.status === "confirmed"} onClick={() => setReviewing(true)}><Pencil size={16}/>调整关联与阶段</button><button className="button button--secondary" disabled={busy || selected.status === "ignored"} onClick={() => act("rematch")}>重新识别与匹配</button>{!selected.matchedApplicationId && selected.status !== "confirmed" ? <button className="button button--primary" disabled={busy || selected.status === "ignored"} onClick={() => setAdding(true)}><Plus size={16}/>新建投递并关联</button> : <button className="button button--primary" disabled={busy || selected.status === "confirmed" || selected.status === "ignored"} onClick={() => act("confirm")}><Check size={16}/>{selected.status === "confirmed" ? "已处理" : "确认并写入时间线"}</button>}</div></> : <div className="email-empty"><Mail size={38}/><strong>选择一封邮件查看识别结果</strong></div>}</Card>
    </div>
    {adding && selected && <NewApplicationDialog
      saving={busy}
      onClose={() => setAdding(false)}
      emailStage={selected.category}
      requireAppliedAt
      description={isApplicationReceipt(selected) ? "已识别为投递成功回执，请确认自动填入的信息" : `这是一封${selected.category}邮件，请补充原投递信息后加入流程`}
      maxAppliedAt={inputDate(selected.receivedAt)}
      defaults={{
        companyName: selected.company ?? "",
        positionTitle: selected.role ?? "",
        appliedAt: isApplicationReceipt(selected) ? inputDate(selected.receivedAt) : "",
        channel: "邮件识别",
      }}
      onError={(reason) => showToast(`新增投递失败：${String(reason)}`, "error")}
      onSubmit={async (input) => {
        setBusy(true);
        try {
          const application = hasLocalDatabase
            ? await createApplicationFromEmail(selected.id, input)
            : await createApplication(input);
          if (!hasLocalDatabase) {
            await attachEmailToApplication(selected.id, application.id);
            await confirmEmailMatch(selected.id);
          }
          setAdding(false);
          await load();
          window.dispatchEvent(new Event("email-index-changed"));
          window.dispatchEvent(new Event("application-index-changed"));
          showToast(isApplicationReceipt(selected) ? "已加入“已投递”流程" : `已创建投递并写入“${selected.category}”阶段`);
          return application;
        } finally {
          setBusy(false);
        }
      }}
    />}
    {reviewing && selected && <EmailReviewDialog
      mail={selected}
      applications={applications.filter((application) => !application.archived)}
      saving={busy}
      onClose={() => setReviewing(false)}
      onSubmit={saveReview}
    />}
  </div>;
}

interface EmailReviewDialogProps {
  mail: RecruitmentEmail;
  applications: ReturnType<typeof useInterviewFlow>["applications"];
  saving: boolean;
  onClose: () => void;
  onSubmit: (applicationId: string, category: string, suggestedStage?: string) => Promise<void>;
}

function EmailReviewDialog({ mail, applications, saving, onClose, onSubmit }: EmailReviewDialogProps) {
  const initialApplicationId = mail.matchedApplicationId && applications.some((item) => item.id === mail.matchedApplicationId)
    ? mail.matchedApplicationId
    : applications[0]?.id ?? "";
  const [applicationId, setApplicationId] = useState(initialApplicationId);
  const [category, setCategory] = useState(categories.includes(mail.category) ? mail.category : "待人工判断");
  const [suggestedStage, setSuggestedStage] = useState(mail.suggestedStage ?? "");

  return <div className="modal-backdrop" onMouseDown={onClose}><div className="dialog email-review-dialog" role="dialog" aria-modal="true" aria-labelledby="email-review-title" onMouseDown={(event) => event.stopPropagation()}>
    <div className="dialog-head"><div><h2 id="email-review-title">调整邮件处理方式</h2><p>人工选择优先于自动识别；你也可以只写入时间线，不改变投递阶段。</p></div><button type="button" onClick={onClose} disabled={saving} aria-label="关闭"><X size={19}/></button></div>
    <div className="form-grid">
      <label className="full"><span>关联到已有投递 *</span><select value={applicationId} onChange={(event) => setApplicationId(event.target.value)} disabled={saving || !applications.length}><option value="">请选择投递</option>{applications.map((application) => <option key={application.id} value={application.id}>{application.company} · {application.role}（{application.stage}）</option>)}</select></label>
      <label><span>邮件分类 *</span><select value={category} onChange={(event) => setCategory(event.target.value)} disabled={saving}>{categories.map((value) => <option key={value}>{value}</option>)}</select></label>
      <label><span>阶段处理</span><select value={suggestedStage} onChange={(event) => setSuggestedStage(event.target.value)} disabled={saving}><option value="">只加入时间线，不改阶段</option>{stages.map((value) => <option key={value}>{value}</option>)}</select></label>
    </div>
    {!applications.length && <div className="settings-notice">当前没有可关联的活跃投递，请先使用“新建投递并关联”。</div>}
    <div className="dialog-actions"><button type="button" className="button button--secondary" onClick={onClose} disabled={saving}>取消</button><button type="button" className="button button--primary" disabled={saving || !applicationId} onClick={() => void onSubmit(applicationId, category, suggestedStage || undefined)}>{saving ? "保存中…" : "保存人工判断"}</button></div>
  </div></div>;
}

function isApplicationReceipt(mail: RecruitmentEmail) { return mail.category.startsWith("投递反馈") && mail.suggestedStage === "已投递"; }
function isManualMatch(mail: RecruitmentEmail) { return mail.reasons.some((reason) => reason.startsWith("用户手动")); }
function extractEmailSchedule(mail: RecruitmentEmail) {
  const primaryBody = mail.bodyText.split(/\n(?:-----\s*(?:original message|原始邮件)\s*-----|发件人:|from:|>)/i)[0] ?? mail.bodyText;
  const text = `${mail.subject}\n${primaryBody.slice(0, 6000)}`;
  const received = new Date(mail.receivedAt);
  if (Number.isNaN(received.getTime())) return undefined;
  const dateMatch = text.match(/(?:(\d{4})[年/-](\d{1,2})[月/-](\d{1,2})日?|(\d{1,2})月(\d{1,2})日)/);
  const relativeMatch = text.match(/(今天|明天|后天)/);
  if (!dateMatch && !relativeMatch) return undefined;
  const date = new Date(received);
  date.setSeconds(0, 0);
  if (dateMatch) {
    const year = dateMatch[1] ? Number(dateMatch[1]) : received.getFullYear();
    const month = Number(dateMatch[2] ?? dateMatch[4]);
    const day = Number(dateMatch[3] ?? dateMatch[5]);
    if (month < 1 || month > 12 || day < 1 || day > 31) return undefined;
    date.setFullYear(year, month - 1, day);
    if (date.getMonth() !== month - 1 || date.getDate() !== day) return undefined;
    if (!dateMatch[1] && date.getTime() < received.getTime() - 180 * 24 * 60 * 60_000) date.setFullYear(year + 1);
  } else if (relativeMatch) {
    const offset = relativeMatch[1] === "明天" ? 1 : relativeMatch[1] === "后天" ? 2 : 0;
    date.setDate(date.getDate() + offset);
  }
  const anchor = dateMatch ?? relativeMatch;
  const tail = anchor ? text.slice((anchor.index ?? 0) + anchor[0].length, (anchor.index ?? 0) + anchor[0].length + 40) : text;
  const timeMatch = tail.match(/(上午|下午|晚上)?\s*(\d{1,2})(?::|点)(\d{1,2})?/);
  if (timeMatch) {
    let hour = Number(timeMatch[2]);
    if ((timeMatch[1] === "下午" || timeMatch[1] === "晚上") && hour < 12) hour += 12;
    const minute = Number(timeMatch[3] ?? 0);
    if (hour > 23 || minute > 59) return undefined;
    date.setHours(hour, minute, 0, 0);
  } else if (/截止|deadline|expire/i.test(text) || mail.category.includes("测评") || mail.category.includes("笔试")) {
    date.setHours(23, 59, 0, 0);
  } else {
    return undefined;
  }
  if (Number.isNaN(date.getTime())
    || date.getMonth() < 0
    || date.getDate() < 1
    || date.getHours() > 23
    || date.getMinutes() > 59) return undefined;
  const isDeadline = /截止|deadline|expire/i.test(text);
  return {
    scheduledAt: date,
    title: isDeadline ? `${mail.category}截止` : mail.category,
  };
}
function inputDate(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "";
  const pad = (part: number) => String(part).padStart(2, "0");
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
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
