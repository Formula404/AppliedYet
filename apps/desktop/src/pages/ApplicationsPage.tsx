import { useEffect, useMemo, useState } from "react";
import { Columns3, Download, Filter, GripVertical, LayoutList, MapPin, Plus, Search, Trash2 } from "lucide-react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { DndContext, pointerWithin, type DragEndEvent, useDraggable, useDroppable } from "@dnd-kit/core";
import { save } from "@tauri-apps/plugin-dialog";
import { columnStages, stageToneMap } from "../data/mock";
import { Badge, Card, PageHeader } from "../components/ui";
import type { Application } from "../types";
import { useInterviewFlow } from "../hooks/useInterviewFlow";
import { exportApplicationsExcel, hasLocalDatabase } from "../services/applications";
import { NewApplicationDialog } from "../components/NewApplicationDialog";
import { requestConfirmation, showError, showFeedback, showSuccess } from "../services/feedback";
import { trackOperation } from "../services/operations";

const columns = [
  { label: "已投递", match: ["投递"] },
  { label: "测评", match: ["测评"] },
  { label: "笔试", match: ["笔试"] },
  { label: "面试", match: ["面", "HR"] },
  { label: "等待结果", match: ["等待"] },
  { label: "Offer", match: ["Offer", "谈薪"] },
  { label: "进入人才库", match: ["人才库"] },
];
const boardColumnForStage = (stage: string) => columns.find((column) => column.match.some((keyword) => stage.includes(keyword)))?.label ?? "已投递";
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
  const { applications: apps, applicationsLoading, archiveApplication, createApplication, deleteApplication, updateApplicationStage } = useInterviewFlow();
  const [params, setParams] = useSearchParams();
  const navigate = useNavigate();
  const [view, setView] = useState<"board" | "list">("board");
  const [query, setQuery] = useState("");
  const [showNew, setShowNew] = useState(params.get("new") === "1");
  const [saving, setSaving] = useState(false);
  const [exporting, setExporting] = useState(false);
  const [statusFilter, setStatusFilter] = useState("all");
  const [cityFilter, setCityFilter] = useState("all");

  useEffect(() => { setShowNew(params.get("new") === "1"); }, [params]);

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
      await trackOperation("更新投递阶段", () => updateApplicationStage(
        application.id,
        newStage,
        (stageToneMap[newStage] || application.stageTone) as Application["stageTone"],
      ), `${application.company} · ${application.role}`);
    } catch (reason) {
      showError(reason, "阶段更新失败");
    }
  };

  const exportExcel = async () => {
    setExporting(true);
    try {
      if (!hasLocalDatabase) throw new Error("浏览器预览模式不支持写入本地 Excel，请在桌面应用中使用导出功能。");
      const path = await save({ defaultPath: `投递记录-${new Date().toLocaleDateString("en-CA")}.xls`, filters: [{ name: "Excel 工作簿", extensions: ["xls"] }] });
      if (!path) return;
      const count = await trackOperation("导出投递记录", () => exportApplicationsExcel(path), "正在生成 Excel 工作簿");
      showFeedback({ title: "导出完成", message: `已导出 ${count} 条投递及其全部流程、状态变更记录。\n${path}`, kind: "success" });
    } catch (reason) {
      showError(reason, "导出失败");
    } finally {
      setExporting(false);
    }
  };

  const close = () => { setShowNew(false); setParams({}); };

  const restoreApplication = async (application: Application) => {
    try {
      await trackOperation("恢复已归档投递", () => archiveApplication(application.id, false), `${application.company} · ${application.role}`);
      showSuccess(`${application.company} · ${application.role} 已重新加入看板与统计。`, "投递已恢复");
    } catch (reason) {
      showError(reason, "恢复失败");
    }
  };

  const removeArchivedApplication = async (application: Application) => {
    const confirmed = await requestConfirmation({
      title: "删除已归档投递？",
      message: `“${application.company} · ${application.role}”将从归档列表、统计和导出中移除。此操作无法在界面中恢复。`,
      confirmLabel: "确认删除",
      cancelLabel: "取消",
      kind: "danger",
    });
    if (!confirmed) return;
    try {
      await trackOperation("删除已归档投递", () => deleteApplication(application.id), `${application.company} · ${application.role}`);
      showSuccess(`${application.company} · ${application.role} 已从已归档投递中删除。`, "投递已删除");
    } catch (reason) {
      showError(reason, "删除失败");
    }
  };

  return (
    <div className="page page-enter">
      <PageHeader title="我的投递" description="记录所有投递，随时掌握每个岗位的进展" action={<div className="page-header-actions"><button className="button button--secondary" disabled={exporting || applicationsLoading} onClick={exportExcel}><Download size={16} />{exporting ? "导出中…" : "导出 Excel"}</button><button className="button button--primary" onClick={() => setShowNew(true)}><Plus size={16} />新增投递</button></div>} />
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
        <DndContext collisionDetection={pointerWithin} onDragEnd={handleDragEnd}>
          <div className="kanban">
            {columns.map(col => {
              const items = filtered.filter((app) => !app.stage.includes("拒绝") && boardColumnForStage(app.stage) === col.label);
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
              <b>{filtered.filter(a => a.stage.includes("拒绝")).length}</b>
            </div>
            <div className="rejected-list">
              {filtered.filter(a => a.stage.includes("拒绝")).map(app => (
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
        <div>{archivedApps.map((app) => <div className="archived-application-row" key={app.id}><span className="company-logo">{app.companyMark}</span><button onClick={() => navigate(`/applications/${app.id}`)}><strong>{app.company}</strong><small>{app.role} · {app.city}</small></button><Badge tone="gray">{app.stage}</Badge><button type="button" className="button button--secondary" onClick={() => void restoreApplication(app)}>恢复</button><button type="button" className="button button--secondary danger-text" onClick={() => void removeArchivedApplication(app)}><Trash2 size={13}/>删除</button></div>)}</div>
      </Card>}
      {showNew && <NewApplicationDialog saving={saving} onClose={close} onError={(reason) => showError(reason, "保存投递失败")} onSubmit={async (input) => {
        setSaving(true);
        try {
          const created = await trackOperation("保存新投递", () => createApplication(input), `${input.companyName} · ${input.positionTitle}`);
          close();
          showSuccess("岗位资料已保存，创建事件已写入流程记录。", "投递已创建");
          return created;
        } finally {
          setSaving(false);
        }
      }} />}
    </div>
  );
}
