import { useEffect, useState, type FormEvent, type ReactNode } from "react";
import { BrainCircuit, Database, KeyRound, Mic2, RotateCcw, ShieldCheck, UserRound, Upload, Trash2, Check } from "lucide-react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { Card, CardHeader } from "../components/ui";
import { hasLocalDatabase } from "../services/applications";
import { testAiProvider } from "../services/ai";
import { deleteResumeProfile, importResumeProfile, listResumeProfiles, setPrimaryResumeProfile, updateResumeProfile, type ResumeProfile, type UpdateResumeProfileInput } from "../services/resumes";
import StructuredResumeSettings from "./StructuredResumeSettings";
import { showToast } from "../services/toast";
import {
  defaultProviderSettings,
  deleteCredential,
  getCredentialStatus,
  getProviderSettings,
  saveAiProviderSettings,
  saveAsrProviderSettings,
  setCredential,
  type AiProviderSettings,
  type AsrProviderSettings,
} from "../services/settings";

type Tab = "profile" | "ai" | "asr" | "data" | "privacy";
type CredentialKey = "ai_api_key" | "asr_api_key";

export default function SettingsPage() {
  const [tab, setTab] = useState<Tab>("profile");
  const [ai, setAi] = useState<AiProviderSettings>(defaultProviderSettings.ai);
  const [asr, setAsr] = useState<AsrProviderSettings>(defaultProviderSettings.asr);
  const [credentialStatus, setCredentialStatus] = useState({ ai_api_key: false, asr_api_key: false });
  const [secret, setSecret] = useState({ ai_api_key: "", asr_api_key: "" });
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState("");
  const [error, setError] = useState("");

  useEffect(() => {
    if (!hasLocalDatabase) {
      setLoading(false);
      return;
    }
    Promise.all([getProviderSettings(), getCredentialStatus("ai_api_key"), getCredentialStatus("asr_api_key")])
      .then(([settings, aiKey, asrKey]) => {
        setAi(settings.ai);
        setAsr(settings.asr);
        setCredentialStatus({ ai_api_key: aiKey, asr_api_key: asrKey });
      })
      .catch((reason) => { setError(String(reason)); showToast(String(reason), "error"); })
      .finally(() => setLoading(false));
  }, []);

  async function saveCredential(key: CredentialKey) {
    if (!secret[key].trim()) return;
    await setCredential(key, secret[key]);
    setSecret((current) => ({ ...current, [key]: "" }));
    setCredentialStatus((current) => ({ ...current, [key]: true }));
  }

  async function submitAi(event: FormEvent) {
    event.preventDefault();
    setSaving(true); setError(""); setMessage("");
    if (hasLocalDatabase) {
      try {
        await saveAiProviderSettings(ai);
        setMessage("AI 服务设置已保存"); showToast("AI 服务设置已保存");
      } catch (reason) {
        setError(`AI 服务设置保存失败：${String(reason)}`); showToast(`AI 服务设置保存失败：${String(reason)}`, "error");
        setSaving(false);
        return;
      }
      try {
        await saveCredential("ai_api_key");
      } catch (reason) {
        setError(`AI 服务设置已保存，但 API Key 保存失败：${String(reason)}`); showToast(`API Key 保存失败：${String(reason)}`, "error");
      }
    } else {
      setMessage("浏览器预览模式：设置仅保留到本次刷新前"); showToast("设置已保存（预览模式）");
    }
    setSaving(false);
  }

  async function submitAsr(event: FormEvent) {
    event.preventDefault();
    setSaving(true); setError(""); setMessage("");
    if (hasLocalDatabase) {
      try {
        await saveAsrProviderSettings(asr);
        setMessage("语音识别设置已保存"); showToast("语音识别设置已保存");
      } catch (reason) {
        setError(`语音识别设置保存失败：${String(reason)}`); showToast(`语音识别设置保存失败：${String(reason)}`, "error");
        setSaving(false);
        return;
      }
      try {
        await saveCredential("asr_api_key");
      } catch (reason) {
        setError(`语音识别设置已保存，但 API Key 保存失败：${String(reason)}`); showToast(`API Key 保存失败：${String(reason)}`, "error");
      }
    } else {
      setMessage("浏览器预览模式：设置仅保留到本次刷新前"); showToast("设置已保存（预览模式）");
    }
    setSaving(false);
  }

  async function removeCredential(key: CredentialKey) {
    setError(""); setMessage("");
    try {
      if (hasLocalDatabase) await deleteCredential(key);
      setCredentialStatus((current) => ({ ...current, [key]: false }));
      setSecret((current) => ({ ...current, [key]: "" }));
      setMessage("凭据已从 Windows 凭据管理器删除"); showToast("凭据已删除");
    } catch (reason) { setError(String(reason)); showToast(String(reason), "error"); }
  }

  async function testConnection() {
    setSaving(true); setError(""); setMessage("");
    try {
      const result = await testAiProvider();
      setMessage(`连接成功 · ${result.model} · ${result.durationMs} ms`); showToast(`连接成功 · ${result.model} · ${result.durationMs} ms`);
    } catch (reason) {
      setError(`连接测试失败：${String(reason)}`); showToast(`连接测试失败：${String(reason)}`, "error");
    } finally { setSaving(false); }
  }

  const navigation: Array<[Tab, typeof BrainCircuit, string]> = [
    ["profile", UserRound, "我的简历"], ["ai", BrainCircuit, "AI 服务"], ["asr", Mic2, "语音识别"],
    ["data", Database, "数据与备份"], ["privacy", ShieldCheck, "隐私与安全"],
  ];

  return <div className="settings-layout">
    <aside>{navigation.map(([value, Icon, label]) => <button type="button" className={tab === value ? "active" : ""} key={value} onClick={() => { setTab(value); setMessage(""); setError(""); }}><Icon size={17}/>{label}</button>)}</aside>
    <div>
      {loading && <Card><div className="settings-notice">正在读取本地设置…</div></Card>}
      {!loading && tab === "profile" && <StructuredResumeSettings onMessage={(value) => { setMessage(value); showToast(value); }} onError={(value) => { setError(value); showToast(value, "error"); }} />}
      {!loading && tab === "ai" && <ProviderForm title="AI 服务" subtitle="配置 OpenAI 兼容接口；API Key 不写入数据库" onSubmit={submitAi} saving={saving}>
        <ProviderPreset value={ai.provider} onChange={(provider) => { const preset = AI_PROVIDER_PRESETS.find((item) => item.name === provider) ?? AI_PROVIDER_PRESETS[0]; setAi({...ai, provider: preset.name, protocol: preset.protocol, baseUrl: preset.baseUrl, model: preset.model, fallbackModel: preset.fallbackModel}); }} />
        <Field label="接口地址" hint="已根据厂商预设填充，也可以手动修改"><input required type="url" value={ai.baseUrl} onChange={(e) => setAi({...ai, baseUrl: e.target.value})}/></Field>
        <div className="settings-form-grid"><Field label="主模型"><input required value={ai.model} onChange={(e) => setAi({...ai, model: e.target.value})}/></Field><Field label="备用模型" hint="可选"><input value={ai.fallbackModel ?? ""} onChange={(e) => setAi({...ai, fallbackModel: e.target.value})}/></Field></div>
        <CredentialField status={credentialStatus.ai_api_key} value={secret.ai_api_key} onChange={(value) => setSecret({...secret, ai_api_key: value})} onDelete={() => removeCredential("ai_api_key")}/>
        <div className="settings-inline-action"><button type="button" className="button button--secondary" disabled={saving || !credentialStatus.ai_api_key} onClick={testConnection}>测试连接</button><small>使用已保存的 API Key 发送最小请求，并记录耗时。</small></div>
        <Card className="privacy-card"><CardHeader title="发送范围" subtitle="默认关闭，只有明确允许的数据才会发送给 AI 服务"/><Toggle checked={ai.allowResume} onChange={(allowResume) => setAi({...ai, allowResume})} label="允许发送简历内容"/><Toggle checked={ai.allowEmail} onChange={(allowEmail) => setAi({...ai, allowEmail})} label="允许发送邮件正文"/><Toggle checked={ai.allowTranscript} onChange={(allowTranscript) => setAi({...ai, allowTranscript})} label="允许发送面试转写"/><Toggle checked={ai.promptBeforeSend} onChange={(promptBeforeSend) => setAi({...ai, promptBeforeSend})} label="每次发送前再次确认"/></Card>
      </ProviderForm>}
      {!loading && tab === "asr" && <ProviderForm title="语音识别" subtitle="配置转写服务与本地音频保留规则" onSubmit={submitAsr} saving={saving}>
        <Field label="接口地址" hint="OpenAI 兼容的 /audio/transcriptions 接口"><input required type="url" value={asr.baseUrl} onChange={(e) => setAsr({...asr, baseUrl: e.target.value})}/></Field>
        <div className="settings-form-grid"><Field label="服务商"><input required value={asr.provider} onChange={(e) => setAsr({...asr, provider: e.target.value})}/></Field><Field label="转写模型"><input required value={asr.model} onChange={(e) => setAsr({...asr, model: e.target.value})}/></Field></div>
        <div className="settings-form-grid"><Field label="默认语言"><select value={asr.language} onChange={(e) => setAsr({...asr, language: e.target.value})}><option value="zh">中文</option><option value="en">English</option><option value="auto">自动检测</option></select></Field><span/></div>
        <div className="settings-form-grid"><Field label="分段时长（秒）"><input type="number" min="30" max="1800" value={asr.segmentSeconds} onChange={(e) => setAsr({...asr, segmentSeconds: Number(e.target.value)})}/></Field><Field label="文件上限（MB）"><input type="number" min="1" max="2048" value={asr.fileLimitMb} onChange={(e) => setAsr({...asr, fileLimitMb: Number(e.target.value)})}/></Field></div>
        <CredentialField status={credentialStatus.asr_api_key} value={secret.asr_api_key} onChange={(value) => setSecret({...secret, asr_api_key: value})} onDelete={() => removeCredential("asr_api_key")}/>
        <Card className="privacy-card"><Toggle checked={asr.speakerDiarization} onChange={(speakerDiarization) => setAsr({...asr, speakerDiarization})} label="启用说话人区分"/><Toggle checked={asr.keepOriginalAudio} onChange={(keepOriginalAudio) => setAsr({...asr, keepOriginalAudio})} label="保留原始音频"/><Toggle checked={asr.deleteTemporaryFiles} onChange={(deleteTemporaryFiles) => setAsr({...asr, deleteTemporaryFiles})} label="转写后删除临时分片"/></Card>
      </ProviderForm>}
      {!loading && tab === "data" && <DataSettings/>}
      {!loading && tab === "privacy" && <Card><CardHeader title="隐私与安全"/><div className="setting-block"><div><strong>凭据隔离</strong><p>API Key 保存在 Windows 凭据管理器，SQLite 仅保存非敏感配置。</p></div><KeyRound size={20}/></div><div className="setting-block"><div><strong>最小化发送</strong><p>简历、邮件和转写内容默认禁止发送，可在 AI 服务中逐项授权。</p></div></div></Card>}
    </div>
  </div>;
}

