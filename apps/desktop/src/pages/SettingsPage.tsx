import { useEffect, useState, type FormEvent, type ReactNode } from "react";
import { BrainCircuit, Database, ExternalLink, FolderOpen, KeyRound, Mail, Mic2, RotateCcw, ShieldCheck, UserRound } from "lucide-react";
import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
import { Card, CardHeader } from "../components/ui";
import { hasLocalDatabase } from "../services/applications";
import { testAiProvider } from "../services/ai";
import { authorizeEmailOAuth } from "../services/emails";
import StructuredResumeSettings from "./StructuredResumeSettings";
import { showToast } from "../services/toast";
import { openExternalUrl } from "../services/external";
import {
  defaultProviderSettings,
  backupDatabase,
  deleteCredential,
  getCredentialStatus,
  getDataLocation,
  getProviderSettings,
  saveAiProviderSettings,
  saveAsrProviderSettings,
  saveEmailSettings,
  restoreDatabase,
  setDataLocation,
  setCredential,
  type AiProviderSettings,
  type AsrProviderSettings,
  type EmailSettings,
} from "../services/settings";

type Tab = "profile" | "ai" | "asr" | "email" | "data" | "privacy";
type CredentialKey = "ai_api_key" | "asr_api_key" | "email_password" | "email_oauth_refresh_token";

