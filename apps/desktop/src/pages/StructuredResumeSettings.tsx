import { useEffect, useState } from "react";
import { Archive, Check, Copy, Plus, RotateCcw, Trash2, Upload, UserRound } from "lucide-react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { Card, CardHeader } from "../components/ui";
import { hasLocalDatabase } from "../services/applications";
import { createBlankResumeProfile, deleteResumeProfile, duplicateResumeProfile, importResumeProfile, listResumeProfiles, setPrimaryResumeProfile, setResumeProfileArchived, updateResumeProfile, type ResumeProfile, type UpdateResumeProfileInput } from "../services/resumes";
import { getCredentialStatus, getProviderSettings } from "../services/settings";

type Personal = { name: string; birthday: string; contact: string; links: string };
type Education = { startDate: string; endDate: string; school: string; degree: string; major: string };
type Experience = { company: string; role: string; startDate: string; endDate: string; description: string };
type Project = { name: string; role: string; startDate: string; endDate: string; description: string; technologies: string };
type Academic = { title: string; kind: string; date: string; description: string; link: string };
type Draft = { name: string; targetDirection: string; notes: string; personal: Personal; education: Education[]; internships: Experience[]; projects: Project[]; skills: string; academics: Academic[]; certificates: string[] };

const emptyPersonal = (): Personal => ({ name: "", birthday: "", contact: "", links: "" });
const emptyEducation = (): Education => ({ startDate: "", endDate: "", school: "", degree: "", major: "" });
const emptyExperience = (): Experience => ({ company: "", role: "", startDate: "", endDate: "", description: "" });
const emptyProject = (): Project => ({ name: "", role: "", startDate: "", endDate: "", description: "", technologies: "" });
const emptyAcademic = (): Academic => ({ title: "", kind: "", date: "", description: "", link: "" });