const AI_PROVIDER_PRESETS = [
  { name: "OpenAI", protocol: "responses" as const, baseUrl: "https://api.openai.com/v1", model: "gpt-4.1-mini", fallbackModel: "gpt-4o-mini" },
  { name: "Anthropic Claude", protocol: "anthropic" as const, baseUrl: "https://api.anthropic.com/v1", model: "claude-3-7-sonnet-latest", fallbackModel: "" },
  { name: "Google Gemini", protocol: "chat" as const, baseUrl: "https://generativelanguage.googleapis.com/v1beta/openai", model: "gemini-2.5-flash", fallbackModel: "gemini-2.0-flash" },
  { name: "DeepSeek", protocol: "chat" as const, baseUrl: "https://api.deepseek.com", model: "deepseek-chat", fallbackModel: "deepseek-reasoner" },
  { name: "通义千问", protocol: "responses" as const, baseUrl: "https://dashscope.aliyuncs.com/compatible-mode/v1", model: "qwen-plus", fallbackModel: "qwen-turbo" },
  { name: "智谱 GLM", protocol: "chat" as const, baseUrl: "https://open.bigmodel.cn/api/paas/v4", model: "glm-4-plus", fallbackModel: "glm-4-flash" },
  { name: "Moonshot Kimi", protocol: "chat" as const, baseUrl: "https://api.moonshot.cn/v1", model: "kimi-k2-turbo-preview", fallbackModel: "moonshot-v1-8k" },
  { name: "硅基流动", protocol: "chat" as const, baseUrl: "https://api.siliconflow.cn/v1", model: "deepseek-ai/DeepSeek-V3", fallbackModel: "" },
  { name: "自定义 OpenAI 兼容", protocol: "chat" as const, baseUrl: "https://", model: "", fallbackModel: "" },
];

