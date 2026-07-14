import { useEffect, useMemo, useState } from "react";
import { Columns3, Filter, GripVertical, LayoutList, MapPin, Plus, Search, X } from "lucide-react";
import { useSearchParams } from "react-router-dom";
import { DndContext, type DragEndEvent, useDraggable, useDroppable } from "@dnd-kit/core";
import { applications as initialApps, columnStages, stageToneMap } from "../data/mock";
import { Badge, Card, PageHeader } from "../components/ui";
import type { Application } from "../types";

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

function DraggableCard({ app }: { app: Application }) {
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
        <span>{app.priority}优先级</span>
        <time>{app.updated}更新</time>
      </div>
    </Card>
  );
}

function RejectedItem({ app }: { app: Application }) {
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
  const [params, setParams] = useSearchParams();
  const [view, setView] = useState<"board" | "list">("board");
  const [query, setQuery] = useState("");
  const [showNew, setShowNew] = useState(params.get("new") === "1");
  const [created, setCreated] = useState(false);
  const [apps, setApps] = useState<Application[]>(() => initialApps.map(a => ({ ...a })));

  useEffect(() => { setShowNew(params.get("new") === "1"); }, [params]);

  const filtered = useMemo(
    () => apps.filter(a => `${a.company}${a.role}${a.city}`.toLowerCase().includes(query.toLowerCase())),
    [query, apps],
  );

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;
    if (!over) return;
    const target = over.id as string;
    const newStage = columnStages[target];
    if (!newStage) return;

    setApps(prev => prev.map(a => {
      if (a.id !== active.id) return a;
      if (a.stage === newStage) return a;
      return { ...a, stage: newStage, stageTone: (stageToneMap[target] || a.stageTone) as Application["stageTone"] };
    }));
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
      {created && <div className="success-banner">已创建演示投递，后续可在岗位详情中补充 JD 与简历版本。<button onClick={() => setCreated(false)}><X size={15} /></button></div>}
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
                    {items.map(app => <DraggableCard key={app.id} app={app} />)}
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
                <RejectedItem key={app.id} app={app} />
              ))}
            </div>
          </RejectedDroppable>
        </DndContext>
      ) : (
        <Card className="table-card">
          <table>
            <thead>
              <tr>
                <th>公司 / 岗位</th><th>地点</th><th>当前阶段</th><th>下一步</th><th>优先级</th><th>最近更新</th>
              </tr>
            </thead>
            <tbody>
              {filtered.map(app => (
                <tr key={app.id}>
                  <td><span className="company-logo">{app.companyMark}</span><span><strong>{app.company}</strong><small>{app.role}</small></span></td>
                  <td>{app.city}</td>
                  <td><Badge tone={app.stageTone}>{app.stage}</Badge></td>
                  <td>{app.nextStep}<small>{app.nextTime}</small></td>
                  <td>{app.priority}</td>
                  <td>{app.updated}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </Card>
      )}
      {showNew && (
        <div className="modal-backdrop">
          <div className="dialog application-dialog">
            <div className="dialog-head">
              <div><h2>新增投递</h2><p>先记录核心信息，稍后可继续完善岗位档案</p></div>
              <button onClick={close}><X size={19} /></button>
            </div>
            <form onSubmit={e => { e.preventDefault(); close(); setCreated(true); }}>
              <div className="form-grid">
                <label><span>公司名称 *</span><input required placeholder="例如：蚂蚁集团" /></label>
                <label><span>岗位名称 *</span><input required placeholder="例如：后端开发工程师" /></label>
                <label><span>工作地点</span><input placeholder="杭州" /></label>
                <label><span>投递渠道</span><select><option>招聘官网</option><option>Boss 直聘</option><option>内推</option><option>其他</option></select></label>
                <label><span>投递日期</span><input type="date" defaultValue="2026-07-13" /></label>
                <label><span>使用简历</span><select><option>后端开发-2026.07</option><option>通用版-2026.06</option></select></label>
                <label className="full"><span>JD 原文</span><textarea rows={5} placeholder="粘贴岗位描述，后续将用于岗位准备与问题预测" /></label>
              </div>
              <div className="dialog-actions">
                <button type="button" className="button button--secondary" onClick={close}>取消</button>
                <button className="button button--primary">保存投递</button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