export default function StructuredResumeSettings({ onMessage, onError }: { onMessage: (message: string) => void; onError: (message: string) => void }) {
  const [profiles, setProfiles] = useState<ResumeProfile[]>([]);
  const [selectedId, setSelectedId] = useState("");
  const [draft, setDraft] = useState<Draft>();
  const [busy, setBusy] = useState(false);
  const [aiConfigured, setAiConfigured] = useState(false);
  const [allowAiResume, setAllowAiResume] = useState(false);
  const [promptBeforeAiSend, setPromptBeforeAiSend] = useState(true);

  useEffect(() => {
    listResumeProfiles().then((items) => { setProfiles(items); const selected = items.find((item) => item.isPrimary) ?? items[0]; if (selected) { setSelectedId(selected.id); setDraft(toDraft(selected)); } }).catch((reason) => onError(String(reason)));
  }, [onError]);
  useEffect(() => { if (hasLocalDatabase) Promise.all([getProviderSettings(), getCredentialStatus("ai_api_key")]).then(([{ ai }, hasKey]) => { setAiConfigured(Boolean(ai.baseUrl && ai.model && hasKey)); setAllowAiResume(ai.allowResume); setPromptBeforeAiSend(ai.promptBeforeSend); }).catch(() => setAiConfigured(false)); }, []);

  const selected = profiles.find((item) => item.id === selectedId);
  const createBlank = async () => { setBusy(true); onError(""); try { const created = await createBlankResumeProfile(`新建简历 · ${new Date().toLocaleDateString("zh-CN")}`); const items = await listResumeProfiles(); setProfiles(items); setSelectedId(created.id); setDraft(toDraft(created)); onMessage("已创建空白简历，可直接填写结构化内容"); } catch (reason) { onError(String(reason)); } finally { setBusy(false); } };
  const importFiles = async () => {
    const paths = await openDialog({ multiple: true, directory: false, filters: [{ name: "简历文件", extensions: ["pdf", "docx", "txt", "md"] }] });
    if (!paths || typeof paths === "string") return;
    const confirmAiSend = aiConfigured && allowAiResume && (!promptBeforeAiSend || window.confirm(`将把选中的 ${paths.length} 份简历发送给 AI 服务进行信息提取。是否继续？\n\n选择”取消”仍会导入文件，但只做基础解析。`));
    setBusy(true); onError("");
    try {
      const outcomes = await Promise.allSettled(paths.map((path) => importResumeProfile(path, confirmAiSend)));
      const imported = outcomes.flatMap((outcome) => outcome.status === "fulfilled" ? [outcome.value] : []);
      const failures = outcomes.flatMap((outcome) => outcome.status === "rejected" ? [String(outcome.reason)] : []);
      const items = await listResumeProfiles(); setProfiles(items);
      const latestId = imported.at(-1)?.profile.id;
      const latest = items.find((item) => item.id === latestId);
      if (latest) { setSelectedId(latest.id); setDraft(toDraft(latest)); }
      const warnings = imported.flatMap((result) => result.warning ? [result.warning] : []);
      const aiCount = imported.filter((result) => result.aiStatus === "succeeded").length;
      if (warnings.length || failures.length) onError([...new Set([...warnings, ...failures])].join("；"));
      if (imported.length) onMessage(`已添加 ${imported.length} 份简历${aiCount ? `，其中 ${aiCount} 份已由 AI 提取信息` : ""}${failures.length ? `；${failures.length} 份导入失败` : ""}，请检查结果`);
      else onError(`全部 ${failures.length} 份简历解析失败：${[...new Set(failures)].join("；")}`);
    } catch (reason) { onError(`简历解析失败：${String(reason)}`); } finally { setBusy(false); }
  };
  const select = (profile: ResumeProfile) => { setSelectedId(profile.id); setDraft(toDraft(profile)); onMessage(""); onError(""); };
  const save = async () => {
    if (!draft || !selectedId) return;
    setBusy(true); onError("");
    try { const updated = await updateResumeProfile(selectedId, serializeDraft(draft)); const items = await listResumeProfiles(); setProfiles(items); setSelectedId(updated.id); setDraft(toDraft(updated)); onMessage(selected?.linkedApplicationCount ? `简历已更新，${selected.linkedApplicationCount} 个关联投递将使用最新内容` : "简历已保存"); } catch (reason) { onError(String(reason)); } finally { setBusy(false); }
  };
  const makePrimary = async () => { if (!selectedId) return; try { await setPrimaryResumeProfile(selectedId); setProfiles((items) => items.map((item) => ({ ...item, isPrimary: item.id === selectedId }))); onMessage("已设为默认简历"); } catch (reason) { onError(String(reason)); } };
  const remove = async () => { if (!selectedId || !selected) return; const linked = selected.linkedApplicationCount > 0; if (!window.confirm(linked ? `这份简历关联了 ${selected.linkedApplicationCount} 个投递。删除后会解除这些关联，确定继续吗？` : "确定永久删除这份简历吗？")) return; try { await deleteResumeProfile(selectedId); const items = await listResumeProfiles(); setProfiles(items); const next = items.find((item) => !item.archivedAt) ?? items[0]; setSelectedId(next?.id ?? ""); setDraft(next ? toDraft(next) : undefined); onMessage(linked ? "简历已删除，相关投递已解除关联" : "简历已删除"); } catch (reason) { onError(String(reason)); } };
  const duplicate = async () => { if (!selectedId) return; try { const copy = await duplicateResumeProfile(selectedId); const items = await listResumeProfiles(); setProfiles(items); setSelectedId(copy.id); setDraft(toDraft(copy)); onMessage("已复制为独立简历版本"); } catch (reason) { onError(String(reason)); } };
  const toggleArchived = async () => { if (!selected) return; try { await setResumeProfileArchived(selected.id, !selected.archivedAt); const items = await listResumeProfiles(); setProfiles(items); const updated = items.find((item) => item.id === selected.id); if (updated) setDraft(toDraft(updated)); onMessage(selected.archivedAt ? "简历已恢复" : "简历已归档"); } catch (reason) { onError(String(reason)); } };
  const set = <K extends keyof Draft>(key: K, value: Draft[K]) => setDraft((current) => current ? ({ ...current, [key]: value }) : current);

  return <div className="resume-page">
    {(!aiConfigured || !allowAiResume) && <div className="resume-ai-hint">{!aiConfigured ? "配置 AI 服务后，导入简历时能自动识别更多信息。" : "当前未授权发送简历给 AI 服务，导入时仅做基础解析。"}你也可以继续手动填写。</div>}
    <div className="resume-page-head"><div><span className="eyebrow">个人资料库</span><h2>我的简历</h2><p>不同岗位可以保留不同版本。上传后请逐项检查，再保存使用。</p></div><div className="resume-toolbar-actions"><button type="button" className="button button--secondary" disabled={busy} onClick={createBlank}><Plus size={15} />新建空白</button><button type="button" className="button button--primary" disabled={!hasLocalDatabase || busy} onClick={importFiles}><Upload size={15} />{busy ? "正在解析…" : "上传简历"}</button></div></div>
    <div className="resume-settings"><Card className="resume-list-card"><CardHeader title="简历版本" subtitle={`${profiles.filter((item) => !item.archivedAt).length} 份使用中 · ${profiles.filter((item) => item.archivedAt).length} 份已归档`} /><div className="resume-list">{profiles.map((profile) => <button type="button" key={profile.id} className={`${profile.id === selectedId ? "active" : ""} ${profile.archivedAt ? "is-archived" : ""}`} onClick={() => select(profile)}><span className="resume-list-icon">{profile.isPrimary ? <Check size={15} /> : profile.archivedAt ? <Archive size={15} /> : <UserRound size={15} />}</span><span><strong>{profile.name}</strong><small>{profile.archivedAt ? "已归档" : profile.targetDirection || profile.fileFormat?.toUpperCase() || "手动创建"} · {profile.linkedApplicationCount} 个投递</small></span><span className="resume-list-arrow">›</span></button>)}{!profiles.length && <div className="resume-empty"><UserRound size={28} /><p>还没有简历<br />点击右上角添加第一份</p></div>}</div><div className="resume-list-tip">修改会直接更新当前简历，关联的投递也会同步更新。</div></Card>
      <Card className="resume-editor-card"><CardHeader title={draft ? "编辑当前简历" : "开始建立简历"} subtitle={draft ? "按条目填写每一段经历，可以随时添加或删除" : "上传简历后，这里会出现结构化编辑器"} />{draft ? <><div className="resume-editor-toolbar"><div className="resume-name-field"><span>当前版本名称</span><input value={draft.name} disabled={Boolean(selected?.archivedAt)} onChange={(event) => set("name", event.target.value)} placeholder="例如：后端开发 · 2026 春招" /></div><div className="resume-toolbar-actions"><button type="button" className="button button--secondary" onClick={duplicate}><Copy size={14} />复制版本</button><button type="button" className="button button--secondary" disabled={Boolean(selected?.archivedAt)} onClick={makePrimary}>{selected?.isPrimary ? <Check size={14} /> : null}{selected?.isPrimary ? "默认简历" : "设为默认"}</button><button type="button" className="button button--secondary" onClick={toggleArchived}>{selected?.archivedAt ? <RotateCcw size={14} /> : <Archive size={14} />}{selected?.archivedAt ? "恢复" : "归档"}</button><button type="button" className="button button--secondary danger-text" onClick={remove}><Trash2 size={14} />删除</button></div></div><div className="resume-editor-note"><Check size={14} /> {selected?.linkedApplicationCount ? `已关联 ${selected.linkedApplicationCount} 个投递；保存后这些投递会使用最新内容。` : "已解析完成，请检查每项内容是否准确。"}</div><div className="structured-resume-form">
        <Section title="版本信息" description="管理版本适用方向和说明"><div className="form-grid form-grid--four"><Input label="适用岗位方向" value={draft.targetDirection} onChange={(value) => set("targetDirection", value)} placeholder="例如：Java 后端 / 数据开发" /><Textarea label="版本说明" value={draft.notes} onChange={(value) => set("notes", value)} placeholder="记录针对哪些岗位做过哪些调整" /></div>{selected && selected.linkedApplicationCount > 0 && <div className="resume-editor-note">版本关联统计：投递 {selected.linkedApplicationCount} · 测评 {selected.assessmentCount}（{rate(selected.assessmentCount, selected.linkedApplicationCount)}）· 面试 {selected.interviewCount}（{rate(selected.interviewCount, selected.linkedApplicationCount)}）· Offer {selected.offerCount}（{rate(selected.offerCount, selected.linkedApplicationCount)}）</div>}</Section>
        <Section title="个人信息" description="用于简历抬头和联系方式"><div className="form-grid form-grid--four"><Input label="姓名" value={draft.personal.name} onChange={(value) => set("personal", { ...draft.personal, name: value })} /><Input label="生日" type="date" value={draft.personal.birthday} onChange={(value) => set("personal", { ...draft.personal, birthday: value })} /><Input label="联系方式" value={draft.personal.contact} onChange={(value) => set("personal", { ...draft.personal, contact: value })} placeholder="手机 / 邮箱" /><Input label="个人链接" value={draft.personal.links} onChange={(value) => set("personal", { ...draft.personal, links: value })} placeholder="GitHub / 作品集 / LinkedIn" /></div></Section>
        <Section title="教育背景" description="按时间倒序添加学校经历" action={<AddButton onClick={() => set("education", [...draft.education, emptyEducation()])} />}>{draft.education.map((item, index) => <EducationRow key={index} item={item} onChange={(value) => set("education", replaceAt(draft.education, index, value))} onRemove={() => set("education", removeAt(draft.education, index))} />)}</Section>
        <Section title="实习经历" description="每段经历独立填写，突出职责和结果" action={<AddButton onClick={() => set("internships", [...draft.internships, emptyExperience()])} />}>{draft.internships.map((item, index) => <ExperienceRow key={index} item={item} onChange={(value) => set("internships", replaceAt(draft.internships, index, value))} onRemove={() => set("internships", removeAt(draft.internships, index))} />)}</Section>
        <Section title="项目/科研/校园经历" description="项目名称、职责、技术和成果分开记录" action={<AddButton onClick={() => set("projects", [...draft.projects, emptyProject()])} />}>{draft.projects.map((item, index) => <ProjectRow key={index} item={item} onChange={(value) => set("projects", replaceAt(draft.projects, index, value))} onRemove={() => set("projects", removeAt(draft.projects, index))} />)}</Section>
        <Section title="专业技能" description="集中填写技能栈，不拆成多个条目"><label className="resume-input resume-skill-input"><span>技能栈</span><input value={draft.skills} onChange={(event) => set("skills", event.target.value)} placeholder="例如：Rust、React、PostgreSQL、Docker、Redis" /></label></Section>
        <Section title="学术成果" description="论文、专利、科研和竞赛成果" action={<AddButton onClick={() => set("academics", [...draft.academics, emptyAcademic()])} />}>{draft.academics.map((item, index) => <AcademicRow key={index} item={item} onChange={(value) => set("academics", replaceAt(draft.academics, index, value))} onRemove={() => set("academics", removeAt(draft.academics, index))} />)}</Section>
        <Section title="技能证书" description="以关键词卡片形式添加">{<KeywordEditor values={draft.certificates} onChange={(value) => set("certificates", value)} placeholder="例如：CET-6" />}</Section>
        {selected?.parsedText && <Section title="导入原文" description="查看从文件中提取的原始文本"><details><summary>查看原文内容</summary><pre className="jd-content">{selected.parsedText}</pre></details></Section>}
      </div><div className="resume-editor-actions"><span>修改后记得保存，切换版本不会自动保存。</span><button type="button" className="button button--primary" disabled={busy || Boolean(selected?.archivedAt)} onClick={save}>保存当前简历</button></div></> : <div className="resume-empty resume-editor-empty"><Upload size={34} /><h3>还没有选中的简历</h3><p>从右上角添加文件，系统会自动解析成结构化栏目。</p></div>}</Card>
    </div>
  </div>;
}