function ProviderPreset({ value, onChange }: { value: string; onChange: (value: string) => void }) {
  return <Field label="服务厂商" hint="选择后会自动填充接口地址、协议和推荐模型"><select value={value} onChange={(event) => onChange(event.target.value)}>{AI_PROVIDER_PRESETS.map((item) => <option key={item.name} value={item.name}>{item.name}</option>)}</select></Field>;
}

function ResumeSettings({ onMessage, onError }: { onMessage: (message: string) => void; onError: (message: string) => void }) {
  const [profiles, setProfiles] = useState<ResumeProfile[]>([]);
  const [selectedId, setSelectedId] = useState("");
  const [draft, setDraft] = useState<UpdateResumeProfileInput | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (!hasLocalDatabase) return;
    listResumeProfiles().then((items) => { setProfiles(items); const selected = items.find((item) => item.isPrimary) ?? items[0]; if (selected) { setSelectedId(selected.id); setDraft(toResumeDraft(selected)); } }).catch((reason) => onError(String(reason)));
  }, [onError]);

  const selectProfile = (profile: ResumeProfile) => { setSelectedId(profile.id); setDraft(toResumeDraft(profile)); onMessage(""); onError(""); };
  const importFiles = async () => {
    const paths = await openDialog({ multiple: true, directory: false, filters: [{ name: "简历文件", extensions: ["pdf", "docx", "txt", "md"] }] });
    if (!paths || typeof paths === "string") return;
    setBusy(true); onError("");
    try {
      const imported = await Promise.all(paths.map((path) => importResumeProfile(path)));
      const items = await listResumeProfiles(); setProfiles(items);
      const selected = imported[imported.length - 1]?.profile; if (selected) { setSelectedId(selected.id); setDraft(toResumeDraft(selected)); }
      onMessage(`已导入 ${imported.length} 份简历，可继续手动修订各部分内容`);
    } catch (reason) { onError(`简历解析失败：${String(reason)}`); } finally { setBusy(false); }
  };
  const save = async () => {
    if (!draft || !selectedId) return;
    setBusy(true); onError("");
    try { const updated = await updateResumeProfile(selectedId, draft); setProfiles((items) => items.map((item) => item.id === updated.id ? updated : item)); setDraft(toResumeDraft(updated)); onMessage("简历内容已保存"); } catch (reason) { onError(String(reason)); } finally { setBusy(false); }
  };
  const makePrimary = async () => { if (!selectedId) return; try { await setPrimaryResumeProfile(selectedId); setProfiles((items) => items.map((item) => ({ ...item, isPrimary: item.id === selectedId }))); onMessage("已设为默认简历"); } catch (reason) { onError(String(reason)); } };
  const remove = async () => { if (!selectedId || !window.confirm("确定删除这份简历吗？")) return; try { await deleteResumeProfile(selectedId); const items = await listResumeProfiles(); setProfiles(items); const selected = items[0]; setSelectedId(selected?.id ?? ""); setDraft(selected ? toResumeDraft(selected) : null); onMessage("简历已删除"); } catch (reason) { onError(String(reason)); } };
  const update = (key: keyof UpdateResumeProfileInput, value: string) => setDraft((current) => current ? ({ ...current, [key]: value }) : current);

  return <div className="resume-page">
    <div className="resume-page-head"><div><span className="eyebrow">个人资料库</span><h2>我的简历</h2><p>管理不同求职方向的简历版本。上传后先由系统解析，再由你确认和修改。</p></div><button type="button" className="button button--primary" disabled={!hasLocalDatabase || busy} onClick={importFiles}><Upload size={15}/>{busy ? "正在解析…" : "添加一份简历"}</button></div>
    <div className="resume-settings"><Card className="resume-list-card"><CardHeader title="简历版本" subtitle={`${profiles.length} 份 · 默认简历用于 AI 准备和投递关联`}/><div className="resume-list">{profiles.map((profile) => <button type="button" key={profile.id} className={profile.id === selectedId ? "active" : ""} onClick={() => selectProfile(profile)}><span className="resume-list-icon">{profile.isPrimary ? <Check size={15}/> : <UserRound size={15}/>}</span><span><strong>{profile.name}</strong><small>{profile.fileFormat?.toUpperCase() ?? "手动创建"} · {profile.isPrimary ? "默认版本" : "点击编辑"}</small></span><span className="resume-list-arrow">›</span></button>)}{!profiles.length && <div className="resume-empty"><UserRound size={28}/><p>还没有简历<br/>点击右上角添加第一份</p></div>}</div><div className="resume-list-tip">你可以为不同岗位方向保留多份版本。</div></Card>
      <Card className="resume-editor-card"><CardHeader title={draft ? "编辑当前简历" : "开始建立简历"} subtitle={draft ? "所有内容都保存在本机，解析结果仅作为可修改草稿" : "上传简历后，这里会出现分栏编辑器"}/>{draft ? <><div className="resume-editor-toolbar"><div className="resume-name-field"><span>当前版本名称</span><input value={draft.name} onChange={(event) => update("name", event.target.value)} placeholder="例如：后端开发 · 2026 春招"/></div><div className="resume-toolbar-actions"><button type="button" className="button button--secondary" onClick={makePrimary}>{profiles.find((item) => item.id === selectedId)?.isPrimary ? <Check size={14}/> : null}{profiles.find((item) => item.id === selectedId)?.isPrimary ? "默认简历" : "设为默认"}</button><button type="button" className="button button--secondary danger-text" onClick={remove}><Trash2 size={14}/>删除</button></div></div><div className="resume-editor-note"><Check size={14}/> 已完成本地解析；请检查联系方式、时间和量化成果后再保存。</div><div className="resume-section-grid"><ResumeTextField label="个人信息" hint="姓名、联系方式、所在地、个人链接" value={draft.personalInfo} onChange={(value) => update("personalInfo", value)} placeholder="例如：张三 · 138**** · Hangzhou · github.com/..."/><ResumeTextField label="教育背景" hint="学校、专业、学历、时间、GPA" value={draft.educationBackground} onChange={(value) => update("educationBackground", value)} placeholder="按时间倒序填写，每段一行"/><ResumeTextField label="实习经历" hint="公司、职位、时间、职责和成果" value={draft.internshipExperience} onChange={(value) => update("internshipExperience", value)} placeholder="用“做了什么 + 结果如何”描述"/><ResumeTextField label="项目经历" hint="背景、职责、技术方案、量化结果" value={draft.projectExperience} onChange={(value) => update("projectExperience", value)} placeholder="突出个人贡献，不只写团队工作"/><ResumeTextField label="专业技能" hint="语言、框架、数据库、中间件、工具" value={draft.professionalSkills} onChange={(value) => update("professionalSkills", value)} placeholder="例如：Rust、React、PostgreSQL…"/><ResumeTextField label="学术成果" hint="论文、专利、科研、竞赛" value={draft.academicAchievements} onChange={(value) => update("academicAchievements", value)} placeholder="没有可留空"/><ResumeTextField label="技能证书" hint="语言、软考、云厂商认证等" value={draft.skillCertificates} onChange={(value) => update("skillCertificates", value)} placeholder="没有可留空"/></div><div className="resume-editor-actions"><span>修改后记得保存，切换版本不会自动保存。</span><button type="button" className="button button--primary" disabled={busy} onClick={save}>保存当前简历</button></div></> : <div className="resume-empty resume-editor-empty"><Upload size={34}/><h3>还没有选中的简历</h3><p>从右上角添加文件，系统会自动解析成可编辑的栏目。</p><button type="button" className="button button--secondary" disabled={!hasLocalDatabase || busy} onClick={importFiles}>添加第一份简历</button></div>}</Card>
    </div>
  </div>;
}

