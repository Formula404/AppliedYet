use crate::{
    credentials,
    db::{Database, ProcessingJobResult},
};
use serde_json::{json, Value};
use std::{
    fs,
    time::{Duration, Instant},
};

pub async fn transcribe_audio(
    database: &Database,
    application_id: Option<&str>,
    path: &str,
) -> Result<ProcessingJobResult, String> {
    let settings = database.get_provider_settings()?.asr;
    let metadata = fs::metadata(path).map_err(|error| format!("无法读取音频文件: {error}"))?;
    if !metadata.is_file() {
        return Err("音频路径不是文件".to_string());
    }
    let file_limit_mb = u64::try_from(settings.file_limit_mb)
        .ok()
        .filter(|value| (1..=2048).contains(value))
        .ok_or_else(|| "ASR 文件大小限制无效，请重新保存设置".to_string())?;
    if metadata.len() > file_limit_mb * 1024 * 1024 {
        return Err(format!("音频超过 {} MB 限制", settings.file_limit_mb));
    }
    let job_id = database.start_processing_job("asr", application_id, path)?;
    let started = Instant::now();
    let result = async {
        let api_key = credentials::get_secret("asr_api_key")?;
        let part = reqwest::multipart::Part::file(path)
            .await
            .map_err(|error| format!("读取音频失败: {error}"))?;
        let mut form = reqwest::multipart::Form::new()
            .part("file", part)
            .text("model", settings.model.clone());
        if settings.language != "auto" {
            form = form.text("language", settings.language.clone());
        }
        if settings.speaker_diarization {
            form = form
                .text("response_format", "diarized_json")
                .text("chunking_strategy", "auto");
        } else {
            form = form.text("response_format", "json");
        }
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(1800))
            .build()
            .map_err(|error| error.to_string())?;
        let endpoint = format!(
            "{}/audio/transcriptions",
            settings.base_url.trim_end_matches('/')
        );
        let response = client
            .post(endpoint)
            .bearer_auth(api_key)
            .multipart(form)
            .send()
            .await
            .map_err(|error| format!("语音识别请求失败: {error}"))?;
        let (status, value) =
            crate::http::read_json_response(response, 32 * 1024 * 1024, "语音识别").await?;
        if !status.is_success() {
            let message = value
                .pointer("/error/message")
                .and_then(Value::as_str)
                .unwrap_or("服务返回错误");
            return Err(format!(
                "语音识别服务错误 ({status}): {}",
                message.chars().take(400).collect::<String>()
            ));
        }
        Ok(json!({ "model": settings.model, "language": settings.language, "transcript": value }))
    }
    .await;
    let duration = started.elapsed().as_millis().min(i64::MAX as u128) as i64;
    match result {
        Ok(value) => {
            let serialized = serde_json::to_string(&value).map_err(|error| error.to_string())?;
            database.finish_processing_job(&job_id, "succeeded", Some(&serialized), None, duration)
        }
        Err(error) => {
            let _ = database.finish_processing_job(&job_id, "failed", None, Some(&error), duration);
            Err(error)
        }
    }
}
