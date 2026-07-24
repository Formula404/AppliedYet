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
            allow_resume: true,
            allow_transcript: true,
            prompt_before_send: false,
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

impl AsrProviderSettings {
    pub fn supports_speaker_diarization(&self) -> bool {
        url::Url::parse(self.base_url.trim())
            .ok()
            .and_then(|value| value.host_str().map(str::to_ascii_lowercase))
            .is_some_and(|host| host == "api.openai.com")
            && self.model.to_ascii_lowercase().contains("diarize")
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
    pub accounts: Vec<EmailAccountSettings>,
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
            accounts: Vec::new(),
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

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct EmailAccountSettings {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub provider: String,
    pub email_address: String,
    pub imap_host: String,
    pub imap_port: i64,
    pub username: String,
    pub use_tls: bool,
    pub auth_method: String,
    pub oauth_client_id: String,
    pub oauth_tenant: String,
}

impl Default for EmailAccountSettings {
    fn default() -> Self {
        let legacy = EmailSettings::default();
        Self {
            id: String::new(),
            name: String::new(),
            enabled: true,
            provider: legacy.provider,
            email_address: legacy.email_address,
            imap_host: legacy.imap_host,
            imap_port: legacy.imap_port,
            username: legacy.username,
            use_tls: legacy.use_tls,
            auth_method: legacy.auth_method,
            oauth_client_id: legacy.oauth_client_id,
            oauth_tenant: legacy.oauth_tenant,
        }
    }
}

impl EmailSettings {
    pub fn active_accounts(&self) -> Vec<EmailAccountSettings> {
        if !self.accounts.is_empty() {
            return self
                .accounts
                .iter()
                .filter(|account| account.enabled)
                .cloned()
                .collect();
        }
        if self.email_address.trim().is_empty() && self.username.trim().is_empty() {
            return Vec::new();
        }
        vec![EmailAccountSettings {
            id: "legacy".into(),
            name: self.email_address.clone(),
            enabled: self.enabled,
            provider: self.provider.clone(),
            email_address: self.email_address.clone(),
            imap_host: self.imap_host.clone(),
            imap_port: self.imap_port,
            username: self.username.clone(),
            use_tls: self.use_tls,
            auth_method: self.auth_method.clone(),
            oauth_client_id: self.oauth_client_id.clone(),
            oauth_tenant: self.oauth_tenant.clone(),
        }]
    }
}