export default function SettingsPage() {
  const [tab, setTab] = useState<Tab>("profile");
  const [ai, setAi] = useState<AiProviderSettings>(defaultProviderSettings.ai);
  const [asr, setAsr] = useState<AsrProviderSettings>(defaultProviderSettings.asr);
  const [email, setEmail] = useState<EmailSettings>(defaultProviderSettings.email);
  const [credentialStatus, setCredentialStatus] = useState({ ai_api_key: false, asr_api_key: false, email_password: false, email_oauth_refresh_token: false });
  const [secret, setSecret] = useState({ ai_api_key: "", asr_api_key: "", email_password: "", email_oauth_refresh_token: "" });
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState("");
  const [error, setError] = useState("");

  useEffect(() => {
    if (!hasLocalDatabase) {
      setLoading(false);
      return;
    }
    let disposed = false;
    Promise.all([getProviderSettings(), getCredentialStatus("ai_api_key"), getCredentialStatus("asr_api_key"), getCredentialStatus("email_password"), getCredentialStatus("email_oauth_refresh_token")])
      .then(([settings, aiKey, asrKey, emailPassword, emailOAuth]) => {
        if (disposed) return;
        setAi(settings.ai);
        setAsr(settings.asr);
        setEmail(settings.email);
        setCredentialStatus({ ai_api_key: aiKey, asr_api_key: asrKey, email_password: emailPassword, email_oauth_refresh_token: emailOAuth });
      })
      .catch((reason) => { if (!disposed) { setError(String(reason)); showToast(String(reason), "error"); } })
      .finally(() => { if (!disposed) setLoading(false); });
    return () => { disposed = true; };
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
      setMessage("预览模式下更改会在刷新后重置"); showToast("设置已保存（预览模式）");
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
      setMessage("预览模式下更改会在刷新后重置"); showToast("设置已保存（预览模式）");
    }
    setSaving(false);
  }

  async function submitEmail(event: FormEvent) {
    event.preventDefault(); setSaving(true); setError(""); setMessage("");
    try {
      if (hasLocalDatabase) {
        if (email.enabled && email.authMethod === "password" && !credentialStatus.email_password && !secret.email_password.trim()) throw new Error("启用邮件检查前，请先填写并保存邮箱授权码或密码");
        if (email.enabled && email.authMethod === "oauth2" && !credentialStatus.email_oauth_refresh_token) throw new Error("启用邮件检查前，请先完成 OAuth2 授权");
        // Credential first: never persist an enabled connection that cannot authenticate.
        if (email.authMethod === "password") await saveCredential("email_password");
        await saveEmailSettings(email);
        window.dispatchEvent(new Event("email-settings-changed"));
      }
      setMessage("邮箱设置已保存，后续邮件检查将使用此连接"); showToast("邮箱设置已保存");
    } catch (reason) { setError(`邮箱设置保存失败：${String(reason)}`); showToast(`邮箱设置保存失败：${String(reason)}`, "error"); }
    finally { setSaving(false); }
  }

  async function authorizeEmail() {
    setSaving(true); setError(""); setMessage("");
    try {
      await saveEmailSettings(email);
      await authorizeEmailOAuth(email);
      setCredentialStatus((current) => ({ ...current, email_oauth_refresh_token: true }));
      window.dispatchEvent(new Event("email-settings-changed"));
      setMessage("邮箱授权成功，可以开始检查邮件了"); showToast("邮箱授权成功");
    } catch (reason) { setError(`OAuth2 授权失败：${String(reason)}`); showToast(`OAuth2 授权失败：${String(reason)}`, "error"); }
    finally { setSaving(false); }
  }

  async function removeCredential(key: CredentialKey) {
    setError(""); setMessage("");
    try {
      if (hasLocalDatabase) {
        if ((key === "email_password" || key === "email_oauth_refresh_token") && email.enabled) {
          const disabledEmail = { ...email, enabled: false };
          await saveEmailSettings(disabledEmail);
          setEmail(disabledEmail);
          window.dispatchEvent(new Event("email-settings-changed"));
        }
        await deleteCredential(key);
      }
      setCredentialStatus((current) => ({ ...current, [key]: false }));
      setSecret((current) => ({ ...current, [key]: "" }));
      setMessage("凭据已从安全存储中删除"); showToast("凭据已删除");
    } catch (reason) { setError(String(reason)); showToast(String(reason), "error"); }
  }

  async function testConnection() {
    setSaving(true); setError(""); setMessage("");
    try {
      await saveAiProviderSettings(ai);
      await saveCredential("ai_api_key");
      await testAiProvider();
      setMessage("AI 服务连接测试通过"); showToast("AI 服务连接测试通过");
    } catch (reason) {
      setError(`连接测试失败：${String(reason)}`); showToast(`连接测试失败：${String(reason)}`, "error");
    } finally { setSaving(false); }
  }

  const navigation: Array<[Tab, typeof BrainCircuit, string]> = [
    ["profile", UserRound, "我的简历"], ["ai", BrainCircuit, "AI 服务"], ["asr", Mic2, "语音识别"], ["email", Mail, "邮箱设置"],
    ["data", Database, "数据与备份"], ["privacy", ShieldCheck, "隐私与安全"],
  ];
  const selectedEmailPreset = EMAIL_PROVIDER_PRESETS.find((item) => item.name === email.provider);

  return <div className="settings-layout">
    <aside>{navigation.map(([value, Icon, label]) => <button type="button" className={tab === value ? "active" : ""} key={value} onClick={() => { setTab(value); setMessage(""); setError(""); }}><Icon size={17} />{label}</button>)}</aside>
    <div>
      {loading && <Card><div className="settings-notice">正在读取本地设置…</div></Card>}
      {!loading && message && <div className="settings-notice" role="status">{message}</div>}
      {!loading && error && <div className="detail-error" role="alert">{error}</div>}
      {!loading && tab === "profile" && <StructuredResumeSettings onMessage={(value) => { setMessage(value); if (value) window.dispatchEvent(new Event("resume-profile-changed")); showToast(value); }} onError={(value) => { setError(value); showToast(value, "error"); }} />}
      {!loading && tab === "ai" && <ProviderForm title="AI 服务" subtitle="连接 AI 服务，获取面试准备和复盘建议" onSubmit={submitAi} saving={saving}>
        <ProviderPreset value={ai.provider} onChange={(provider) => { const preset = AI_PROVIDER_PRESETS.find((item) => item.name === provider); if (!preset) { setError("未知的 AI 服务厂商"); return; } setAi({ ...ai, provider: preset.name, protocol: preset.protocol, baseUrl: preset.baseUrl, model: preset.model, fallbackModel: preset.fallbackModel }); }} />
        <Field label="接口地址" hint="已根据厂商预设填充，也可以手动修改"><input required type="url" value={ai.baseUrl} onChange={(e) => setAi({ ...ai, baseUrl: e.target.value })} /></Field>
        <div className="settings-form-grid"><Field label="主模型"><input required value={ai.model} onChange={(e) => setAi({ ...ai, model: e.target.value })} /></Field><Field label="备用模型" hint="可选"><input value={ai.fallbackModel ?? ""} onChange={(e) => setAi({ ...ai, fallbackModel: e.target.value })} /></Field></div>
        <div className="settings-form-grid"><Field label="最大输出 Token"><input type="number" min="256" max="32768" value={ai.maxOutputTokens} onChange={(e) => setAi({ ...ai, maxOutputTokens: Number(e.target.value) })} /></Field><Field label="请求超时（秒）"><input type="number" min="5" max="300" value={ai.timeoutSeconds} onChange={(e) => setAi({ ...ai, timeoutSeconds: Number(e.target.value) })} /></Field></div>
        <CredentialField status={credentialStatus.ai_api_key} value={secret.ai_api_key} onChange={(value) => setSecret({ ...secret, ai_api_key: value })} onDelete={() => removeCredential("ai_api_key")} />
        <div className="settings-inline-action"><button type="button" className="button button--secondary" disabled={!hasLocalDatabase || saving || (!credentialStatus.ai_api_key && !secret.ai_api_key.trim())} onClick={testConnection}>保存并测试连接</button><small>保存后会发送一个测试请求确认服务可用</small></div>
        <Card className="privacy-card"><CardHeader title="AI 数据范围" subtitle="控制哪些数据会发送给 AI 服务" /><Toggle checked={ai.allowResume} onChange={(allowResume) => setAi({ ...ai, allowResume })} label="允许向 AI 服务发送简历与岗位信息" /><Toggle checked={ai.allowTranscript} onChange={(allowTranscript) => setAi({ ...ai, allowTranscript })} label="允许向 AI 服务发送面试转写内容" /><Toggle checked={ai.promptBeforeSend} onChange={(promptBeforeSend) => setAi({ ...ai, promptBeforeSend })} label="每次发送简历等敏感内容前要求确认" /></Card>
      </ProviderForm>}
      {!loading && tab === "email" && <ProviderForm title="邮箱设置" subtitle="连接邮箱，自动识别招聘邮件" onSubmit={submitEmail} saving={saving}>
        <Field label="邮箱服务" hint="选择常用服务商后会自动填写 IMAP 服务器、端口和推荐认证方式"><select value={email.provider} onChange={(event) => { const provider = event.target.value; const preset = EMAIL_PROVIDER_PRESETS.find((item) => item.name === provider); setEmail(preset ? { ...email, provider, imapHost: preset.imapHost, imapPort: preset.imapPort, useTls: preset.useTls, authMethod: preset.authMethod } : { ...email, provider }); }}>{EMAIL_PROVIDER_PRESETS.map((item) => <option key={item.name}>{item.name}</option>)}</select></Field>
        <div className="settings-form-grid"><Field label="邮箱地址"><input type="email" value={email.emailAddress} onChange={(event) => setEmail({ ...email, emailAddress: event.target.value })} /></Field><Field label="登录用户名" hint="通常与邮箱地址相同"><input value={email.username} onChange={(event) => setEmail({ ...email, username: event.target.value })} /></Field></div>
        <div className="settings-form-grid"><Field label="IMAP 服务器"><input placeholder="imap.example.com" value={email.imapHost} onChange={(event) => setEmail({ ...email, imapHost: event.target.value })} /></Field><Field label="端口"><input type="number" min="1" max="65535" value={email.imapPort} onChange={(event) => setEmail({ ...email, imapPort: Number(event.target.value) })} /></Field></div>
        <Field label="认证方式"><select value={email.authMethod} onChange={(event) => setEmail({ ...email, authMethod: event.target.value as EmailSettings["authMethod"] })}><option value="password">授权码 / 密码</option><option value="oauth2">OAuth2（Gmail / Microsoft）</option></select></Field>
        {email.authMethod === "password" ? <CredentialField label="邮箱授权码 / 密码" status={credentialStatus.email_password} value={secret.email_password} onChange={(value) => setSecret({ ...secret, email_password: value })} onDelete={() => removeCredential("email_password")} /> : <>
          <Field label="OAuth2 Client ID" hint="请使用服务商控制台创建“桌面应用/公共客户端”；Client ID 不是密钥，可保存在本地设置中"><input value={email.oauthClientId} onChange={(event) => setEmail({ ...email, oauthClientId: event.target.value })} placeholder="输入桌面应用 Client ID" /></Field>
          {email.provider === "Outlook" && <Field label="Microsoft Tenant" hint="个人与多租户应用使用 common；组织可填写租户 ID"><input value={email.oauthTenant} onChange={(event) => setEmail({ ...email, oauthTenant: event.target.value })} /></Field>}
          <div className="settings-inline-action"><button type="button" className="button button--secondary" disabled={!hasLocalDatabase || saving || !email.oauthClientId.trim()} onClick={authorizeEmail}>{saving ? "等待浏览器授权…" : credentialStatus.email_oauth_refresh_token ? "重新授权 OAuth2" : "连接并授权 OAuth2"}</button>{credentialStatus.email_oauth_refresh_token && <button type="button" className="text-button danger-text" disabled={saving} onClick={() => removeCredential("email_oauth_refresh_token")}>移除本机授权</button>}<small>{credentialStatus.email_oauth_refresh_token ? "授权信息已保存，每次检查邮件时会自动续期。" : "将打开浏览器完成授权，授权后会自动返回应用。"}</small></div>
        </>}
        {selectedEmailPreset?.credentialUrl && <div className="settings-inline-action"><a className="button button--secondary" href={selectedEmailPreset.credentialUrl} onClick={(event) => { event.preventDefault(); void openExternalUrl(selectedEmailPreset.credentialUrl).catch((reason) => showToast(String(reason), "error")); }}><ExternalLink size={15} />{selectedEmailPreset.credentialAction}</a><small>{selectedEmailPreset.credentialHint}</small></div>}
        <div className="settings-form-grid"><Field label="检查间隔（分钟）"><input type="number" min="1" max="1440" value={email.pollingMinutes} onChange={(event) => setEmail({ ...email, pollingMinutes: Number(event.target.value) })} /></Field><span /></div>
        <Card className="privacy-card"><CardHeader title="连接行为" subtitle="放心设置，不会影响你的原邮件" /><Toggle checked={email.useTls} onChange={(useTls) => setEmail({ ...email, useTls, imapPort: useTls && email.imapPort === 143 ? 993 : email.imapPort })} label="使用 TLS 加密连接（远程服务器必须启用，防止凭据和邮件明文传输）" /><Toggle checked={email.enabled} onChange={(enabled) => setEmail({ ...email, enabled })} label="启用定时邮件检查（按上方间隔自动读取新邮件）" /></Card>
      </ProviderForm>}
      {!loading && tab === "asr" && <ProviderForm title="语音识别" subtitle="配置语音转文字，方便导入面试录音" onSubmit={submitAsr} saving={saving}>
        <Field label="接口地址" hint="OpenAI 兼容的 /audio/transcriptions 接口"><input required type="url" value={asr.baseUrl} onChange={(e) => setAsr({ ...asr, baseUrl: e.target.value })} /></Field>
        <div className="settings-form-grid"><Field label="服务商"><input required value={asr.provider} onChange={(e) => setAsr({ ...asr, provider: e.target.value })} /></Field><Field label="转写模型"><input required value={asr.model} onChange={(e) => setAsr({ ...asr, model: e.target.value })} /></Field></div>
        <div className="settings-form-grid"><Field label="默认语言"><select value={asr.language} onChange={(e) => setAsr({ ...asr, language: e.target.value })}><option value="zh">中文</option><option value="en">English</option><option value="auto">自动检测</option></select></Field><span /></div>
        <div className="settings-form-grid"><Field label="文件上限（MB）" hint="音频会从原文件流式上传，不会一次性载入内存或创建本地分片"><input type="number" min="1" max="2048" value={asr.fileLimitMb} onChange={(e) => setAsr({ ...asr, fileLimitMb: Number(e.target.value) })} /></Field><span /></div>
        <CredentialField status={credentialStatus.asr_api_key} value={secret.asr_api_key} onChange={(value) => setSecret({ ...secret, asr_api_key: value })} onDelete={() => removeCredential("asr_api_key")} />
        <Card className="privacy-card"><Toggle checked={asr.speakerDiarization} onChange={(speakerDiarization) => setAsr({ ...asr, speakerDiarization })} label="启用说话人区分（区分面试中不同人的发言）" /></Card>
      </ProviderForm>}
      {!loading && tab === "data" && <DataSettings />}
      {!loading && tab === "privacy" && <Card><CardHeader title="隐私与安全" subtitle="你的数据安全由你掌控" /><div className="setting-block"><div><strong>敏感凭据单独保管</strong><p>AI、语音和邮箱密码保存在 Windows 凭据管理器，不写入业务数据库或备份。</p></div><KeyRound size={20} /></div><div className="setting-block"><div><strong>本地数据由你掌控</strong><p>投递、简历和面试记录默认存放在你选择的本地目录；只有使用 AI、语音识别或邮箱检查时，相关必要内容才会发往你配置的服务。</p></div></div><div className="setting-block"><div><strong>删除会同步解除关联</strong><p>删除简历时会解除投递关系；删除或移动数据前，界面会明确提示影响范围。</p></div></div></Card>}
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

const EMAIL_PROVIDER_PRESETS = [
  { name: "自定义 IMAP", imapHost: "", imapPort: 993, useTls: true, authMethod: "password" as const, credentialUrl: "", credentialAction: "", credentialHint: "" },
  { name: "QQ 邮箱", imapHost: "imap.qq.com", imapPort: 993, useTls: true, authMethod: "password" as const, credentialUrl: "https://mail.qq.com/", credentialAction: "前往 QQ 邮箱获取授权码", credentialHint: "登录后进入“设置 → 账号与安全 → 安全设置 → IMAP/SMTP 服务”，开启服务并生成授权码。" },
  { name: "网易邮箱", imapHost: "imap.163.com", imapPort: 993, useTls: true, authMethod: "password" as const, credentialUrl: "https://mail.163.com/", credentialAction: "前往网易邮箱获取授权码", credentialHint: "登录后进入“设置 → POP3/SMTP/IMAP”，开启 IMAP/SMTP 服务并设置客户端授权密码。" },
  { name: "网易 188 邮箱", imapHost: "imap.188.com", imapPort: 993, useTls: true, authMethod: "password" as const, credentialUrl: "https://mail.188.com/", credentialAction: "前往 188 邮箱获取授权码", credentialHint: "请开启 IMAP 并使用客户端授权码；连接器会按网易要求发送 RFC 2971 客户端 ID。" },
  { name: "Outlook", imapHost: "outlook.office365.com", imapPort: 993, useTls: true, authMethod: "oauth2" as const, credentialUrl: "https://entra.microsoft.com/#view/Microsoft_AAD_RegisteredApps/ApplicationsListBlade", credentialAction: "前往 Microsoft Entra 创建应用", credentialHint: "创建应用后添加“移动和桌面应用程序”平台、http://localhost 重定向 URI，并允许公共客户端流。" },
  { name: "Gmail", imapHost: "imap.gmail.com", imapPort: 993, useTls: true, authMethod: "oauth2" as const, credentialUrl: "https://console.cloud.google.com/apis/credentials", credentialAction: "前往 Google Cloud 创建 Client ID", credentialHint: "先配置 OAuth 同意屏幕，再创建“桌面应用”类型的 OAuth Client ID，并为测试应用添加当前邮箱。" },
];

function ProviderPreset({ value, onChange }: { value: string; onChange: (value: string) => void }) {
  return <Field label="服务厂商" hint="选择后会自动填充接口地址、协议和推荐模型"><select value={value} onChange={(event) => onChange(event.target.value)}>{AI_PROVIDER_PRESETS.map((item) => <option key={item.name} value={item.name}>{item.name}</option>)}</select></Field>;
}

function ProviderForm({ title, subtitle, onSubmit, saving, children }: { title: string; subtitle: string; onSubmit: (event: FormEvent) => void; saving: boolean; children: ReactNode }) {
  return <form onSubmit={onSubmit}><Card className="provider-settings-card"><CardHeader title={title} subtitle={subtitle} /><div className="provider-form">{children}<div className="settings-actions"><button className="button button--primary" disabled={saving}>{saving ? "保存中…" : "保存设置"}</button></div></div></Card></form>;
}

function Field({ label, hint, children }: { label: string; hint?: string; children: ReactNode }) { return <label className="settings-field"><span>{label}</span>{children}{hint && <small>{hint}</small>}</label>; }

function CredentialField({ label = "API Key", status, value, onChange, onDelete }: { label?: string; status: boolean; value: string; onChange: (value: string) => void; onDelete: () => void }) { return <Field label={label} hint="出于安全考虑，已保存的密钥不会回显；留空表示保持原凭据不变"><div className="credential-field"><input type="password" autoComplete="new-password" value={value} onChange={(e) => onChange(e.target.value)} placeholder={status ? "••••••••（已配置）" : `输入${label}`} /><span className={status ? "credential-state is-set" : "credential-state"}>{status ? "已配置" : "未配置"}</span>{status && <button type="button" className="text-button danger-text" onClick={onDelete}>删除</button>}</div></Field>; }

function Toggle({ checked, onChange, label }: { checked: boolean; onChange: (checked: boolean) => void; label: string }) { return <label className="settings-toggle"><input type="checkbox" checked={checked} onChange={(e) => onChange(e.target.checked)} /><span>{label}</span></label>; }

function DataSettings() {
  const [location, setLocation] = useState("");
  const [operation, setOperation] = useState<"move" | "backup" | "restore">();
  useEffect(() => { let disposed = false; if (hasLocalDatabase) getDataLocation().then((value) => { if (!disposed) setLocation(value); }).catch(() => undefined); return () => { disposed = true; }; }, []);
  const choose = async () => { const directory = await openDialog({ directory: true, multiple: false, title: "选择数据保存目录" }); if (!directory || typeof directory !== "string") return; setOperation("move"); try { setLocation(await setDataLocation(directory)); showToast("已切换数据位置，原数据库保留为安全副本"); } catch (reason) { showToast(String(reason), "error"); } finally { setOperation(undefined); } };
  const backup = async () => {
    const date = new Date().toISOString().slice(0, 10);
    const path = await saveDialog({ title: "导出数据备份", defaultPath: `applied-yet-backup-${date}.sqlite3`, filters: [{ name: "投了吗数据库", extensions: ["sqlite3"] }] });
    if (!path) return;
    setOperation("backup");
    try { await backupDatabase(path); showToast("数据备份已通过完整性检查并保存"); } catch (reason) { showToast(String(reason), "error"); } finally { setOperation(undefined); }
  };
  const restore = async () => {
    const path = await openDialog({ title: "选择数据备份", multiple: false, filters: [{ name: "投了吗数据库", extensions: ["sqlite3", "db"] }] });
    if (!path || typeof path !== "string") return;
    if (!window.confirm("恢复后会立即切换到所选备份；当前数据库将原样保留，系统凭据不会改变。确定继续吗？")) return;
    setOperation("restore");
    try {
      await restoreDatabase(path);
      window.alert("备份恢复成功，应用将重新载入。");
      window.location.reload();
    } catch (reason) { showToast(String(reason), "error"); setOperation(undefined); }
  };
  const busy = operation !== undefined;
  return <><Card><CardHeader title="数据与备份" /><div className="setting-block"><div><strong>数据保存位置</strong><p>{hasLocalDatabase ? location || "正在读取当前数据位置…" : "预览模式下展示的是示例数据，不会影响你的真实信息"}</p></div><button type="button" className="button button--secondary" disabled={!hasLocalDatabase || busy} onClick={choose}><FolderOpen size={16} />{operation === "move" ? "正在移动…" : "选择位置"}</button></div><div className="setting-block"><div><strong>导出数据备份</strong><p>生成一份完整的求职数据副本，方便你迁移或存档</p></div><button type="button" className="button button--secondary" disabled={!hasLocalDatabase || busy} onClick={backup}><Database size={16} />{operation === "backup" ? "正在备份…" : "导出备份"}</button></div><div className="setting-block"><div><strong>恢复数据备份</strong><p>从之前备份的文件恢复你的所有求职数据</p></div><button type="button" className="button button--secondary" disabled={!hasLocalDatabase || busy} onClick={restore}><RotateCcw size={16} />{operation === "restore" ? "正在恢复…" : "恢复备份"}</button></div></Card></>;
}
