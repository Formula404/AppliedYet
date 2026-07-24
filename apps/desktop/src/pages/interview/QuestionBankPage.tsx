import { useEffect, useRef, useState } from "react";
import { ChevronLeft, ChevronRight, Eye, Pencil, Plus, Search, X } from "lucide-react";
import { Badge, Card } from "../../components/ui";
import {
  getQuestionBankItem,
  listQuestionMatchCandidates,
  listQuestionBankItems,
  resolveQuestionMatch,
  saveQuestionBankItem,
  type QuestionBankDetail,
  type QuestionBankItem,
  type QuestionMatchCandidate,
  type SaveQuestionBankInput,
} from "../../services/interviews";
import { hasLocalDatabase } from "../../services/applications";
import { showError, showSuccess } from "../../services/feedback";
import { trackOperation } from "../../services/operations";

const emptyDraft: SaveQuestionBankInput = {
  prompt: "",
  category: "专业知识",
  bestAnswer: "",
  mastery: "待加强",
};
const masteryWidth: Record<QuestionBankItem["mastery"], number> = {
  "待加强": 25,
  "练习中": 50,
  "熟悉": 75,
  "掌握": 100,
};
const recordLabels: Record<string, string> = {
  real_asked: "真实面试",
  mock_answered: "模拟练习",
  reference_mentioned: "面经",
};
export default function QuestionBankPage() {
  const [items, setItems] = useState<QuestionBankItem[]>([]);
  const [total, setTotal] = useState(0);
  const [allCount, setAllCount] = useState(0);
  const [dueCount, setDueCount] = useState(0);
  const [query, setQuery] = useState("");
  const [debouncedQuery, setDebouncedQuery] = useState("");
  const [filter, setFilter] = useState<"all" | "due">("all");
  const [cursors, setCursors] = useState<(string | undefined)[]>([undefined]);
  const [pageIndex, setPageIndex] = useState(0);
  const [nextCursor, setNextCursor] = useState<string>();
  const [editing, setEditing] = useState<QuestionBankItem | null>();
  const [detail, setDetail] = useState<QuestionBankDetail>();
  const [draft, setDraft] = useState<SaveQuestionBankInput>(emptyDraft);
  const [candidates, setCandidates] = useState<QuestionMatchCandidate[]>([]);
  const [allowSeparate, setAllowSeparate] = useState(false);
  const [saving, setSaving] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const listRequestId = useRef(0);

  useEffect(() => {
    const timer = window.setTimeout(() => setDebouncedQuery(query.trim()), 250);
    return () => window.clearTimeout(timer);
  }, [query]);

  useEffect(() => {
    setCursors([undefined]);
    setPageIndex(0);
  }, [debouncedQuery, filter]);

  useEffect(() => {
    const requestId = ++listRequestId.current;
    setLoading(true);
    listQuestionBankItems({
      query: debouncedQuery,
      status: "active",
      reviewState: filter === "due" ? "due" : undefined,
      sort: "review_priority",
      direction: "desc",
      pageSize: 30,
      cursor: cursors[pageIndex],
    }).then((page) => {
      if (requestId !== listRequestId.current) return;
      setItems(page.items);
      setTotal(page.total);
      setAllCount(page.facets.active);
      setDueCount(page.facets.due);
      setNextCursor(page.nextCursor);
      setError("");
    }).catch((reason) => {
      if (requestId === listRequestId.current) setError(String(reason));
    }).finally(() => {
      if (requestId === listRequestId.current) setLoading(false);
    });
    return () => {
      if (requestId === listRequestId.current) listRequestId.current += 1;
    };
  }, [cursors, debouncedQuery, filter, pageIndex]);

  useEffect(() => {
    if (error) showError(error, "个人题库操作失败");
  }, [error]);

  const refresh = () => {
    const requestId = ++listRequestId.current;
    setLoading(true);
    listQuestionBankItems({
      query: debouncedQuery,
      status: "active",
      reviewState: filter === "due" ? "due" : undefined,
      sort: "review_priority",
      direction: "desc",
      pageSize: 30,
      cursor: cursors[pageIndex],
    }).then((page) => {
      if (requestId !== listRequestId.current) return;
      if (!page.items.length && pageIndex > 0) {
        setPageIndex((value) => value - 1);
        return;
      }
      setItems(page.items);
      setTotal(page.total);
      setAllCount(page.facets.active);
      setDueCount(page.facets.due);
      setNextCursor(page.nextCursor);
    }).catch((reason) => {
      if (requestId === listRequestId.current) setError(String(reason));
    }).finally(() => {
      if (requestId === listRequestId.current) setLoading(false);
    });
  };

  const openCreate = () => {
    setEditing(null);
    setDraft(emptyDraft);
    setCandidates([]);
    setAllowSeparate(false);
    setError("");
  };

  const openEdit = (item: QuestionBankItem) => {
    setEditing(item);
    setDraft({
      prompt: item.prompt,
      category: item.category,
      bestAnswer: item.bestAnswer,
      mastery: item.mastery,
    });
    setCandidates([]);
    setError("");
  };

  const save = async () => {
    if (!draft.prompt.trim()) {
      setError("问题不能为空");
      return;
    }
    setSaving(true);
    setError("");
    try {
      if (!editing && !allowSeparate) {
        const matches = await trackOperation("检查题库相似问题", () => listQuestionMatchCandidates(draft.prompt));
        if (matches.length) {
          setCandidates(matches);
          return;
        }
      }
      const saved = await trackOperation(editing ? "保存题库问题" : "添加题库问题", () => saveQuestionBankItem(editing?.id, {
        ...draft,
        forceNew: !editing && allowSeparate,
      }), draft.prompt);
      if (!editing && allowSeparate && candidates[0]) {
        await resolveQuestionMatch(
          saved.id,
          candidates[0].question.id,
          "keep_separate",
          "用户选择单独保存",
        );
      }
      setEditing(undefined);
      refresh();
      showSuccess(editing ? "问题与最佳回答已更新。" : "问题已加入个人题库。", editing ? "题库问题已保存" : "题库问题已添加");
    } catch (reason) {
      setError(String(reason));
    } finally {
      setSaving(false);
    }
  };

  const useExisting = async (item: QuestionBankItem) => {
    setSaving(true);
    try {
      await trackOperation("更新已有题库问题", () => saveQuestionBankItem(item.id, {
        prompt: item.prompt,
        category: item.category,
        bestAnswer: item.bestAnswer,
        mastery: item.mastery,
      }), item.prompt);
      setEditing(undefined);
      refresh();
      showSuccess("已使用并更新已有问题。", "题库已更新");
    } catch (reason) {
      setError(String(reason));
    } finally {
      setSaving(false);
    }
  };

  const showDetail = async (item: QuestionBankItem) => {
    if (!hasLocalDatabase) return;
    try {
      setDetail(await trackOperation("读取题库问题详情", () => getQuestionBankItem(item.id), item.prompt));
    } catch (reason) {
      setError(String(reason));
    }
  };

  const visibleRecords = detail?.evidence.filter((record) => recordLabels[record.eventType]) ?? [];

  return <>
    <div className="question-bank-toolbar">
      <div className="knowledge-tabs">
        <button className={filter === "all" ? "active" : ""} onClick={() => setFilter("all")}>
          全部题目 <b>{allCount}</b>
        </button>
        <button className={filter === "due" ? "active" : ""} onClick={() => setFilter("due")}>
          需要复习 <b>{dueCount}</b>
        </button>
      </div>
      <div className="question-bank-actions">
        <label>
          <Search size={15}/>
          <input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="搜索题目"/>
        </label>
        <button className="button button--primary" disabled={!hasLocalDatabase} onClick={openCreate}>
          <Plus size={15}/>添加问题
        </button>
      </div>
    </div>


    <Card className="table-card question-bank-table question-bank-table--simple">
      <table>
        <colgroup>
          <col className="question-bank-col-question"/>
          <col className="question-bank-col-type"/>
          <col className="question-bank-col-practice"/>
          <col className="question-bank-col-mastery"/>
          <col className="question-bank-col-actions"/>
        </colgroup>
        <thead><tr><th>问题</th><th>类型</th><th>练习情况</th><th>掌握程度</th><th>操作</th></tr></thead>
        <tbody>{items.map((item) => <tr key={item.id}>
          <td>
            <strong>{item.prompt}</strong>
          </td>
          <td>
            <Badge tone={item.category === "行为面试" ? "purple" : item.category === "项目深挖" ? "orange" : "blue"}>
              {item.category}
            </Badge>
          </td>
          <td>
            <button className="question-bank-practice" onClick={() => showDetail(item)}>
              <span><b>{item.realInterviewCount}</b><small>面试</small></span>
              <span><b>{item.practiceCount}</b><small>练习</small></span>
              <span><b>{item.referenceCount}</b><small>面经</small></span>
            </button>
          </td>
          <td>
            <div className="question-bank-mastery">
              <span className="mastery"><i style={{ width: `${masteryWidth[item.mastery]}%` }}/></span>
              <span>{item.mastery}</span>
              {item.needsReview && <small>建议复习</small>}
            </div>
          </td>
          <td>
            <div className="question-bank-row-actions">
              <button title="查看练习记录" onClick={() => showDetail(item)}><Eye size={14}/></button>
              <button title="编辑问题" onClick={() => openEdit(item)}><Pencil size={14}/></button>
            </div>
          </td>
        </tr>)}</tbody>
      </table>

      {!items.length && <div className="question-bank-empty">
        {loading ? "正在加载…" : query ? "没有找到相关问题" : filter === "due" ? "目前没有需要复习的问题" : "题库还没有内容，先添加一道想练习的问题吧。"}
      </div>}

      {(pageIndex > 0 || nextCursor) && <div className="question-bank-pagination question-bank-pagination--simple">
        <span>共 {total} 道题</span>
        <button disabled={pageIndex === 0} onClick={() => setPageIndex((value) => value - 1)}>
          <ChevronLeft size={15}/>上一页
        </button>
        <button disabled={!nextCursor} onClick={() => {
          if (!nextCursor) return;
          setCursors((current) => [...current.slice(0, pageIndex + 1), nextCursor]);
          setPageIndex((value) => value + 1);
        }}>
          下一页<ChevronRight size={15}/>
        </button>
      </div>}
    </Card>

    {detail && <div className="modal-backdrop" onMouseDown={() => setDetail(undefined)}>
      <div className="dialog question-bank-dialog question-bank-detail" onMouseDown={(event) => event.stopPropagation()}>
        <div className="dialog-head">
          <div><h2>{detail.prompt}</h2><p>{detail.category} · {detail.mastery}</p></div>
          <button onClick={() => setDetail(undefined)}><X size={18}/></button>
        </div>
        <div className="question-bank-detail-body">
          <div className="question-bank-detail-stats">
            <span><b>{detail.realInterviewCount}</b><small>真实面试遇到</small></span>
            <span><b>{detail.practiceCount}</b><small>模拟练习</small></span>
            <span><b>{detail.referenceCount}</b><small>面经提及</small></span>
          </div>
          <section>
            <h3>参考回答</h3>
            <p className={detail.bestAnswer ? "" : "muted"}>{detail.bestAnswer || "还没有整理参考回答，可以点击编辑补充。"}</p>
          </section>
          {detail.variants.length > 1 && <section>
            <h3>其他问法</h3>
            <ul>{detail.variants.filter((variant) => variant !== detail.prompt).map((variant) => <li key={variant}>{variant}</li>)}</ul>
          </section>}
          <section>
            <h3>练习记录</h3>
            {visibleRecords.length ? <div className="question-bank-records">{visibleRecords.map((record) => <article key={record.id}>
              <Badge tone={record.eventType === "real_asked" ? "purple" : record.eventType === "mock_answered" ? "green" : "blue"}>
                {recordLabels[record.eventType]}
              </Badge>
              <div>
                <strong>{record.company ? `${record.company}${record.position ? ` · ${record.position}` : ""}` : record.prompt}</strong>
                <small>{new Date(record.occurredAt).toLocaleString("zh-CN")}{record.round ? ` · ${record.round}` : ""}</small>
              </div>
            </article>)}</div> : <p className="muted">还没有面试或练习记录。</p>}
          </section>
        </div>
      </div>
    </div>}

    {editing !== undefined && <div className="modal-backdrop" onMouseDown={() => setEditing(undefined)}>
      <div className="dialog question-bank-dialog" onMouseDown={(event) => event.stopPropagation()}>
        <div className="dialog-head">
          <div><h2>{editing ? "编辑问题" : "添加问题"}</h2><p>整理你想反复练习的问题和参考回答</p></div>
          <button onClick={() => setEditing(undefined)}><X size={18}/></button>
        </div>
        <div className="question-bank-form">
          <label>
            <span>问题 *</span>
            <textarea rows={3} value={draft.prompt} onChange={(event) => {
              setDraft({ ...draft, prompt: event.target.value });
              setCandidates([]);
              setAllowSeparate(false);
            }}/>
          </label>
          <div>
            <label>
              <span>类型</span>
              <select value={draft.category} onChange={(event) => setDraft({ ...draft, category: event.target.value })}>
                {["专业知识", "行为面试", "岗位动机", "项目深挖", "其他"].map((value) => <option key={value}>{value}</option>)}
              </select>
            </label>
            <label>
              <span>当前掌握</span>
              <select value={draft.mastery} onChange={(event) => setDraft({ ...draft, mastery: event.target.value as QuestionBankItem["mastery"] })}>
                {["待加强", "练习中", "熟悉", "掌握"].map((value) => <option key={value}>{value}</option>)}
              </select>
            </label>
          </div>
          <label>
            <span>参考回答（可选）</span>
            <textarea rows={6} value={draft.bestAnswer} onChange={(event) => setDraft({ ...draft, bestAnswer: event.target.value })} placeholder="记录回答思路、关键点或示例"/>
          </label>
          {candidates.length > 0 && !allowSeparate && <div className="question-match-suggestions">
            <strong>题库里可能已经有相似的问题</strong>
            {candidates.slice(0, 3).map((candidate) => <div key={candidate.question.id}>
              <span>{candidate.question.prompt}</span>
              <button className="button button--secondary" onClick={() => void useExisting(candidate.question)}>使用已有问题</button>
            </div>)}
            <button className="question-match-create-anyway" onClick={() => setAllowSeparate(true)}>这不是同一道题，仍然新建</button>
          </div>}
          {error && <p className="field-error">{error}</p>}
          <div className="form-actions">
            <button className="button button--secondary" onClick={() => setEditing(undefined)}>取消</button>
            <button className="button button--primary" disabled={saving} onClick={save}>{saving ? "保存中…" : "保存"}</button>
          </div>
        </div>
      </div>
    </div>}
  </>;
}