function Section({ title, description, action, children }: { title: string; description: string; action?: React.ReactNode; children: React.ReactNode }) { return <section className="resume-form-section"><div className="resume-section-heading"><div><h3>{title}</h3><p>{description}</p></div>{action}</div>{children}</section>; }
function AddButton({ onClick }: { onClick: () => void }) { return <button type="button" className="resume-add-button" onClick={onClick}><Plus size={14} />添加一条</button>; }
function Input({ label, value, onChange, type = "text", placeholder }: { label: string; value: string; onChange: (value: string) => void; type?: string; placeholder?: string }) { return <label className="resume-input"><span>{label}</span><input type={type} value={value} onChange={(event) => onChange(event.target.value)} placeholder={placeholder} /></label>; }
function Textarea({ label, value, onChange, placeholder }: { label: string; value: string; onChange: (value: string) => void; placeholder?: string }) { return <label className="resume-input resume-input--wide"><span>{label}</span><textarea rows={3} value={value} onChange={(event) => onChange(event.target.value)} placeholder={placeholder} /></label>; }
function EducationRow({ item, onChange, onRemove }: { item: Education; onChange: (value: Education) => void; onRemove: () => void }) { return <div className="resume-entry"><div className="form-grid form-grid--five"><Input label="开始时间" type="month" value={item.startDate} onChange={(value) => onChange({ ...item, startDate: value })} /><Input label="结束时间" type="month" value={item.endDate} onChange={(value) => onChange({ ...item, endDate: value })} /><Input label="院校" value={item.school} onChange={(value) => onChange({ ...item, school: value })} /><Input label="学位" value={item.degree} onChange={(value) => onChange({ ...item, degree: value })} /><Input label="专业" value={item.major} onChange={(value) => onChange({ ...item, major: value })} /></div><RemoveButton onClick={onRemove} /></div>; }
function ExperienceRow({ item, onChange, onRemove }: { item: Experience; onChange: (value: Experience) => void; onRemove: () => void }) { return <div className="resume-entry"><div className="form-grid form-grid--four"><Input label="公司" value={item.company} onChange={(value) => onChange({ ...item, company: value })} /><Input label="职位" value={item.role} onChange={(value) => onChange({ ...item, role: value })} /><Input label="开始时间" type="month" value={item.startDate} onChange={(value) => onChange({ ...item, startDate: value })} /><Input label="结束时间" type="month" value={item.endDate} onChange={(value) => onChange({ ...item, endDate: value })} /><Textarea label="工作内容与成果" value={item.description} onChange={(value) => onChange({ ...item, description: value })} placeholder="描述职责、技术和量化结果" /></div><RemoveButton onClick={onRemove} /></div>; }
function ProjectRow({ item, onChange, onRemove }: { item: Project; onChange: (value: Project) => void; onRemove: () => void }) { return <div className="resume-entry"><div className="form-grid form-grid--four"><Input label="项目名称" value={item.name} onChange={(value) => onChange({ ...item, name: value })} /><Input label="担任角色" value={item.role} onChange={(value) => onChange({ ...item, role: value })} /><Input label="开始时间" type="month" value={item.startDate} onChange={(value) => onChange({ ...item, startDate: value })} /><Input label="结束时间" type="month" value={item.endDate} onChange={(value) => onChange({ ...item, endDate: value })} /><Input label="技术栈" value={item.technologies} onChange={(value) => onChange({ ...item, technologies: value })} placeholder="Rust / React / PostgreSQL" /><Textarea label="项目内容与成果" value={item.description} onChange={(value) => onChange({ ...item, description: value })} /></div><RemoveButton onClick={onRemove} /></div>; }
function AcademicRow({ item, onChange, onRemove }: { item: Academic; onChange: (value: Academic) => void; onRemove: () => void }) { return <div className="resume-entry"><div className="form-grid form-grid--four"><Input label="成果名称" value={item.title} onChange={(value) => onChange({ ...item, title: value })} /><Input label="类型" value={item.kind} onChange={(value) => onChange({ ...item, kind: value })} placeholder="论文 / 专利 / 竞赛" /><Input label="时间" type="month" value={item.date} onChange={(value) => onChange({ ...item, date: value })} /><Input label="链接" value={item.link} onChange={(value) => onChange({ ...item, link: value })} /><Textarea label="成果说明" value={item.description} onChange={(value) => onChange({ ...item, description: value })} /></div><RemoveButton onClick={onRemove} /></div>; }
function RemoveButton({ onClick }: { onClick: () => void }) { return <button type="button" className="resume-remove-button" onClick={onClick} aria-label="删除此条"><Trash2 size={14} /></button>; }
function KeywordEditor({ values, onChange, placeholder }: { values: string[]; onChange: (values: string[]) => void; placeholder: string }) { const [value, setValue] = useState(""); const add = () => { if (value.trim()) { onChange([...values.filter(Boolean), value.trim()]); setValue(""); } }; return <div className="keyword-editor"><div className="keyword-list">{values.filter(Boolean).map((item, index) => <span className="keyword-chip" key={`${item}-${index}`}>{item}<button type="button" onClick={() => onChange(values.filter((_, itemIndex) => itemIndex !== index))}>×</button></span>)}</div><div className="keyword-input"><input value={value} onChange={(event) => setValue(event.target.value)} onKeyDown={(event) => { if (event.key === "Enter") { event.preventDefault(); add(); } }} placeholder={placeholder} /><button type="button" className="button button--secondary" onClick={add}><Plus size={14} />添加</button></div></div>; }

