use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiApplicationContext {
    pub application_id: String,
    pub company_name: String,
    pub position_title: String,
    pub department: Option<String>,
    pub location: Option<String>,
    pub current_stage: String,
    pub jd_raw: Option<String>,
    pub company_notes: Option<String>,
    pub next_action: Option<String>,
    pub resume: Option<ResumeAiContext>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResumeAiContext {
    pub id: String,
    pub name: String,
    pub target_direction: String,
    pub personal: serde_json::Value,
    pub education: serde_json::Value,
    pub internships: serde_json::Value,
    pub projects: serde_json::Value,
    pub skills: String,
    pub academics: serde_json::Value,
    pub certificates: serde_json::Value,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredInterviewPreparation {
    pub id: String,
    pub application_id: String,
    pub ai_call_id: String,
    pub content: serde_json::Value,
    pub sources: serde_json::Value,
    pub model: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiCallSummary {
    pub id: String,
    pub feature: String,
    pub model: String,
    pub status: String,
    pub attempts: i64,
    pub duration_ms: Option<i64>,
    pub input_sources: serde_json::Value,
    pub error_message: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessingJobResult {
    pub id: String,
    pub kind: String,
    pub status: String,
    pub result: Option<serde_json::Value>,
    pub duration_ms: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessingJobSummary {
    pub id: String,
    pub kind: String,
    pub application_id: Option<String>,
    pub source_path: String,
    pub source_size_bytes: Option<u64>,
    pub status: String,
    pub duration_ms: Option<i64>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub import_status: String,
    pub interview_session_id: Option<String>,
    pub import_error_message: Option<String>,
    pub import_started_at: Option<String>,
    pub import_completed_at: Option<String>,
    pub text_preview: Option<String>,
    pub character_count: Option<i64>,
    pub progress_phase: Option<String>,
    pub progress_completed: Option<i64>,
    pub progress_total: Option<i64>,
    pub progress_message: Option<String>,
}
