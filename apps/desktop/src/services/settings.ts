import { invoke } from "@tauri-apps/api/core";

export interface AiProviderSettings {
  provider: string;
  protocol: "responses" | "chat" | "anthropic";
  baseUrl: string;
  model: string;
  fallbackModel?: string;
  allowResume: boolean;
  allowEmail: boolean;
  allowTranscript: boolean;
  promptBeforeSend: boolean;
}

export interface AsrProviderSettings {
  provider: string;
  baseUrl: string;
  model: string;
  language: string;
  speakerDiarization: boolean;
  segmentSeconds: number;
  fileLimitMb: number;
  keepOriginalAudio: boolean;
  deleteTemporaryFiles: boolean;
}

export interface ProviderSettings {
  ai: AiProviderSettings;
  asr: AsrProviderSettings;
  email: EmailSettings;
}

export interface EmailSettings {
  provider: string;
  emailAddress: string;
  imapHost: string;
  imapPort: number;
  username: string;
  useTls: boolean;
  pollingMinutes: number;
  enabled: boolean;
  authMethod: "password" | "oauth2";
  oauthClientId: string;
  oauthTenant: string;
}

export const defaultProviderSettings: ProviderSettings = {
  ai: {
    provider: "OpenAI",
    protocol: "responses",
    baseUrl: "https://api.openai.com/v1",
    model: "gpt-4.1-mini",
    fallbackModel: "",
    allowResume: true,
    allowEmail: true,
    allowTranscript: true,
    promptBeforeSend: false,
  },
  asr: {
    provider: "OpenAI 兼容接口",
    baseUrl: "https://api.openai.com/v1",
    model: "gpt-4o-mini-transcribe",
    language: "zh",
    speakerDiarization: false,
    segmentSeconds: 300,
    fileLimitMb: 500,
    keepOriginalAudio: true,
    deleteTemporaryFiles: true,
  },
  email: { provider: "自定义 IMAP", emailAddress: "", imapHost: "", imapPort: 993, username: "", useTls: true, pollingMinutes: 10, enabled: false, authMethod: "password", oauthClientId: "", oauthTenant: "common" },
};

export function getProviderSettings() {
  return invoke<ProviderSettings>("get_provider_settings");
}

export function saveAiProviderSettings(settings: AiProviderSettings) {
  return invoke<void>("save_ai_provider_settings", { settings });
}

export function saveAsrProviderSettings(settings: AsrProviderSettings) {
  return invoke<void>("save_asr_provider_settings", { settings });
}

export function saveEmailSettings(settings: EmailSettings) {
  return invoke<void>("save_email_settings", { settings });
}

export const getDataLocation = () => invoke<string>("get_data_location");
export const setDataLocation = (directory: string) => invoke<string>("set_data_location", { directory });

export function getCredentialStatus(key: "ai_api_key" | "asr_api_key" | "email_password" | "email_oauth_refresh_token") {
  return invoke<boolean>("credential_status", { key });
}

export function setCredential(key: "ai_api_key" | "asr_api_key" | "email_password" | "email_oauth_refresh_token", secret: string) {
  return invoke<void>("set_credential", { key, secret });
}

export function deleteCredential(key: "ai_api_key" | "asr_api_key" | "email_password" | "email_oauth_refresh_token") {
  return invoke<void>("delete_credential", { key });
}
