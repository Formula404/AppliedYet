import { useEffect, useMemo, useState } from "react";
import { CheckCircle2, Columns3, Download, Filter, GripVertical, LayoutList, MapPin, Plus, Search, Trash2, X } from "lucide-react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { DndContext, type DragEndEvent, useDraggable, useDroppable } from "@dnd-kit/core";
import { save } from "@tauri-apps/plugin-dialog";
import { columnStages, stageToneMap } from "../data/mock";
import { Badge, Card, PageHeader } from "../components/ui";
import type { Application } from "../types";
import { useInterviewFlow } from "../hooks/useInterviewFlow";
import { listResumeProfiles, type ResumeProfile } from "../services/resumes";
import { exportApplicationsExcel, hasLocalDatabase } from "../services/applications";

const columns = [
  { label: "已投递", match: ["投递"] },
  { label: "测评", match: ["测评"] },
  { label: "笔试", match: ["笔试"] },
  { label: "面试", match: ["面", "HR"] },
  { label: "等待结果", match: ["等待"] },
  { label: "Offer", match: ["Offer", "谈薪"] },
  { label: "进入人才库", match: ["人才库"] },
];
const systemTimeZone = Intl.DateTimeFormat().resolvedOptions().timeZone;
const localDateTime = (value: string) => {
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString("zh-CN", { month: "numeric", day: "numeric", hour: "2-digit", minute: "2-digit", hour12: false, timeZone: systemTimeZone });
};

function DroppableColumn({ col, children }: { col: (typeof columns)[number]; children: React.ReactNode }) {
  const { setNodeRef, isOver } = useDroppable({ id: col.label });
  return (
    <div ref={setNodeRef} className={`kanban-column ${isOver ? "over" : ""}`}>
      {children}
    </div>
  );
}

function DraggableCard({ app, onOpen }: { app: Application; onOpen: () => void }) {
  const { attributes, listeners, setNodeRef, transform, isDragging } = useDraggable({ id: app.id });
  const style = transform ? { transform: `translate(${transform.x}px, ${transform.y}px)` } : undefined;
  return (
    <Card ref={setNodeRef} className={`application-card ${isDragging ? "dragging" : ""}`} style={style}>
      <div className="application-top">
        <span className="company-logo">{app.companyMark}</span>
        <span>
          <strong>{app.company}</strong>
          <small>{app.role}</small>
        </span>
        <div className="drag-handle" {...listeners} {...attributes}>
          <GripVertical size={17} />
        </div>
      </div>
      <div className="application-meta">
        <span><MapPin size={13} />{app.city}</span>
        <Badge tone={app.stageTone}>{app.stage}</Badge>
      </div>
      <div className="application-next">
        <small>下一步</small>
        <strong>{app.nextStep}</strong>
        <time>{localDateTime(app.nextTime)}</time>
      </div>
      {app.risk && <p className="risk-note">! {app.risk}</p>}
      <div className="application-foot">
        <span>{app.resumeName ? `简历：${app.resumeName}` : `${app.priority}优先级`}</span>
        <time>{localDateTime(app.updated)}更新</time>
        <button type="button" className="application-detail-link" onClick={onOpen}>详情</button>
      </div>
    </Card>
  );
}

function RejectedItem({ app, onOpen }: { app: Application; onOpen: () => void }) {
  const { attributes, listeners, setNodeRef, transform, isDragging } = useDraggable({ id: app.id });
  const style = transform ? { transform: `translate(${transform.x}px, ${transform.y}px)` } : undefined;
  return (
    <div ref={setNodeRef} className={`rejected-item ${isDragging ? "dragging" : ""}`} style={style}>
      <span className="company-logo">{app.companyMark}</span>
      <div className="rejected-item-info">
        <strong>{app.company}</strong>
        <small>{app.role} · {app.city}</small>
      </div>
      <Badge tone="red">已拒绝</Badge>
      <span className="rejected-item-time">{localDateTime(app.updated)}</span>
      <button type="button" className="application-detail-link" onClick={onOpen}>详情</button>
      <div className="drag-handle" {...listeners} {...attributes}>
        <GripVertical size={15} />
      </div>
    </div>
  );
}