function toDraft(profile: ResumeProfile): Draft {
  const parsedSkills = parseArray<string>(profile.professionalSkills, []);
  const personal = parseObject(profile.personalInfo, parsePersonal(profile.personalInfo, emptyPersonal()));
  return { name: profile.name, targetDirection: profile.targetDirection, notes: profile.notes, personal, education: parseArray(profile.educationBackground, parseEducation(profile.educationBackground)), internships: parseArray(profile.internshipExperience, parseEntries(profile.internshipExperience, emptyExperience())), projects: parseArray(profile.projectExperience, parseEntries(profile.projectExperience, emptyProject())), skills: parsedSkills.length ? parsedSkills.join("、") : profile.professionalSkills, academics: parseArray(profile.academicAchievements, parseEntries(profile.academicAchievements, emptyAcademic())), certificates: parseArray(profile.skillCertificates, profile.skillCertificates ? profile.skillCertificates.split(/[，,、\n|]/).map((item) => item.trim()).filter(Boolean) : []) };
}
function serializeDraft(draft: Draft): UpdateResumeProfileInput { return { name: draft.name, targetDirection: draft.targetDirection, notes: draft.notes, personalInfo: JSON.stringify(draft.personal), educationBackground: JSON.stringify(draft.education), internshipExperience: JSON.stringify(draft.internships), projectExperience: JSON.stringify(draft.projects), professionalSkills: draft.skills, academicAchievements: JSON.stringify(draft.academics), skillCertificates: JSON.stringify(draft.certificates) }; }
function parseArray<T>(value: string, fallback: T[]): T[] { try { const parsed = JSON.parse(value); return Array.isArray(parsed) ? parsed as T[] : fallback; } catch { return fallback; } }
function parseObject<T>(value: string, fallback: T): T { try { const parsed = JSON.parse(value); return parsed && typeof parsed === "object" && !Array.isArray(parsed) ? parsed as T : fallback; } catch { return fallback; } }
function lines(value: string) { return value.split(/\r?\n/).map((line) => line.replace(/^[-•·*]\s*/, "").trim()).filter(Boolean); }
function parsePersonal(value: string, fallback: Personal): Personal {
  const text = value.replace(/\r?\n/g, " ");
  const email = text.match(/[\w.+-]+@[\w-]+\.[\w.-]+/)?.[0] ?? "";
  const phone = text.match(/(?:1[3-9]\d{9}|(?:电话|手机|联系方式)\s*[:：]?\s*[\d\-+() ]{7,})/)?.[0] ?? "";
  const links = [...text.matchAll(/https?:\/\/[^\s，,；;]+/g)].map((match) => match[0]).join("、");
  const name = lines(value).find((line) => line.length <= 12 && !/[：:@\d]/.test(line)) ?? fallback.name;
  return { ...fallback, name, contact: [phone, email].filter(Boolean).join(" · "), links };
}
function parseEducation(value: string): Education[] {
  const result = lines(value).map((line) => { const dates = line.match(/(\d{4}[./年-]\d{1,2}).*?(\d{4}[./年-]\d{1,2}|至今)?/); const parts = line.split(/[|｜·•,，;；]/).map((part) => part.trim()).filter(Boolean); return { ...emptyEducation(), startDate: normalizeMonth(dates?.[1]), endDate: normalizeMonth(dates?.[2]), school: parts.find((part) => /大学|学院|学校/.test(part)) ?? parts[0] ?? "", degree: parts.find((part) => /本科|硕士|博士|学士|研究生|专科/.test(part)) ?? "", major: parts.find((part) => /专业|计算机|工程|科学|管理/.test(part)) ?? "" }; }).filter((item) => item.school); return result.length ? result : [emptyEducation()];
}
function parseEntries<T>(value: string, empty: T): T[] {
  const entries = lines(value);
  if (!entries.length) return [];
  // 普通简历中的职责/成果往往一行一条，不能逐行当成多段经历；
  // 只有明显的“时间/公司”标题才进行分组。
  const starts = entries.reduce<number[]>((result, line, index) => {
    if (index > 0 && /(?:19|20)\d{2}\s*[./年-]\s*\d{1,2}|至今/.test(line)) result.push(index);
    return result;
  }, []);
  if (!starts.length) return [{ ...empty, description: entries.join("\n") }] as T[];
  const groups: string[] = [];
  let start = 0;
  for (const next of [...starts, entries.length]) { groups.push(entries.slice(start, next).join("\n")); start = next; }
  return groups.filter(Boolean).map((description) => ({ ...empty, description })) as T[];
}
function normalizeMonth(value?: string) { return value && value !== "至今" ? value.replace(/[年/.]/g, "-").replace(/月$/, "").replace(/-$/, "") : value === "至今" ? "" : ""; }
function replaceAt<T>(items: T[], index: number, value: T): T[] { return items.map((item, itemIndex) => itemIndex === index ? value : item); }
function removeAt<T>(items: T[], index: number): T[] { return items.filter((_, itemIndex) => itemIndex !== index); }
function rate(value: number, total: number) { return total ? `${Math.round(value / total * 100)}%` : "0%"; }
