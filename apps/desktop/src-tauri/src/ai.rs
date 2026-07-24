use crate::{
    credentials,
    db::{
        AiApplicationContext, AiProviderSettings, CreateInterviewQuestion, Database,
        InterviewQuestionReview, InterviewSessionRecord, StoredInterviewPreparation,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::{Duration, Instant};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderConnectionResult {
    pub ok: bool,
    pub model: String,
    pub duration_ms: i64,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InterviewPreparation {
    pub summary: String,
    pub resume_match: ResumeMatch,
    pub focus_areas: Vec<FocusArea>,
    pub predicted_questions: Vec<PredictedQuestion>,
    pub action_plan: Vec<ActionItem>,
    pub source_notes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResumeMatch {
    pub summary: String,
    pub strengths: Vec<String>,
    pub risks: Vec<String>,
    pub evidence_to_prepare: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FocusArea {
    pub title: String,
    pub reason: String,
    pub priority: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PredictedQuestion {
    pub question: String,
    pub rationale: String,
    pub source_basis: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ResumeQuestionSet {
    questions: Vec<PredictedQuestion>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct InterviewReviewSet {
    summary: String,
    reviews: Vec<InterviewQuestionReview>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct TranscriptInterview {
    round: String,
    questions: Vec<TranscriptQuestion>,
}

#[derive(Debug, Deserialize, Serialize)]
struct TranscriptQuestion {
    question: String,
    answer: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionItem {
    pub action: String,
    pub estimated_minutes: i64,
}

pub async fn test_connection(database: &Database) -> Result<ProviderConnectionResult, String> {
    let settings = database.get_provider_settings()?.ai;
    let api_key = credentials::get_secret("ai_api_key")?;
    let sources = "[]";
    let call_id = database.start_ai_call(
        None,
        "connection_test",
        &settings.provider,
        &settings.model,
        sources,
    )?;
    let started = Instant::now();
    let result = request_text(
        &settings,
        &api_key,
        &settings.model,
        "你是连接测试助手。只回复 OK。",
        "回复 OK",
        None,
    )
    .await;
    let duration_ms = elapsed_ms(started);
    match result {
        Ok(_) => {
            database.finish_ai_call(&call_id, "succeeded", 1, duration_ms, None, None)?;
            Ok(ProviderConnectionResult {
                ok: true,
                model: settings.model,
                duration_ms,
                message: "连接成功".into(),
            })
        }
        Err(error) => {
            database.finish_ai_call(&call_id, "failed", 1, duration_ms, None, Some(&error))?;
            Err(error)
        }
    }
}

pub async fn generate_interview_preparation(
    database: &Database,
    application_id: &str,
    confirm_ai_send: bool,
) -> Result<StoredInterviewPreparation, String> {
    let context = database.get_ai_application_context(application_id)?;
    if context
        .jd_raw
        .as_deref()
        .unwrap_or_default()
        .trim()
        .is_empty()
    {
        return Err("请先在投递详情中补充岗位 JD".to_string());
    }
    let settings = database.get_provider_settings()?.ai;
    if context.resume.is_some() && !settings.allow_resume {
        return Err("当前投递关联了简历，请先在 AI 服务设置中允许发送简历内容".to_string());
    }
    if context.resume.is_some() && settings.prompt_before_send && !confirm_ai_send {
        return Err("发送简历前需要用户确认".to_string());
    }
    let api_key = credentials::get_secret("ai_api_key")?;
    let sources = source_snapshot(&context);
    let sources_json = serde_json::to_string(&sources).map_err(|error| error.to_string())?;
    let call_id = database.start_ai_call(
        Some(application_id),
        "interview_preparation",
        &settings.provider,
        &settings.model,
        &sources_json,
    )?;
    let started = Instant::now();
    let system = "你是严谨的求职面试准备助手。只能根据提供的岗位 JD、关联简历和投递上下文生成建议；不得虚构候选人经历。先完成简历与 JD 的匹配分析，再生成准备重点和预测问题。建议必须具体、可执行，并在 sourceBasis 中标明来自 JD、简历中的具体字段或投递上下文。输出简体中文。";
    let user = format!(
        "请为以下投递生成面试准备方案：\n{}",
        serde_json::to_string_pretty(&context).map_err(|error| error.to_string())?
    );

    let models = candidate_models(&settings);
    let mut attempts = 0_i64;
    let mut last_error = "AI 未返回有效结果".to_string();
    for model in models {
        for repair in 0..2 {
            attempts += 1;
            let schema_fallback = repair > 0 && schema_transport_is_unsupported(&last_error);
            let repair_note = if repair == 0 {
                String::new()
            } else if schema_fallback {
                format!(
                    "\n当前服务不支持 JSON Schema 请求参数，请直接按以下 Schema 输出纯 JSON：\n{}",
                    interview_schema()
                )
            } else {
                "\n上一次输出未通过 Schema 校验。请重新生成并严格满足所有必填字段、类型和枚举值。"
                    .to_string()
            };
            match request_text(
                &settings,
                &api_key,
                &model,
                system,
                &format!("{user}{repair_note}"),
                (!schema_fallback).then(interview_schema),
            )
            .await
            {
                Ok(text) => {
                    match serde_json::from_str::<InterviewPreparation>(strip_json_fence(&text)) {
                        Ok(preparation) => {
                            if let Err(error) = validate_preparation(&preparation) {
                                last_error = error;
                                continue;
                            }
                            let content_json = serde_json::to_string(&preparation)
                                .map_err(|error| error.to_string())?;
                            let duration_ms = elapsed_ms(started);
                            database.finish_ai_call(
                                &call_id,
                                "succeeded",
                                attempts,
                                duration_ms,
                                Some(&content_json),
                                None,
                            )?;
                            return database.save_interview_preparation(
                                application_id,
                                &call_id,
                                &content_json,
                                &sources_json,
                                &model,
                            );
                        }
                        Err(error) => last_error = format!("结构化输出解析失败: {error}"),
                    }
                }
                Err(error) => last_error = error,
            }
        }
    }
    let duration_ms = elapsed_ms(started);
    database.finish_ai_call(
        &call_id,
        "failed",
        attempts,
        duration_ms,
        None,
        Some(&last_error),
    )?;
    Err(last_error)
}

pub async fn generate_resume_questions(
    database: &Database,
    application_id: &str,
    count: i64,
    confirm_ai_send: bool,
) -> Result<Vec<PredictedQuestion>, String> {
    if !(1..=30).contains(&count) {
        return Err("问题数量必须在 1 到 30 之间".to_string());
    }
    let context = database.get_ai_application_context(application_id)?;
    if context.resume.is_none() {
        return Err("当前投递尚未关联简历".to_string());
    }
    let settings = database.get_provider_settings()?.ai;
    if !settings.allow_resume {
        return Err("请先在 AI 服务设置中允许发送简历内容".to_string());
    }
    if settings.prompt_before_send && !confirm_ai_send {
        return Err("发送简历前需要用户确认".to_string());
    }
    let api_key = credentials::get_secret("ai_api_key")?;
    let sources = source_snapshot(&context);
    let sources_json = serde_json::to_string(&sources).map_err(|error| error.to_string())?;
    let call_id = database.start_ai_call(
        Some(application_id),
        "resume_questions",
        &settings.provider,
        &settings.model,
        &sources_json,
    )?;
    let started = Instant::now();
    let user = format!(
        "请根据以下投递关联的真实简历和岗位 JD，生成 {count} 道针对候选人具体经历的深挖问题。问题必须能在简历中找到明确依据，不得生成与该候选人无关的通用占位题。\n{}",
        serde_json::to_string_pretty(&context).map_err(|error| error.to_string())?
    );
    let mut attempts = 0_i64;
    let mut last_error = "AI 未返回有效的简历问题".to_string();
    for model in candidate_models(&settings) {
        for repair in 0..2 {
            attempts += 1;
            let schema_fallback = repair > 0 && schema_transport_is_unsupported(&last_error);
            let note = if repair == 0 {
                String::new()
            } else if schema_fallback {
                format!(
                    "\n当前服务不支持 JSON Schema 请求参数，请直接按以下 Schema 输出纯 JSON：\n{}",
                    resume_questions_schema(count)
                )
            } else {
                "\n上一次输出未通过校验，请保证问题数量正确、每题非空并提供简历字段依据。"
                    .to_string()
            };
            match request_text(
                &settings,
                &api_key,
                &model,
                "你是严谨的技术面试官。只根据关联简历与 JD 生成问题，不得虚构项目、公司、技术或量化成果。只输出 JSON。",
                &format!("{user}{note}"),
                (!schema_fallback).then(|| resume_questions_schema(count)),
            ).await {
                Ok(text) => match serde_json::from_str::<ResumeQuestionSet>(strip_json_fence(&text)) {
                    Ok(result) if result.questions.len() == count as usize && result.questions.iter().all(|item| !item.question.trim().is_empty() && !item.source_basis.is_empty()) => {
                        let response = serde_json::to_string(&result).map_err(|error| error.to_string())?;
                        database.finish_ai_call(&call_id, "succeeded", attempts, elapsed_ms(started), Some(&response), None)?;
                        return Ok(result.questions);
                    }
                    Ok(_) => last_error = "简历问题数量或来源依据不符合要求".to_string(),
                    Err(error) => last_error = format!("简历问题结构解析失败: {error}"),
                },
                Err(error) => last_error = error,
            }
        }
    }
    database.finish_ai_call(
        &call_id,
        "failed",
        attempts,
        elapsed_ms(started),
        None,
        Some(&last_error),
    )?;
    Err(last_error)
}

pub async fn generate_interview_review(
    database: &Database,
    session_id: &str,
    confirm_ai_send: bool,
) -> Result<InterviewSessionRecord, String> {
    let session = database.get_interview_session(session_id)?;
    if session.status == "进行中" {
        return Err("请先完成本场面试，再生成复盘".into());
    }
    if session.questions.is_empty() {
        return Err("本场面试没有可复盘的问题".into());
    }
    let settings = database.get_provider_settings()?.ai;
    ensure_transcript_sharing_allowed(&settings)?;
    ensure_sensitive_send_confirmed(&settings, confirm_ai_send)?;
    let api_key = credentials::get_secret("ai_api_key")?;
    let sources = json!([{ "type": "interview_session", "sessionId": session_id, "questionCount": session.questions.len() }]);
    let call_id = database.start_ai_call(
        Some(&session.application_id),
        "interview_review",
        &settings.provider,
        &settings.model,
        &sources.to_string(),
    )?;
    let started = Instant::now();
    let user = format!(
        "请对以下真实问答记录逐题评分并给出具体改进建议。未作答题应给低分并明确指出未作答；不得编造候选人没有说过的内容。\n{}",
        serde_json::to_string_pretty(&session).map_err(|error| error.to_string())?
    );
    let schema = interview_review_schema(&session);
    let mut attempts = 0_i64;
    let mut last_error = "AI 未返回有效复盘".to_string();
    for model in candidate_models(&settings) {
        for repair in 0..2 {
            attempts += 1;
            let schema_fallback = repair > 0 && schema_transport_is_unsupported(&last_error);
            let note = if repair == 0 {
                String::new()
            } else {
                "\n上一次输出无效。必须为输入中的每个 questionId 返回且只返回一条评价，分数为 0 到 100。".into()
            };
            match request_text(
                &settings,
                &api_key,
                &model,
                "你是严谨的面试复盘教练。评价必须基于候选人的实际回答，关注准确性、完整性、结构、证据和表达；只输出 JSON。",
                &format!("{user}{note}"),
                (!schema_fallback).then(|| schema.clone()),
            )
            .await
            {
                Ok(text) => match serde_json::from_str::<InterviewReviewSet>(strip_json_fence(&text)) {
                    Ok(result) if valid_interview_review(&session, &result) => {
                        let response = serde_json::to_string(&result).map_err(|error| error.to_string())?;
                        let reviewed = database.save_interview_session_review(
                            session_id,
                            &result.summary,
                            &result.reviews,
                        )?;
                        database.finish_ai_call(
                            &call_id,
                            "succeeded",
                            attempts,
                            elapsed_ms(started),
                            Some(&response),
                            None,
                        )?;
                        return Ok(reviewed);
                    }
                    Ok(_) => last_error = "复盘结果的问题编号、数量或分数无效".into(),
                    Err(error) => last_error = format!("复盘结果解析失败: {error}"),
                },
                Err(error) => last_error = error,
            }
        }
    }
    database.finish_ai_call(
        &call_id,
        "failed",
        attempts,
        elapsed_ms(started),
        None,
        Some(&last_error),
    )?;
    Err(last_error)
}

pub async fn import_interview_transcript(
    database: &Database,
    application_id: &str,
    transcript: &str,
    confirm_ai_send: bool,
) -> Result<InterviewSessionRecord, String> {
    let transcript = transcript.trim();
    if transcript.is_empty() {
        return Err("复盘材料中没有可解析的文字".into());
    }
    if transcript.chars().count() > crate::MAX_INTERVIEW_MATERIAL_CHARACTERS {
        return Err("复盘材料过长，请精简到 6 万字以内".into());
    }
    let context = database.get_interview_review_application_context(application_id)?;
    let settings = database.get_provider_settings()?.ai;
    ensure_transcript_sharing_allowed(&settings)?;
    ensure_sensitive_send_confirmed(&settings, confirm_ai_send)?;
    let api_key = credentials::get_secret("ai_api_key")?;
    let sources =
        json!([{ "type": "interview_transcript", "characters": transcript.chars().count() }]);
    let call_id = database.start_ai_call(
        Some(application_id),
        "interview_transcript_parse",
        &settings.provider,
        &settings.model,
        &sources.to_string(),
    )?;
    let started = Instant::now();
    let user = format!(
        "岗位：{} · {}\n请从以下面试记录中按发生顺序还原面试官问题和候选人回答。不要把旁白、时间戳、说话人标签当成问题；无法确定的回答留空。\n\n{}",
        context.company_name, context.position_title, transcript
    );
    let schema = transcript_schema();
    let mut attempts = 0_i64;
    let mut last_error = "AI 未能从材料中识别问答".to_string();
    for model in candidate_models(&settings) {
        for repair in 0..2 {
            attempts += 1;
            let schema_fallback = repair > 0 && schema_transport_is_unsupported(&last_error);
            let repair_note = if repair == 0 {
                String::new()
            } else if schema_fallback {
                format!(
                    "\n当前服务不支持 JSON Schema 请求参数，请直接按以下 Schema 输出纯 JSON：\n{}",
                    schema
                )
            } else {
                format!(
                    "\n上一次输出无法解析（{}）。请重新输出完整 JSON；round、question、answer 的值必须都是字符串，questions 必须是数组。不要把字段值包装成对象。",
                    truncate(&last_error, 500)
                )
            };
            match request_text(
                &settings,
                &api_key,
                &model,
                "你是面试记录整理助手。只能忠实整理输入材料，不得补写没有出现的问题或答案；只输出 JSON。",
                &format!("{user}{repair_note}"),
                (!schema_fallback).then(|| schema.clone()),
            )
            .await
            {
                Ok(text) => match parse_transcript_interview(&text) {
                    Ok(result)
                        if !result.questions.is_empty()
                            && result.questions.len() <= crate::MAX_INTERVIEW_QUESTIONS
                            && result.questions.iter().all(|item| !item.question.trim().is_empty()) =>
                    {
                        let questions: Vec<CreateInterviewQuestion> = result
                            .questions
                            .into_iter()
                            .map(|item| CreateInterviewQuestion {
                                prompt: item.question,
                                source: "真实面试".into(),
                                answer: item.answer,
                            })
                            .collect();
                        let session = database.create_interview_session(
                            application_id,
                            "真实面试",
                            result.round.trim().if_empty("真实面试"),
                            "待复盘",
                            &questions,
                        )?;
                        database.finish_ai_call(
                            &call_id,
                            "succeeded",
                            attempts,
                            elapsed_ms(started),
                            Some(&serde_json::to_string(&session).map_err(|error| error.to_string())?),
                            None,
                        )?;
                        return Ok(session);
                    }
                    Ok(_) => last_error = "材料中没有识别到有效问答".into(),
                    Err(error) => last_error = format!("问答结构解析失败: {error}"),
                },
                Err(error) => last_error = error,
            }
        }
    }
    database.finish_ai_call(
        &call_id,
        "failed",
        attempts,
        elapsed_ms(started),
        None,
        Some(&last_error),
    )?;
    Err(last_error)
}

trait IfEmpty {
    fn if_empty<'a>(&'a self, fallback: &'a str) -> &'a str;
}

impl IfEmpty for str {
    fn if_empty<'a>(&'a self, fallback: &'a str) -> &'a str {
        if self.is_empty() {
            fallback
        } else {
            self
        }
    }
}

fn valid_interview_review(session: &InterviewSessionRecord, result: &InterviewReviewSet) -> bool {
    if result.summary.trim().is_empty() || result.reviews.len() != session.questions.len() {
        return false;
    }
    let expected: std::collections::HashSet<&str> = session
        .questions
        .iter()
        .map(|question| question.id.as_str())
        .collect();
    let actual: std::collections::HashSet<&str> = result
        .reviews
        .iter()
        .map(|review| review.question_id.as_str())
        .collect();
    expected == actual
        && result
            .reviews
            .iter()
            .all(|review| (0..=100).contains(&review.score) && !review.evaluation.trim().is_empty())
}

fn interview_review_schema(session: &InterviewSessionRecord) -> Value {
    let ids: Vec<&str> = session
        .questions
        .iter()
        .map(|question| question.id.as_str())
        .collect();
    json!({
        "type":"object","additionalProperties":false,
        "properties":{
            "summary":{"type":"string"},
            "reviews":{"type":"array","minItems":ids.len(),"maxItems":ids.len(),"items":{
                "type":"object","additionalProperties":false,
                "properties":{
                    "questionId":{"type":"string","enum":ids},
                    "score":{"type":"integer","minimum":0,"maximum":100},
                    "evaluation":{"type":"string"}
                },
                "required":["questionId","score","evaluation"]
            }}
        },
        "required":["summary","reviews"]
    })
}

fn transcript_schema() -> Value {
    json!({
        "type":"object","additionalProperties":false,
        "properties":{
            "round":{"type":"string"},
            "questions":{"type":"array","minItems":1,"maxItems":crate::MAX_INTERVIEW_QUESTIONS,"items":{
                "type":"object","additionalProperties":false,
                "properties":{"question":{"type":"string"},"answer":{"type":"string"}},
                "required":["question","answer"]
            }}
        },
        "required":["round","questions"]
    })
}

fn resume_questions_schema(count: i64) -> Value {
    json!({
        "type":"object","additionalProperties":false,
        "properties":{"questions":{"type":"array","minItems":count,"maxItems":count,"items":{
            "type":"object","additionalProperties":false,
            "properties":{"question":{"type":"string"},"rationale":{"type":"string"},"sourceBasis":{"type":"array","minItems":1,"items":{"type":"string"}}},
            "required":["question","rationale","sourceBasis"]
        }}},
        "required":["questions"]
    })
}

fn schema_transport_is_unsupported(error: &str) -> bool {
    let error = error.to_ascii_lowercase();
    error.contains("response_format")
        || error.contains("json_schema")
        || error.contains("structured output")
        || error.contains("结构化输出不支持")
        || error.contains("不支持结构化输出")
}

fn strip_json_fence(value: &str) -> &str {
    let value = value.trim();
    let value = value
        .strip_prefix("```json")
        .or_else(|| value.strip_prefix("```"))
        .unwrap_or(value);
    value.strip_suffix("```").unwrap_or(value).trim()
}

fn parse_transcript_interview(text: &str) -> Result<TranscriptInterview, String> {
    let text = strip_json_fence(text);
    match serde_json::from_str::<TranscriptInterview>(text) {
        Ok(result) => Ok(result),
        Err(strict_error) => {
            let value: Value = serde_json::from_str(text).map_err(|_| strict_error.to_string())?;
            normalize_transcript_interview(&value).ok_or_else(|| strict_error.to_string())
        }
    }
}

fn normalize_transcript_interview(value: &Value) -> Option<TranscriptInterview> {
    let (round, questions) = match value {
        Value::Object(object) => {
            let round = ["round", "interviewRound", "title", "轮次"]
                .iter()
                .find_map(|key| object.get(*key).and_then(json_text))
                .unwrap_or_default();
            let questions = [
                "questions",
                "qaPairs",
                "qa_pairs",
                "questionAnswerPairs",
                "items",
                "records",
                "问答",
            ]
            .iter()
            .find_map(|key| object.get(*key))?;
            (round, questions)
        }
        Value::Array(_) => (String::new(), value),
        _ => return None,
    };
    let items: Vec<&Value> = match questions {
        Value::Array(items) => items.iter().collect(),
        Value::Object(items) => items.values().collect(),
        _ => return None,
    };
    let questions = items
        .into_iter()
        .filter_map(normalize_transcript_question)
        .collect::<Vec<_>>();
    (!questions.is_empty()).then_some(TranscriptInterview { round, questions })
}

fn normalize_transcript_question(value: &Value) -> Option<TranscriptQuestion> {
    let object = value.as_object()?;
    let question = ["question", "prompt", "q", "问题", "提问"]
        .iter()
        .find_map(|key| object.get(*key).and_then(json_text))?;
    if question.trim().is_empty() {
        return None;
    }
    let answer = ["answer", "response", "a", "回答", "答案"]
        .iter()
        .find_map(|key| object.get(*key).and_then(json_text))
        .unwrap_or_default();
    Some(TranscriptQuestion { question, answer })
}

fn json_text(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.trim().to_string()),
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(boolean) => Some(boolean.to_string()),
        Value::Array(items) => {
            let text = items
                .iter()
                .filter_map(json_text)
                .collect::<Vec<_>>()
                .join("\n");
            (!text.is_empty()).then_some(text)
        }
        Value::Object(object) => {
            for key in [
                "text", "content", "value", "name", "title", "label", "question", "answer",
                "response", "问题", "回答",
            ] {
                if let Some(text) = object.get(key).and_then(json_text) {
                    return Some(text);
                }
            }
            if object.len() == 1 {
                return object.values().next().and_then(json_text);
            }
            None
        }
        Value::Null => None,
    }
}

pub(crate) fn candidate_models(settings: &AiProviderSettings) -> Vec<String> {
    let mut models = vec![settings.model.clone()];
    if let Some(fallback) = settings
        .fallback_model
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        if fallback != settings.model {
            models.push(fallback.to_string());
        }
    }
    models
}

pub(crate) async fn request_text(
    settings: &AiProviderSettings,
    api_key: &str,
    model: &str,
    system: &str,
    user: &str,
    schema: Option<Value>,
) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(settings.timeout_seconds as u64))
        .build()
        .map_err(|error| format!("创建 AI 客户端失败: {error}"))?;
    let (endpoint, body, anthropic) = match settings.protocol.as_str() {
        "anthropic" => {
            let system = schema.as_ref().map_or_else(
                || system.to_string(),
                |schema| {
                    format!(
                        "{system}\n请只输出符合以下 JSON Schema 的 JSON，不要使用 Markdown 代码块：\n{schema}"
                    )
                },
            );
            (
                format!("{}/messages", settings.base_url.trim_end_matches('/')),
                json!({
                    "model": model, "system": system,
                    "messages": [{ "role": "user", "content": user }],
                    "max_tokens": settings.max_output_tokens
                }),
                true,
            )
        }
        "chat" => {
            let mut body = json!({
                "model": model,
                "messages": [
                    { "role": "system", "content": system },
                    { "role": "user", "content": user }
                ],
                "max_tokens": settings.max_output_tokens
            });
            if let Some(schema) = schema {
                body["response_format"] = json!({ "type": "json_schema", "json_schema": {
                    "name": "structured_output", "strict": true, "schema": schema
                }});
            }
            (
                format!(
                    "{}/chat/completions",
                    settings.base_url.trim_end_matches('/')
                ),
                body,
                false,
            )
        }
        "responses" => {
            let mut body = json!({
                "model": model,
                "input": [
                    { "role": "system", "content": system },
                    { "role": "user", "content": user }
                ],
                "max_output_tokens": settings.max_output_tokens,
                "store": false
            });
            if let Some(schema) = schema {
                body["text"] = json!({ "format": {
                    "type": "json_schema", "name": "structured_output",
                    "strict": true, "schema": schema
                }});
            }
            (
                format!("{}/responses", settings.base_url.trim_end_matches('/')),
                body,
                false,
            )
        }
        _ => return Err("AI 接口协议无效，请重新保存设置".into()),
    };
    let request = client.post(endpoint).json(&body);
    let request = if anthropic {
        request
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
    } else {
        request.bearer_auth(api_key)
    };
    let response = request
        .send()
        .await
        .map_err(|error| format!("AI 请求失败: {error}"))?;
    let (status, value) = crate::http::read_json_response(response, 8 * 1024 * 1024, "AI").await?;
    if !status.is_success() {
        let detail = value
            .pointer("/error/message")
            .and_then(Value::as_str)
            .unwrap_or("服务返回错误");
        return Err(format!("AI 服务错误 ({status}): {}", truncate(detail, 400)));
    }
    extract_output_text(&value, settings.protocol.as_str())
        .ok_or_else(|| "AI 响应中没有可用文本".to_string())
}

fn extract_output_text(value: &Value, protocol: &str) -> Option<String> {
    if protocol == "chat" {
        return value
            .pointer("/choices/0/message/content")
            .and_then(Value::as_str)
            .map(str::to_string);
    }
    if protocol == "anthropic" {
        return value
            .pointer("/content/0/text")
            .and_then(Value::as_str)
            .map(str::to_string);
    }
    value
        .get("output")?
        .as_array()?
        .iter()
        .flat_map(|item| {
            item.get("content")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .find_map(|content| {
            content
                .get("text")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
}

fn source_snapshot(context: &AiApplicationContext) -> Value {
    json!([
        { "type": "job_description", "field": "positions.jd_raw", "characters": context.jd_raw.as_deref().unwrap_or_default().chars().count() },
        { "type": "application_context", "fields": ["company_name", "position_title", "department", "location", "current_stage", "next_action"] },
        { "type": "company_notes", "included": context.company_notes.as_deref().is_some_and(|value| !value.trim().is_empty()) }
        ,{ "type": "resume_profile", "included": context.resume.is_some(), "profileId": context.resume.as_ref().map(|resume| resume.id.as_str()), "profileName": context.resume.as_ref().map(|resume| resume.name.as_str()) }
    ])
}

fn validate_preparation(value: &InterviewPreparation) -> Result<(), String> {
    if value.summary.trim().is_empty()
        || value.focus_areas.is_empty()
        || value.predicted_questions.is_empty()
        || value.action_plan.is_empty()
    {
        return Err("结构化输出缺少必要内容，已准备重试".to_string());
    }
    if value
        .focus_areas
        .iter()
        .any(|item| !matches!(item.priority.as_str(), "high" | "medium" | "low"))
    {
        return Err("结构化输出包含无效优先级，已准备重试".to_string());
    }
    Ok(())
}

fn interview_schema() -> Value {
    json!({
        "type": "object", "additionalProperties": false,
        "properties": {
            "summary": { "type": "string" },
            "resumeMatch": { "type": "object", "additionalProperties": false,
                "properties": {
                    "summary": {"type":"string"},
                    "strengths": {"type":"array","items":{"type":"string"}},
                    "risks": {"type":"array","items":{"type":"string"}},
                    "evidenceToPrepare": {"type":"array","items":{"type":"string"}}
                },
                "required": ["summary","strengths","risks","evidenceToPrepare"] },
            "focusAreas": { "type": "array", "items": { "type": "object", "additionalProperties": false,
                "properties": { "title": {"type":"string"}, "reason": {"type":"string"}, "priority": {"type":"string", "enum":["high","medium","low"]} },
                "required": ["title","reason","priority"] } },
            "predictedQuestions": { "type": "array", "items": { "type": "object", "additionalProperties": false,
                "properties": { "question":{"type":"string"}, "rationale":{"type":"string"}, "sourceBasis":{"type":"array","items":{"type":"string"}} },
                "required": ["question","rationale","sourceBasis"] } },
            "actionPlan": { "type": "array", "items": { "type": "object", "additionalProperties": false,
                "properties": { "action":{"type":"string"}, "estimatedMinutes":{"type":"integer","minimum":1,"maximum":480} },
                "required": ["action","estimatedMinutes"] } },
            "sourceNotes": { "type":"array", "items":{"type":"string"} }
        },
        "required": ["summary","resumeMatch","focusAreas","predictedQuestions","actionPlan","sourceNotes"]
    })
}

fn elapsed_ms(started: Instant) -> i64 {
    started.elapsed().as_millis().min(i64::MAX as u128) as i64
}
fn ensure_transcript_sharing_allowed(settings: &AiProviderSettings) -> Result<(), String> {
    if settings.allow_transcript {
        Ok(())
    } else {
        Err("隐私设置不允许向 AI 服务发送面试转写内容".into())
    }
}
fn ensure_sensitive_send_confirmed(
    settings: &AiProviderSettings,
    confirm_ai_send: bool,
) -> Result<(), String> {
    if !settings.prompt_before_send || confirm_ai_send {
        Ok(())
    } else {
        Err("发送敏感内容前需要用户确认".into())
    }
}
fn truncate(value: &str, max: usize) -> String {
    value.chars().take(max).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_responses_output_text() {
        let value = json!({"output":[{"type":"message","content":[{"type":"output_text","text":"{\"summary\":\"ok\"}"}]}]});
        assert_eq!(
            extract_output_text(&value, "responses").as_deref(),
            Some("{\"summary\":\"ok\"}")
        );
    }

    #[test]
    fn transcript_parser_accepts_strict_schema_output() {
        let result = parse_transcript_interview(
            r#"{"round":"一面","questions":[{"question":"什么是 RAG？","answer":"检索增强生成。"}]}"#,
        )
        .unwrap();
        assert_eq!(result.round, "一面");
        assert_eq!(result.questions.len(), 1);
        assert_eq!(result.questions[0].question, "什么是 RAG？");
        assert_eq!(result.questions[0].answer, "检索增强生成。");
    }

    #[test]
    fn transcript_parser_normalizes_object_wrapped_text() {
        let result = parse_transcript_interview(
            r#"{
                "round":{"text":"技术一面"},
                "questions":[
                    {
                        "question":{"content":"请介绍一个 AI 项目。"},
                        "answer":{"text":"我做过企业知识库问答。"}
                    },
                    {
                        "question":"什么是 RAG？",
                        "answer":["先检索外部知识库。","再把结果交给模型生成答案。"]
                    }
                ]
            }"#,
        )
        .unwrap();
        assert_eq!(result.round, "技术一面");
        assert_eq!(result.questions.len(), 2);
        assert_eq!(result.questions[0].question, "请介绍一个 AI 项目。");
        assert_eq!(result.questions[0].answer, "我做过企业知识库问答。");
        assert_eq!(
            result.questions[1].answer,
            "先检索外部知识库。\n再把结果交给模型生成答案。"
        );
    }

    #[test]
    fn transcript_parser_accepts_common_aliases_and_blank_answers() {
        let result = parse_transcript_interview(
            r#"{"interviewRound":"初面","qaPairs":[{"q":"为什么离职？"},{"问题":"期望薪资？","回答":""}]}"#,
        )
        .unwrap();
        assert_eq!(result.round, "初面");
        assert_eq!(result.questions.len(), 2);
        assert_eq!(result.questions[0].answer, "");
    }

    #[test]
    fn transcript_parser_accepts_snake_case_pairs_without_parsing_metadata() {
        let result = parse_transcript_interview(
            r#"{
                "qa_pairs":[{"question":"什么是 RAG？","answer":"检索增强生成。"}],
                "metadata":{"question":"伪造的问题","answer":"伪造的回答"}
            }"#,
        )
        .unwrap();
        assert_eq!(result.questions.len(), 1);
        assert_eq!(result.questions[0].question, "什么是 RAG？");
    }

    #[test]
    fn transcript_parser_rejects_objects_without_a_question_collection() {
        let result = parse_transcript_interview(
            r#"{"metadata":{"question":"不应被解析的问题","answer":"不应被解析的回答"}}"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn schema_uses_strict_camel_case_fields() {
        let schema = interview_schema();
        assert_eq!(schema["additionalProperties"], false);
        assert!(schema["properties"]["predictedQuestions"].is_object());
    }

    #[test]
    fn chinese_schema_errors_enable_transport_fallback() {
        assert!(schema_transport_is_unsupported("当前服务不支持结构化输出"));
        assert!(schema_transport_is_unsupported("结构化输出不支持此模型"));
    }

    #[test]
    fn transcript_privacy_setting_is_enforced_before_network_access() {
        let mut settings = AiProviderSettings {
            allow_transcript: false,
            prompt_before_send: true,
            ..AiProviderSettings::default()
        };
        assert!(ensure_transcript_sharing_allowed(&settings).is_err());
        settings.allow_transcript = true;
        assert!(ensure_transcript_sharing_allowed(&settings).is_ok());
        assert!(ensure_sensitive_send_confirmed(&settings, false).is_err());
        assert!(ensure_sensitive_send_confirmed(&settings, true).is_ok());
    }
}
