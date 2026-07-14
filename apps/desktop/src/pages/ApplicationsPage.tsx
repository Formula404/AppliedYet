import { useEffect, useMemo, useState } from "react";
import { Columns3, Filter, GripVertical, LayoutList, MapPin, Plus, Search, X } from "lucide-react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { DndContext, type DragEndEvent, useDraggable, useDroppable } from "@dnd-kit/core";
import { columnStages, stageToneMap } from "../data/mock";
import { Badge, Card, PageHeader } from "../components/ui";
import type { Application } from "../types";
import { useInterviewFlow } from "../hooks/useInterviewFlow";
import { showToast } from "../services/toast";
import { listResumeProfiles, type ResumeProfile } from "../services/resumes";

const columns = [
  { label: "已投递", match: ["投递"] },
  { label: "测评", match: ["测评"] },
  { label: "面试", match: ["面", "HR"] },
  { label: "等待结果", match: ["等待"] },
  { label: "Offer", match: ["Offer"] },
];

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
        <time>{app.nextTime}</time>
      </div>
      {app.risk && <p className="risk-note">! {app.risk}</p>}
      <div className="application-foot">
        <span>{app.resumeName ? `简历：${app.resumeName}` : `${app.priority}优先级`}</span>
        <time>{app.updated}更新</time>
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
      <span className="rejected-item-time">{app.updated}</span>
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
  const { applications: apps, applicationsLoading, applicationsError, archiveApplication, createApplication, updateApplicationStage } = useInterviewFlow();
  const [params, setParams] = useSearchParams();
  const navigate = useNavigate();
  const [view, setView] = useState<"board" | "list">("board");
  const [query, setQuery] = useState("");
  const [showNew, setShowNew] = useState(params.get("new") === "1");
  const [saving, setSaving] = useState(false);
  const [resumes, setResumes] = useState<ResumeProfile[]>([]);

  useEffect(() => { setShowNew(params.get("new") === "1"); }, [params]);
  useEffect(() => { if (showNew) listResumeProfiles().then((items) => setResumes(items.filter((item) => !item.archivedAt))).catch((reason) => showToast(String(reason), "error")); }, [showNew]);
  useEffect(() => { if (applicationsError) showToast(`本地数据操作失败：${applicationsError}`, "error"); }, [applicationsError]);

  const activeApps = useMemo(() => apps.filter((item) => !item.archived), [apps]);
  const archivedApps = useMemo(() => apps.filter((item) => item.archived), [apps]);
  const filtered = useMemo(
    () => activeApps.filter(a => `${a.company}${a.role}${a.city}`.toLowerCase().includes(query.toLowerCase())),
    [query, activeApps],
  );

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;
    if (!over) return;
    const target = over.id as string;
    const newStage = columnStages[target];
    if (!newStage) return;

    const application = apps.find((item) => item.id === active.id);
    if (!application || application.stage === newStage) return;
    updateApplicationStage(application.id, newStage, (stageToneMap[newStage] || application.stageTone) as Application["stageTone"]);
  };

  const close = () => { setShowNew(false); setParams({}); };

  return (
    <div className="page page-enter">
      <PageHeader title="我的投递" description="集中管理岗位状态、下一步行动与完整流程记录" action={<button className="button button--primary" onClick={() => setShowNew(true)}><Plus size={16} />新增投递</button>} />
      <div className="toolbar">
        <div className="inline-search"><Search size={16} /><input value={query} onChange={e => setQuery(e.target.value)} placeholder="搜索公司或岗位" /></div>
        <button className="filter-button"><Filter size={16} />全部状态</button>
        <button className="filter-button">全部城市</button>
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
                    <button><Plus size={15} /></button>
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
                  <td>{app.nextStep}<small>{app.nextTime}</small></td>
                  <td>{app.priority}</td>
                  <td>{app.updated}</td>
                  <td><button type="button" className="application-detail-link" onClick={() => navigate(`/applications/${app.id}`)}>查看详情</button></td>
                </tr>
              ))}
            </tbody>
          </table>
        </Card>
      )}
      {archivedApps.length > 0 && <Card className="archived-applications">
        <div className="archived-applications-head"><div><h3>已归档投递</h3><span>{archivedApps.length} 项</span></div><small>归档记录不参与首页统计和提醒</small></div>
        <div>{archivedApps.map((app) => <div className="archived-application-row" key={app.id}><span className="company-logo">{app.companyMark}</span><button onClick={() => navigate(`/applications/${app.id}`)}><strong>{app.company}</strong><small>{app.role} · {app.city}</small></button><Badge tone="gray">{app.stage}</Badge><button className="button button--secondary" onClick={() => archiveApplication(app.id, false)}>恢复</button></div>)}</div>
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
                  positionTitle: String(data.get("positionTitle") || ""),
                  location: String(data.get("location") || ""),
                  channel: String(data.get("channel") || ""),
                  appliedAt: String(data.get("appliedAt") || ""),
                  jdRaw: String(data.get("jdRaw") || ""),
                  resumeProfileId: String(data.get("resumeProfileId") || "") || undefined,
                });
                close();
                showToast("已创建投递并关联简历版本。");
              } catch {
                // 数据层已记录可展示的错误信息，保留表单供用户修改后重试。
              } finally {
                setSaving(false);
              }
            }}>
              <div className="form-grid">
                <label><span>公司名称 *</span><input name="companyName" required placeholder="例如：蚂蚁集团" /></label>
                <label><span>岗位名称 *</span><input name="positionTitle" required placeholder="例如：后端开发工程师" /></label>
                <label><span>工作地点</span><input name="location" placeholder="杭州" /></label>
                <label><span>投递渠道</span><select name="channel"><option>招聘官网</option><option>Boss 直聘</option><option>内推</option><option>其他</option></select></label>
                <label><span>投递日期</span><input name="appliedAt" type="date" defaultValue={new Date().toLocaleDateString("en-CA")} /></label>
                <label><span>使用简历</span><select key={resumes.map((item) => item.id).join("|")} name="resumeProfileId" defaultValue={resumes.find((item) => item.isPrimary)?.id ?? ""}><option value="">暂不关联</option>{resumes.map((resume) => <option key={resume.id} value={resume.id}>{resume.name}{resume.targetDirection ? ` · ${resume.targetDirection}` : ""}{resume.isPrimary ? "（默认）" : ""}</option>)}</select></label>
                <label className="full"><span>JD 原文</span><textarea name="jdRaw" rows={5} placeholder="粘贴岗位描述，后续将用于岗位准备与问题预测" /></label>
              </div>
              <div className="dialog-actions">
                <button type="button" className="button button--secondary" onClick={close} disabled={saving}>取消</button>
                <button className="button button--primary" disabled={saving}>{saving ? "保存中…" : "保存投递"}</button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
