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
}

export const defaultProviderSettings: ProviderSettings = {
  ai: {
    provider: "OpenAI",
    protocol: "responses",
    baseUrl: "https://api.openai.com/v1",
    model: "gpt-4.1-mini",
    fallbackModel: "",
    allowResume: false,
    allowEmail: false,
    allowTranscript: false,
    promptBeforeSend: true,
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

export function getCredentialStatus(key: "ai_api_key" | "asr_api_key") {
  return invoke<boolean>("credential_status", { key });
}

export function setCredential(key: "ai_api_key" | "asr_api_key", secret: string) {
  return invoke<void>("set_credential", { key, secret });
}

export function deleteCredential(key: "ai_api_key" | "asr_api_key") {
  return invoke<void>("delete_credential", { key });
}