function toResumeDraft(profile: ResumeProfile): UpdateResumeProfileInput { return { name: profile.name, targetDirection: profile.targetDirection, notes: profile.notes, personalInfo: profile.personalInfo, educationBackground: profile.educationBackground, internshipExperience: profile.internshipExperience, projectExperience: profile.projectExperience, professionalSkills: profile.professionalSkills, academicAchievements: profile.academicAchievements, skillCertificates: profile.skillCertificates }; }
function ResumeTextField({ label, hint, value, onChange, placeholder }: { label: string; hint: string; value: string; onChange: (value: string) => void; placeholder: string }) { return <label className="resume-text-field"><span>{label}<small>{hint}</small></span><textarea rows={7} value={value} onChange={(event) => onChange(event.target.value)} placeholder={placeholder}/></label>; }

function ProviderForm({ title, subtitle, onSubmit, saving, children }: { title: string; subtitle: string; onSubmit: (event: FormEvent) => void; saving: boolean; children: ReactNode }) {
  return <form onSubmit={onSubmit}><Card className="provider-settings-card"><CardHeader title={title} subtitle={subtitle}/><div className="provider-form">{children}<div className="settings-actions"><button className="button button--primary" disabled={saving}>{saving ? "保存中…" : "保存设置"}</button></div></div></Card></form>;
}

