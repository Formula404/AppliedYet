import { useEffect, useMemo, useState } from "react";
import { Pencil, Plus, Search, Trash2, X } from "lucide-react";
import { Badge, Card } from "../../components/ui";
import {
  deleteQuestionBankItem,
  listQuestionBankItems,
  saveQuestionBankItem,
  type QuestionBankItem,
  type SaveQuestionBankInput,
} from "../../services/interviews";
import { hasLocalDatabase } from "../../services/applications";

const emptyDraft: SaveQuestionBankInput = { prompt: "", category: "专业知识", bestAnswer: "", mastery: "待加强" };
const masteryWidth: Record<QuestionBankItem["mastery"], number> = { "待加强": 25, "练习中": 50, "熟悉": 75, "掌握": 100 };

export default function QuestionBankPage() {
  const [items, setItems] = useState<QuestionBankItem[]>([]);
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState<"全部" | "薄弱项">("全部");
  const [editing, setEditing] = useState<QuestionBankItem | null>();
  const [draft, setDraft] = useState<SaveQuestionBankInput>(emptyDraft);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");

  const refresh = () => listQuestionBankItems().then(setItems).catch((reason) => setError(String(reason)));
  useEffect(() => { if (hasLocalDatabase) refresh(); }, []);

  const filtered = useMemo(() => items.filter((item) => {
    const matchesFilter = filter === "全部" || item.mastery === "待加强" || item.mastery === "练习中";
    const keyword = query.trim().toLowerCase();
    return matchesFilter && (!keyword || `${item.prompt} ${item.category} ${item.source}`.toLowerCase().includes(keyword));
  }), [filter, items, query]);
  const weakCount = items.filter((item) => item.mastery === "待加强" || item.mastery === "练习中").length;

  const openCreate = () => { setEditing(null); setDraft(emptyDraft); setError(""); };
  const openEdit = (item: QuestionBankItem) => {
    setEditing(item);
    setDraft({ prompt: item.prompt, category: item.category, bestAnswer: item.bestAnswer, mastery: item.mastery });
    setError("");
  };
  const save = async () => {
    if (!draft.prompt.trim()) { setError("问题不能为空"); return; }
    setSaving(true); setError("");
    try {
      const saved = await saveQuestionBankItem(editing?.id, draft);
      setItems((current) => editing ? current.map((item) => item.id === saved.id ? saved : item) : [saved, ...current]);
      setEditing(undefined);
    } catch (reason) { setError(String(reason)); }
    finally { setSaving(false); }
  };
  const remove = async (item: QuestionBankItem) => {
    if (!window.confirm(`确定从个人题库删除这道题吗？\n\n${item.prompt}`)) return;
    try { await deleteQuestionBankItem(item.id); setItems((current) => current.filter((value) => value.id !== item.id)); }
    catch (reason) { setError(String(reason)); }
  };

  return <>
    <div className="question-bank-toolbar"><div className="knowledge-tabs"><button className={filter === "全部" ? "active" : ""} onClick={() => setFilter("全部")}>全部问题 <b>{items.length}</b></button><button className={filter === "薄弱项" ? "active" : ""} onClick={() => setFilter("薄弱项")}>薄弱项 <b>{weakCount}</b></button></div><div className="question-bank-actions"><label><Search size={15}/><input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="搜索问题、类型或来源"/></label><button className="button button--primary" disabled={!hasLocalDatabase} onClick={openCreate}><Plus size={15}/>新建问题</button></div></div>
    {error && editing === undefined && <p className="detail-error">{error}</p>}
    <Card className="table-card question-bank-table"><table><thead><tr><th>问题</th><th>类型</th><th>出现次数</th><th>掌握程度</th><th>最近出现</th><th>操作</th></tr></thead><tbody>{filtered.map((item) => <tr key={item.id}><td><strong>{item.prompt}</strong><small>来源：{item.source}{item.bestAnswer ? " · 已整理参考回答" : ""}</small></td><td><Badge tone={item.category === "行为面试" ? "purple" : item.category === "项目深挖" ? "orange" : "blue"}>{item.category}</Badge></td><td>{item.occurrenceCount} 次</td><td><span className="mastery"><i style={{ width: `${masteryWidth[item.mastery]}%` }}/></span>{item.mastery}</td><td>{new Date(item.lastSeenAt).toLocaleDateString("zh-CN")}</td><td><div className="question-bank-row-actions"><button title="编辑" onClick={() => openEdit(item)}><Pencil size={14}/></button><button title="删除" onClick={() => remove(item)}><Trash2 size={14}/></button></div></td></tr>)}</tbody></table>{!filtered.length && <div className="question-bank-empty">{items.length ? "没有符合当前筛选的问题" : "题库还没有内容。创建模拟面试或导入真实面试后，出现过的问题会自动沉淀到这里。"}</div>}</Card>
    {editing !== undefined && <div className="modal-backdrop" onMouseDown={() => setEditing(undefined)}><div className="dialog question-bank-dialog" onMouseDown={(event) => event.stopPropagation()}><div className="dialog-head"><div><h2>{editing ? "编辑题库问题" : "新建题库问题"}</h2><p>手工维护参考回答和掌握程度；面试中出现过的问题会自动统计次数。</p></div><button onClick={() => setEditing(undefined)}><X size={18}/></button></div><div className="question-bank-form"><label><span>问题 *</span><textarea rows={3} value={draft.prompt} onChange={(event) => setDraft({ ...draft, prompt: event.target.value })}/></label><div><label><span>类型</span><select value={draft.category} onChange={(event) => setDraft({ ...draft, category: event.target.value })}>{["专业知识", "行为面试", "岗位动机", "项目深挖", "其他"].map((value) => <option key={value}>{value}</option>)}</select></label><label><span>掌握程度</span><select value={draft.mastery} onChange={(event) => setDraft({ ...draft, mastery: event.target.value as QuestionBankItem["mastery"] })}>{["待加强", "练习中", "熟悉", "掌握"].map((value) => <option key={value}>{value}</option>)}</select></label></div><label><span>参考回答</span><textarea rows={6} value={draft.bestAnswer} onChange={(event) => setDraft({ ...draft, bestAnswer: event.target.value })} placeholder="整理你认可的回答结构、关键事实和证据"/></label>{error && <p className="field-error">{error}</p>}<div className="form-actions"><button className="button button--secondary" onClick={() => setEditing(undefined)}>取消</button><button className="button button--primary" disabled={saving} onClick={save}>{saving ? "保存中…" : "保存"}</button></div></div></div></div>}
  </>;
}