function RejectedDroppable({ children }: { children: React.ReactNode }) {
  const { setNodeRef, isOver } = useDroppable({ id: "已拒绝" });
  return (
    <div ref={setNodeRef} className={`rejected-section ${isOver ? "over" : ""}`}>
      {children}
    </div>
  );
}

export default function ApplicationsPage() {
  const { applications: apps, applicationsLoading, applicationsError, archiveApplication, createApplication, deleteApplication, updateApplicationStage } = useInterviewFlow();
  const [params, setParams] = useSearchParams();
  const navigate = useNavigate();
  const [view, setView] = useState<"board" | "list">("board");
  const [query, setQuery] = useState("");
  const [showNew, setShowNew] = useState(params.get("new") === "1");
  const [saving, setSaving] = useState(false);
  const [exporting, setExporting] = useState(false);
  const [resumes, setResumes] = useState<ResumeProfile[]>([]);
  const [statusFilter, setStatusFilter] = useState("all");
  const [cityFilter, setCityFilter] = useState("all");
  const [notice, setNotice] = useState<{ title: string; message: string; kind: "success" | "error" } | null>(null);
  const [pendingDelete, setPendingDelete] = useState<Application | null>(null);

  useEffect(() => { setShowNew(params.get("new") === "1"); }, [params]);
  useEffect(() => { if (showNew) listResumeProfiles().then((items) => setResumes(items.filter((item) => !item.archivedAt))).catch((reason) => setNotice({ title: "简历读取失败", message: String(reason), kind: "error" })); }, [showNew]);
  useEffect(() => { if (applicationsError) setNotice({ title: "数据操作失败", message: applicationsError, kind: "error" }); }, [applicationsError]);

  const activeApps = useMemo(() => apps.filter((item) => !item.archived), [apps]);
  const archivedApps = useMemo(() => apps.filter((item) => item.archived), [apps]);
  const statuses = useMemo(() => Array.from(new Set(activeApps.map((item) => item.stage))).sort((a, b) => a.localeCompare(b, "zh-CN")), [activeApps]);
  const cities = useMemo(() => Array.from(new Set(activeApps.map((item) => item.city).filter(Boolean))).sort((a, b) => a.localeCompare(b, "zh-CN")), [activeApps]);
  const filtered = useMemo(
    () => activeApps.filter((app) => {
      const matchesQuery = `${app.company}${app.role}${app.city}${app.stage}${app.resumeName || ""}`.toLowerCase().includes(query.trim().toLowerCase());
      return matchesQuery && (statusFilter === "all" || app.stage === statusFilter) && (cityFilter === "all" || app.city === cityFilter);
    }),
    [query, activeApps, statusFilter, cityFilter],
  );

  const handleDragEnd = async (event: DragEndEvent) => {
    const { active, over } = event;
    if (!over) return;
    const target = over.id as string;
    const newStage = columnStages[target];
    if (!newStage) return;

    const application = apps.find((item) => item.id === active.id);
    if (!application || application.stage === newStage) return;
    try {
      await updateApplicationStage(application.id, newStage, (stageToneMap[newStage] || application.stageTone) as Application["stageTone"]);
    } catch (reason) {
      setNotice({ title: "阶段更新失败", message: String(reason), kind: "error" });
    }
  };

  const exportExcel = async () => {
    setExporting(true);
    try {
      if (!hasLocalDatabase) throw new Error("浏览器预览模式不支持写入本地 Excel，请在桌面应用中使用导出功能。");
      const path = await save({ defaultPath: `投递记录-${new Date().toLocaleDateString("en-CA")}.xls`, filters: [{ name: "Excel 工作簿", extensions: ["xls"] }] });
      if (!path) return;
      const count = await exportApplicationsExcel(path);
      setNotice({ title: "导出完成", message: `已导出 ${count} 条投递及其全部流程、状态变更记录。\n${path}`, kind: "success" });
    } catch (reason) {
      setNotice({ title: "导出失败", message: String(reason), kind: "error" });
    } finally {
      setExporting(false);
    }
  };

  const close = () => { setShowNew(false); setParams({}); };

  return (
    <div className="page page-enter">
      <PageHeader title="我的投递" description="集中管理岗位状态、下一步行动与完整流程记录" action={<div className="page-header-actions"><button className="button button--secondary" disabled={exporting || applicationsLoading} onClick={exportExcel}><Download size={16} />{exporting ? "导出中…" : "导出 Excel"}</button><button className="button button--primary" onClick={() => setShowNew(true)}><Plus size={16} />新增投递</button></div>} />
      <div className="toolbar">
        <div className="inline-search"><Search size={16} /><input value={query} onChange={e => setQuery(e.target.value)} placeholder="搜索公司或岗位" /></div>
        <label className="filter-select"><Filter size={16} /><select aria-label="按状态筛选" value={statusFilter} onChange={(event) => setStatusFilter(event.target.value)}><option value="all">全部状态</option>{statuses.map((status) => <option key={status}>{status}</option>)}</select></label>
        <label className="filter-select"><MapPin size={15} /><select aria-label="按城市筛选" value={cityFilter} onChange={(event) => setCityFilter(event.target.value)}><option value="all">全部城市</option>{cities.map((city) => <option key={city}>{city}</option>)}</select></label>
        {(query || statusFilter !== "all" || cityFilter !== "all") && <button type="button" className="filter-clear" onClick={() => { setQuery(""); setStatusFilter("all"); setCityFilter("all"); }}>清除筛选</button>}
        <span className="toolbar-spacer" />
        <div className="view-toggle">
          <button className={view === "board" ? "active" : ""} onClick={() => setView("board")}><Columns3 size={16} />看板</button>
          <button className={view === "list" ? "active" : ""} onClick={() => setView("list")}><LayoutList size={16} />列表</button>
        </div>
      </div>
      {applicationsLoading && <div className="success-banner">正在读取本地数据库…</div>}
      {view === "board" ? (
        <DndContext onDragEnd={handleDragEnd}>
          <div className="kanban">
            {columns.map(col => {
              const items = filtered.filter(a => col.match.some(m => a.stage.includes(m)));
              return (
                <DroppableColumn key={col.label} col={col}>
                  <div className="kanban-title">
                    <span>{col.label}</span><b>{items.length}</b>
                    <button type="button" aria-label={`新增${col.label}投递`} onClick={() => setShowNew(true)}><Plus size={15} /></button>
                  </div>
                  <div className="kanban-items">
                    {items.map(app => <DraggableCard key={app.id} app={app} onOpen={() => navigate(`/applications/${app.id}`)} />)}
                  </div>
                </DroppableColumn>
              );
            })}
          </div>
          <RejectedDroppable>
            <div className="rejected-header">
              <h3>已拒绝</h3>
              <b>{filtered.filter(a => a.stage === "已拒绝").length}</b>
            </div>
            <div className="rejected-list">
              {filtered.filter(a => a.stage === "已拒绝").map(app => (
                <RejectedItem key={app.id} app={app} onOpen={() => navigate(`/applications/${app.id}`)} />
              ))}
            </div>
          </RejectedDroppable>
        </DndContext>
      ) : (
        <Card className="table-card">
          <table>
            <thead>
              <tr>
                <th>公司 / 岗位</th><th>地点</th><th>当前阶段</th><th>使用简历</th><th>下一步</th><th>优先级</th><th>最近更新</th><th>操作</th>
              </tr>
            </thead>
            <tbody>
              {filtered.map(app => (
                <tr key={app.id} onDoubleClick={() => navigate(`/applications/${app.id}`)} title="双击查看详情">
                  <td><span className="company-logo">{app.companyMark}</span><span><strong>{app.company}</strong><small>{app.role}</small></span></td>
                  <td>{app.city}</td>
                  <td><Badge tone={app.stageTone}>{app.stage}</Badge></td>
                  <td>{app.resumeName || "未关联"}</td>
                  <td>{app.nextStep}<small>{localDateTime(app.nextTime)}</small></td>
                  <td>{app.priority}</td>
                  <td>{localDateTime(app.updated)}</td>
                  <td><button type="button" className="application-detail-link" onClick={() => navigate(`/applications/${app.id}`)}>查看详情</button></td>
                </tr>
              ))}
              {!filtered.length && <tr className="application-table-empty"><td colSpan={8}><div className="detail-empty">没有符合当前筛选条件的投递</div></td></tr>}
            </tbody>
          </table>
        </Card>
      )}
      {archivedApps.length > 0 && <Card className="archived-applications">
        <div className="archived-applications-head"><div><h3>已归档投递</h3><span>{archivedApps.length} 项</span></div><small>归档记录不参与首页统计和提醒</small></div>
        <div>{archivedApps.map((app) => <div className="archived-application-row" key={app.id}><span className="company-logo">{app.companyMark}</span><button onClick={() => navigate(`/applications/${app.id}`)}><strong>{app.company}</strong><small>{app.role} · {app.city}</small></button><Badge tone="gray">{app.stage}</Badge><button type="button" className="button button--secondary" onClick={async () => { try { await archiveApplication(app.id, false); setNotice({ title: "投递已恢复", message: `${app.company} · ${app.role} 已重新加入看板与统计。`, kind: "success" }); } catch (reason) { setNotice({ title: "恢复失败", message: String(reason), kind: "error" }); } }}>恢复</button><button type="button" className="button button--secondary danger-text" onClick={() => setPendingDelete(app)}><Trash2 size={13}/>删除</button></div>)}</div>
      </Card>}
      {showNew && (
        <div className="modal-backdrop">
          <div className="dialog application-dialog">
            <div className="dialog-head">
              <div><h2>新增投递</h2><p>先记录核心信息，稍后可继续完善岗位档案</p></div>
              <button onClick={close}><X size={19} /></button>
            </div>
            <form onSubmit={async e => {
              e.preventDefault();
              const data = new FormData(e.currentTarget);
              setSaving(true);
              try {
                await createApplication({
                  companyName: String(data.get("companyName") || ""),
                  companyShortName: String(data.get("companyShortName") || "") || undefined,
                  industry: String(data.get("industry") || "") || undefined,
                  companyType: String(data.get("companyType") || "") || undefined,
                  website: String(data.get("website") || "") || undefined,
                  companyNotes: String(data.get("companyNotes") || "") || undefined,
                  positionTitle: String(data.get("positionTitle") || ""),
                  department: String(data.get("department") || "") || undefined,
                  location: String(data.get("location") || ""),
                  recruitmentType: String(data.get("recruitmentType") || "") || undefined,
                  jobCode: String(data.get("jobCode") || "") || undefined,
                  sourceUrl: String(data.get("sourceUrl") || "") || undefined,
                  channel: String(data.get("channel") || ""),
                  appliedAt: String(data.get("appliedAt") || ""),
                  priority: Number(data.get("priority") || 2),
                  jdRaw: String(data.get("jdRaw") || ""),
                  resumeProfileId: String(data.get("resumeProfileId") || "") || undefined,
                });
                close();
                setNotice({ title: "投递已创建", message: "岗位资料已保存，创建事件已写入流程记录。", kind: "success" });
              } catch (reason) {
                setNotice({ title: "保存投递失败", message: String(reason), kind: "error" });
              } finally {
                setSaving(false);
              }
            }}>
              <div className="form-grid">
                <label><span>公司名称 *</span><input name="companyName" required placeholder="例如：蚂蚁集团" /></label>
                <label><span>公司简称</span><input name="companyShortName" placeholder="例如：蚂蚁" /></label>
                <label><span>行业</span><input name="industry" placeholder="例如：互联网金融" /></label>
                <label><span>公司性质</span><input name="companyType" placeholder="例如：民营企业" /></label>
                <label><span>岗位名称 *</span><input name="positionTitle" required placeholder="例如：后端开发工程师" /></label>
                <label><span>部门</span><input name="department" placeholder="例如：基础架构部" /></label>
                <label><span>工作地点</span><input name="location" placeholder="杭州" /></label>
                <label><span>招聘类型</span><select name="recruitmentType"><option value="">未设置</option><option>校招</option><option>实习</option><option>社招</option></select></label>
                <label><span>岗位编号</span><input name="jobCode" /></label>
                <label><span>投递渠道</span><select name="channel"><option>招聘官网</option><option>Boss 直聘</option><option>内推</option><option>其他</option></select></label>
                <label><span>投递日期</span><input name="appliedAt" type="date" defaultValue={new Date().toLocaleDateString("en-CA")} /></label>
                <label><span>优先级</span><select name="priority" defaultValue="2"><option value="3">高</option><option value="2">中</option><option value="1">普通</option></select></label>
                <label><span>使用简历</span><select key={resumes.map((item) => item.id).join("|")} name="resumeProfileId" defaultValue={resumes.find((item) => item.isPrimary)?.id ?? ""}><option value="">暂不关联</option>{resumes.map((resume) => <option key={resume.id} value={resume.id}>{resume.name}{resume.targetDirection ? ` · ${resume.targetDirection}` : ""}{resume.isPrimary ? "（默认）" : ""}</option>)}</select></label>
                <label><span>公司官网</span><input name="website" type="url" placeholder="https://" /></label>
                <label className="full"><span>招聘链接</span><input name="sourceUrl" type="url" placeholder="https://" /></label>
                <label className="full"><span>JD 原文</span><textarea name="jdRaw" rows={5} placeholder="粘贴岗位描述，后续将用于岗位准备与问题预测" /></label>
                <label className="full"><span>公司备注</span><textarea name="companyNotes" rows={3} placeholder="记录团队、业务或沟通信息" /></label>
              </div>
              <div className="dialog-actions">
                <button type="button" className="button button--secondary" onClick={close} disabled={saving}>取消</button>
                <button className="button button--primary" disabled={saving}>{saving ? "保存中…" : "保存投递"}</button>
              </div>
            </form>
          </div>
        </div>
      )}
      {pendingDelete && <div className="modal-backdrop status-modal-backdrop" onMouseDown={() => setPendingDelete(null)}><div className="dialog status-dialog confirm-delete-dialog" role="alertdialog" aria-modal="true" onMouseDown={(event) => event.stopPropagation()}><span className="status-dialog-icon status-dialog-icon--danger"><Trash2 size={24}/></span><h2>删除已归档投递？</h2><p>“{pendingDelete.company} · {pendingDelete.role}”将从归档列表、统计和导出中移除。此操作无法在界面中恢复。</p><div className="confirm-dialog-actions"><button type="button" className="button button--secondary" onClick={() => setPendingDelete(null)}>取消</button><button type="button" className="button button--danger" onClick={async () => { const target = pendingDelete; try { await deleteApplication(target.id); setPendingDelete(null); setNotice({ title: "投递已删除", message: `${target.company} · ${target.role} 已从已归档投递中删除。`, kind: "success" }); } catch (reason) { setPendingDelete(null); setNotice({ title: "删除失败", message: String(reason), kind: "error" }); } }}><Trash2 size={14}/>确认删除</button></div></div></div>}
      {notice && <div className="modal-backdrop status-modal-backdrop" onMouseDown={() => setNotice(null)}><div className={`dialog status-dialog status-dialog--${notice.kind}`} role="alertdialog" aria-modal="true" onMouseDown={(event) => event.stopPropagation()}><button type="button" className="status-dialog-close" onClick={() => setNotice(null)}><X size={18} /></button><span className="status-dialog-icon">{notice.kind === "success" ? <CheckCircle2 size={26} /> : <X size={24} />}</span><h2>{notice.title}</h2><p>{notice.message}</p><button type="button" className="button button--primary" onClick={() => setNotice(null)}>知道了</button></div></div>}
    </div>
  );
}
