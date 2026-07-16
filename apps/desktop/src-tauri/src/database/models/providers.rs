use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AiProviderSettings {
    pub provider: String,
    pub protocol: String,
    pub base_url: String,
    pub model: String,
    pub fallback_model: Option<String>,
    pub max_output_tokens: i64,
    pub timeout_seconds: i64,
    pub allow_resume: bool,
    pub allow_transcript: bool,
    pub prompt_before_send: bool,
}

impl Default for AiProviderSettings {
    fn default() -> Self {
        Self {
            provider: "OpenAI".into(),
            protocol: "responses".into(),
            base_url: "https://api.openai.com/v1".into(),
            model: "gpt-4.1-mini".into(),
            fallback_model: None,
            max_output_tokens: 4096,
            timeout_seconds: 60,
            allow_resume: false,
            allow_transcript: false,
            prompt_before_send: true,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AsrProviderSettings {
    pub provider: String,
    pub base_url: String,
    pub model: String,
    pub language: String,
    pub speaker_diarization: bool,
    pub segment_seconds: i64,
    pub file_limit_mb: i64,
    pub keep_original_audio: bool,
    pub delete_temporary_files: bool,
}

impl Default for AsrProviderSettings {
    fn default() -> Self {
        Self {
            provider: "OpenAI 兼容接口".into(),
            base_url: "https://api.openai.com/v1".into(),
            model: "gpt-4o-mini-transcribe".into(),
            language: "zh".into(),
            speaker_diarization: false,
            segment_seconds: 300,
            file_limit_mb: 500,
            keep_original_audio: true,
            delete_temporary_files: true,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderSettings {
    pub ai: AiProviderSettings,
    pub asr: AsrProviderSettings,
    pub email: EmailSettings,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct EmailSettings {
    pub provider: String,
    pub email_address: String,
    pub imap_host: String,
    pub imap_port: i64,
    pub username: String,
    pub use_tls: bool,
    pub polling_minutes: i64,
    pub enabled: bool,
    pub auth_method: String,
    pub oauth_client_id: String,
    pub oauth_tenant: String,
}

impl Default for EmailSettings {
    fn default() -> Self {
        Self {
            provider: "自定义 IMAP".into(),
            email_address: String::new(),
            imap_host: String::new(),
            imap_port: 993,
            username: String::new(),
            use_tls: true,
            polling_minutes: 10,
            enabled: false,
            auth_method: "password".into(),
            oauth_client_id: String::new(),
            oauth_tenant: "common".into(),
        }
    }
}