function Field({ label, hint, children }: { label: string; hint?: string; children: ReactNode }) { return <label className="settings-field"><span>{label}</span>{children}{hint && <small>{hint}</small>}</label>; }

function CredentialField({ status, value, onChange, onDelete }: { status: boolean; value: string; onChange: (value: string) => void; onDelete: () => void }) { return <Field label="API Key" hint="出于安全考虑，已保存的密钥不会回显；留空表示保持原密钥不变"><div className="credential-field"><input type="password" autoComplete="new-password" value={value} onChange={(e) => onChange(e.target.value)} placeholder={status ? "••••••••（已配置）" : "输入 API Key"}/><span className={status ? "credential-state is-set" : "credential-state"}>{status ? "已配置" : "未配置"}</span>{status && <button type="button" className="text-button danger-text" onClick={onDelete}>删除</button>}</div></Field>; }

function Toggle({ checked, onChange, label }: { checked: boolean; onChange: (checked: boolean) => void; label: string }) { return <label className="settings-toggle"><input type="checkbox" checked={checked} onChange={(e) => onChange(e.target.checked)}/><span>{label}</span></label>; }

function DataSettings() { return <><Card><CardHeader title="数据与备份"/><div className="setting-block"><div><strong>数据保存位置</strong><p>数据库、简历和面试材料保存在应用的本地数据目录</p></div><span className="credential-state is-set">本地存储</span></div><div className="setting-block"><div><strong>自动备份</strong><p>备份功能将在下一阶段接入；系统凭据不会写入备份</p></div><button className="button button--secondary" disabled>即将支持</button></div><div className="setting-block"><div><strong>恢复备份</strong><p>恢复前将进行格式与完整性检查</p></div><button className="button button--secondary" disabled><RotateCcw size={16}/>恢复备份</button></div></Card></>; }
