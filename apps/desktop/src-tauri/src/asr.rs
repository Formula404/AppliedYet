use crate::{
    credentials,
    db::{AsrProviderSettings, Database, ProcessingJobResult},
};
use serde_json::{json, Value};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

const AUDIO_OVERLAP_SECONDS: f64 = 1.0;
const MP3_BITS_PER_SECOND: u64 = 32_000;
const MAX_TRANSCRIPTION_ATTEMPTS: usize = 3;

#[derive(Debug, Clone)]
struct AudioChunk {
    path: PathBuf,
    actual_start: f64,
    keep_start: f64,
    keep_end: f64,
}

#[derive(Debug)]
struct PreparedAudio {
    directory: PathBuf,
    chunks: Vec<AudioChunk>,
    duration_seconds: f64,
    delete_on_drop: bool,
}

#[derive(Debug, Clone, Copy)]
struct AudioInfo {
    duration_seconds: f64,
    audio_streams: usize,
}

#[derive(Debug)]
struct TranscriptionRequestError {
    message: String,
    retryable: bool,
}

impl TranscriptionRequestError {
    fn permanent(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            retryable: false,
        }
    }

    fn retryable(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            retryable: true,
        }
    }
}

impl Drop for PreparedAudio {
    fn drop(&mut self) {
        if self.delete_on_drop {
            let _ = fs::remove_dir_all(&self.directory);
        }
    }
}

pub async fn transcribe_audio(
    database: &Database,
    application_id: Option<&str>,
    path: &str,
) -> Result<ProcessingJobResult, String> {
    let settings = database.get_provider_settings()?.asr;
    let job_id = database.start_processing_job("asr", application_id, path)?;
    let started = Instant::now();
    let result = transcribe_audio_inner(database, &job_id, path, &settings).await;
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

async fn transcribe_audio_inner(
    database: &Database,
    job_id: &str,
    path: &str,
    settings: &AsrProviderSettings,
) -> Result<Value, String> {
    let metadata = fs::metadata(path).map_err(|error| format!("无法读取音频文件: {error}"))?;
    if !metadata.is_file() {
        return Err("音频路径不是文件".to_string());
    }
    if settings.speaker_diarization && !settings.supports_speaker_diarization() {
        return Err(
            "当前接口或模型不支持说话人区分；请使用 OpenAI diarize 模型或关闭该选项".to_string(),
        );
    }
    let upload_limit = effective_upload_limit_bytes(settings)?;
    let api_key = credentials::get_secret("asr_api_key")?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(1800))
        .build()
        .map_err(|error| format!("创建语音识别客户端失败: {error}"))?;

    if settings.speaker_diarization {
        if metadata.len() > upload_limit {
            return Err("说话人区分暂不支持本地分片；请关闭说话人区分后重试".into());
        }
        let mut value = request_transcription(&client, settings, &api_key, Path::new(path), false)
            .await
            .map_err(|error| error.message)?;
        let transcript = required_transcript_text(&value)?;
        ensure_transcript_length(&transcript)?;
        value["text"] = Value::String(transcript);
        return Ok(json!({
            "model": settings.model,
            "language": settings.language,
            "transcript": value,
            "progress": completed_progress(1)
        }));
    }

    let source = PathBuf::from(path);
    let probe_source = source.clone();
    let audio_info = tauri::async_runtime::spawn_blocking(move || probe_audio_info(&probe_source))
        .await
        .map_err(|error| format!("读取媒体信息任务失败: {error}"))?;
    let should_chunk = is_video_file(&source)
        || audio_info
            .as_ref()
            .is_ok_and(|info| info.duration_seconds > settings.segment_seconds as f64)
        || metadata.len() > upload_limit;

    if !should_chunk {
        match request_transcription(&client, settings, &api_key, &source, true).await {
            Ok(mut value) => {
                let transcript = required_transcript_text(&value)?;
                ensure_transcript_length(&transcript)?;
                value["text"] = Value::String(transcript);
                return Ok(json!({
                    "model": settings.model,
                    "language": settings.language,
                    "transcript": value,
                    "progress": completed_progress(1)
                }));
            }
            Err(error) if upload_too_large(&error.message) => {}
            Err(error) => return Err(error.message),
        }
    }

    database.update_processing_job_progress(
        job_id,
        &progress_value("preparing", 0, 0, "正在压缩并切分音频"),
    )?;
    let source = source.clone();
    let segment_seconds = safe_segment_seconds(settings.segment_seconds, upload_limit);
    let delete_temporary_files = settings.delete_temporary_files;
    let prepared = tauri::async_runtime::spawn_blocking(move || {
        prepare_audio_chunks(&source, segment_seconds, delete_temporary_files)
    })
    .await
    .map_err(|error| format!("音频预处理任务失败: {error}"))??;
    let total = prepared.chunks.len();
    if total == 0 {
        return Err("音频预处理完成，但没有生成可转写的分片".into());
    }

    let mut texts = Vec::with_capacity(total);
    let mut combined_segments = Vec::new();
    for (index, chunk) in prepared.chunks.iter().enumerate() {
        database.update_processing_job_progress(
            job_id,
            &progress_value(
                "transcribing",
                index,
                total,
                &format!("正在转写第 {} / {} 段", index + 1, total),
            ),
        )?;
        let value =
            request_transcription_with_retry(&client, settings, &api_key, &chunk.path).await?;
        let (text, segments) = extract_chunk_transcript(&value, chunk)?;
        if !text.trim().is_empty() {
            texts.push(text);
        }
        combined_segments.extend(segments);
        database.update_processing_job_progress(
            job_id,
            &progress_value(
                "transcribing",
                index + 1,
                total,
                &format!("已完成第 {} / {} 段", index + 1, total),
            ),
        )?;
    }

    database.update_processing_job_progress(
        job_id,
        &progress_value("merging", total, total, "正在合并转写结果"),
    )?;
    let transcript = merge_transcript_parts(&texts);
    if transcript.trim().is_empty() {
        return Err("语音识别完成，但服务没有返回可用文字".into());
    }
    ensure_transcript_length(&transcript)?;
    Ok(json!({
        "model": settings.model,
        "language": settings.language,
        "transcript": {
            "text": transcript,
            "duration": prepared.duration_seconds,
            "segments": combined_segments
        },
        "chunks": {
            "total": total,
            "segmentSeconds": segment_seconds,
            "overlapSeconds": AUDIO_OVERLAP_SECONDS
        },
        "progress": completed_progress(total)
    }))
}

async fn request_transcription_with_retry(
    client: &reqwest::Client,
    settings: &AsrProviderSettings,
    api_key: &str,
    path: &Path,
) -> Result<Value, String> {
    let mut last_error = String::new();
    for attempt in 0..MAX_TRANSCRIPTION_ATTEMPTS {
        match request_transcription(client, settings, api_key, path, true).await {
            Ok(value) => return Ok(value),
            Err(error) if !error.retryable => return Err(error.message),
            Err(error) => last_error = error.message,
        }
        if attempt + 1 < MAX_TRANSCRIPTION_ATTEMPTS {
            tokio::time::sleep(Duration::from_secs(1_u64 << attempt)).await;
        }
    }
    Err(format!(
        "分片转写重试 {} 次后仍失败：{}",
        MAX_TRANSCRIPTION_ATTEMPTS, last_error
    ))
}

async fn request_transcription(
    client: &reqwest::Client,
    settings: &AsrProviderSettings,
    api_key: &str,
    path: &Path,
    prefer_timestamps: bool,
) -> Result<Value, TranscriptionRequestError> {
    let part = reqwest::multipart::Part::file(path)
        .await
        .map_err(|error| TranscriptionRequestError::permanent(format!("读取音频失败: {error}")))?;
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
    } else if prefer_timestamps && supports_verbose_transcript(settings) {
        form = form
            .text("response_format", "verbose_json")
            .text("timestamp_granularities[]", "segment");
    } else {
        form = form.text("response_format", "json");
    }
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
        .map_err(|error| {
            TranscriptionRequestError::retryable(format!("语音识别请求失败: {error}"))
        })?;
    let (status, value) = crate::http::read_json_response(response, 32 * 1024 * 1024, "语音识别")
        .await
        .map_err(TranscriptionRequestError::retryable)?;
    if !status.is_success() {
        let message = value
            .pointer("/error/message")
            .and_then(Value::as_str)
            .unwrap_or("服务返回错误");
        let message = format!(
            "语音识别服务错误 ({status}): {}",
            message.chars().take(400).collect::<String>()
        );
        return Err(if transcription_status_is_retryable(status) {
            TranscriptionRequestError::retryable(message)
        } else {
            TranscriptionRequestError::permanent(message)
        });
    }
    Ok(value)
}

fn transcription_status_is_retryable(status: reqwest::StatusCode) -> bool {
    matches!(status.as_u16(), 408 | 425 | 429) || status.is_server_error()
}

fn prepare_audio_chunks(
    source: &Path,
    segment_seconds: i64,
    delete_on_drop: bool,
) -> Result<PreparedAudio, String> {
    let info = probe_audio_info(source)?;
    let duration = info.duration_seconds;
    if !duration.is_finite() || duration <= 0.0 {
        return Err("无法确定有效的音频时长".into());
    }
    let directory = env::temp_dir()
        .join("appliedyet-asr")
        .join(uuid::Uuid::new_v4().to_string());
    fs::create_dir_all(&directory).map_err(|error| format!("创建音频临时目录失败: {error}"))?;
    let plan = chunk_plan(duration, segment_seconds as f64);
    let ffmpeg = media_tool_path();
    let mut chunks = Vec::with_capacity(plan.len());
    for (index, (actual_start, actual_end, keep_start, keep_end)) in plan.into_iter().enumerate() {
        let output_path = directory.join(format!("chunk_{:04}.mp3", index + 1));
        let mut command = Command::new(&ffmpeg);
        command
            .arg("-hide_banner")
            .arg("-loglevel")
            .arg("error")
            .arg("-y")
            .arg("-ss")
            .arg(format!("{actual_start:.3}"))
            .arg("-i")
            .arg(source)
            .arg("-t")
            .arg(format!("{:.3}", actual_end - actual_start));
        if info.audio_streams > 1 {
            let inputs = (0..info.audio_streams)
                .map(|stream| format!("[0:a:{stream}]"))
                .collect::<String>();
            command
                .arg("-filter_complex")
                .arg(format!(
                    "{inputs}amix=inputs={}:duration=longest:dropout_transition=0[aout]",
                    info.audio_streams
                ))
                .arg("-map")
                .arg("[aout]");
        } else {
            command.arg("-map").arg("0:a:0");
        }
        let output = command
            .arg("-vn")
            .arg("-ac")
            .arg("1")
            .arg("-ar")
            .arg("16000")
            .arg("-c:a")
            .arg("libmp3lame")
            .arg("-b:a")
            .arg("32k")
            .arg(&output_path)
            .output()
            .map_err(|error| {
                format!(
                    "无法启动 FFmpeg（{}）：{error}。请安装 FFmpeg 或设置 APPLIEDYET_FFMPEG_PATH",
                    ffmpeg.display()
                )
            })?;
        if !output.status.success() {
            let detail = String::from_utf8_lossy(&output.stderr);
            let _ = fs::remove_dir_all(&directory);
            return Err(format!(
                "FFmpeg 切分第 {} 段失败：{}",
                index + 1,
                detail.chars().take(500).collect::<String>()
            ));
        }
        chunks.push(AudioChunk {
            path: output_path,
            actual_start,
            keep_start,
            keep_end,
        });
    }
    Ok(PreparedAudio {
        directory,
        chunks,
        duration_seconds: duration,
        delete_on_drop,
    })
}

fn probe_audio_info(path: &Path) -> Result<AudioInfo, String> {
    let ffmpeg = media_tool_path();
    let output = Command::new(&ffmpeg)
        .arg("-hide_banner")
        .arg("-i")
        .arg(path)
        .output()
        .map_err(|error| {
            format!(
                "无法启动 FFmpeg（{}）：{error}。请安装 FFmpeg 或设置 APPLIEDYET_FFMPEG_PATH",
                ffmpeg.display()
            )
        })?;
    let detail = String::from_utf8_lossy(&output.stderr);
    let duration = detail
        .lines()
        .find_map(parse_ffmpeg_duration)
        .ok_or_else(|| {
            format!(
                "无法读取媒体时长或文件中没有可用音轨：{}",
                detail.chars().take(500).collect::<String>()
            )
        })?;
    let audio_streams = detail
        .lines()
        .filter(|line| {
            let line = line.trim();
            line.starts_with("Stream #0:") && line.contains("Audio:")
        })
        .count();
    if audio_streams == 0 {
        return Err("媒体文件中没有可用音轨".into());
    }
    Ok(AudioInfo {
        duration_seconds: duration,
        audio_streams,
    })
}

fn parse_ffmpeg_duration(line: &str) -> Option<f64> {
    let value = line.split("Duration:").nth(1)?.split(',').next()?.trim();
    if value == "N/A" {
        return None;
    }
    let mut parts = value.split(':');
    let hours = parts.next()?.parse::<f64>().ok()?;
    let minutes = parts.next()?.parse::<f64>().ok()?;
    let seconds = parts.next()?.parse::<f64>().ok()?;
    Some(hours * 3600.0 + minutes * 60.0 + seconds)
}

fn media_tool_path() -> PathBuf {
    if let Ok(configured) = env::var("APPLIEDYET_FFMPEG_PATH") {
        return PathBuf::from(configured);
    }
    if let Ok(executable) = env::current_exe() {
        if let Some(parent) = executable.parent() {
            let bundled = parent.join(if cfg!(windows) {
                "ffmpeg.exe"
            } else {
                "ffmpeg"
            });
            if bundled.is_file() {
                return bundled;
            }
        }
    }
    PathBuf::from("ffmpeg")
}

fn is_video_file(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .is_some_and(|extension| {
            matches!(
                extension.as_str(),
                "mp4" | "webm" | "mov" | "mkv" | "avi" | "m4v"
            )
        })
}

fn chunk_plan(duration: f64, segment_seconds: f64) -> Vec<(f64, f64, f64, f64)> {
    let total = (duration / segment_seconds).ceil().max(1.0) as usize;
    (0..total)
        .map(|index| {
            let nominal_start = index as f64 * segment_seconds;
            let nominal_end = ((index + 1) as f64 * segment_seconds).min(duration);
            let actual_start = if index == 0 {
                nominal_start
            } else {
                (nominal_start - AUDIO_OVERLAP_SECONDS).max(0.0)
            };
            let actual_end = if index + 1 == total {
                nominal_end
            } else {
                (nominal_end + AUDIO_OVERLAP_SECONDS).min(duration)
            };
            (
                actual_start,
                actual_end,
                nominal_start - actual_start,
                nominal_end - actual_start,
            )
        })
        .collect()
}

fn extract_chunk_transcript(
    value: &Value,
    chunk: &AudioChunk,
) -> Result<(String, Vec<Value>), String> {
    let mut texts = Vec::new();
    let mut adjusted = Vec::new();
    let mut saw_timed_segment = false;
    if let Some(segments) = value.get("segments").and_then(Value::as_array) {
        for segment in segments {
            let start = segment.get("start").and_then(Value::as_f64);
            let end = segment.get("end").and_then(Value::as_f64);
            if let (Some(start), Some(end)) = (start, end) {
                saw_timed_segment = true;
                let midpoint = (start + end) / 2.0;
                if midpoint < chunk.keep_start || midpoint >= chunk.keep_end {
                    continue;
                }
                if let Some(text) = segment.get("text").and_then(Value::as_str) {
                    let text = text.trim();
                    if !text.is_empty() {
                        texts.push(text.to_string());
                    }
                }
                let mut segment = segment.clone();
                segment["start"] = Value::from(start + chunk.actual_start);
                segment["end"] = Value::from(end + chunk.actual_start);
                adjusted.push(segment);
            }
        }
    }
    if !saw_timed_segment {
        let text = required_transcript_text(value)?;
        return Ok((text, adjusted));
    }
    Ok((texts.join("\n"), adjusted))
}

fn merge_transcript_parts(parts: &[String]) -> String {
    let mut merged = String::new();
    for part in parts {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if merged.is_empty() {
            merged.push_str(part);
            continue;
        }
        let overlap = exact_text_overlap(&merged, part, 240);
        if overlap == 0 && !merged.ends_with('\n') {
            merged.push('\n');
        }
        merged.extend(part.chars().skip(overlap));
    }
    merged.trim().to_string()
}

fn exact_text_overlap(left: &str, right: &str, maximum: usize) -> usize {
    let left = left.chars().collect::<Vec<_>>();
    let right = right.chars().collect::<Vec<_>>();
    let maximum = maximum.min(left.len()).min(right.len());
    (4..=maximum)
        .rev()
        .find(|length| left[left.len() - length..] == right[..*length])
        .unwrap_or(0)
}

fn required_transcript_text(value: &Value) -> Result<String, String> {
    extract_transcript_text(value)
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
        .ok_or_else(|| "语音识别完成，但服务没有返回可用文字".to_string())
}

fn ensure_transcript_length(transcript: &str) -> Result<(), String> {
    if transcript.chars().count() > crate::MAX_INTERVIEW_MATERIAL_CHARACTERS {
        Err("音频转写超过 6 万字限制，请拆分录音后重试".into())
    } else {
        Ok(())
    }
}

fn effective_upload_limit_bytes(settings: &AsrProviderSettings) -> Result<u64, String> {
    let configured = u64::try_from(settings.file_limit_mb)
        .ok()
        .filter(|value| (1..=2048).contains(value))
        .ok_or_else(|| "ASR 单片上传限制无效，请重新保存设置".to_string())?;
    let provider_limit = if provider_host(settings).as_deref() == Some("api.groq.com") {
        24
    } else {
        configured
    };
    Ok(configured.min(provider_limit) * 1024 * 1024)
}

fn safe_segment_seconds(configured: i64, upload_limit: u64) -> i64 {
    let size_limited =
        ((upload_limit as f64 * 0.8 * 8.0) / MP3_BITS_PER_SECOND as f64).floor() as i64;
    configured.min(size_limited).clamp(30, 1800)
}

fn supports_verbose_transcript(settings: &AsrProviderSettings) -> bool {
    provider_host(settings).as_deref() == Some("api.groq.com")
        || settings.model.to_ascii_lowercase().contains("whisper")
}

fn provider_host(settings: &AsrProviderSettings) -> Option<String> {
    url::Url::parse(settings.base_url.trim())
        .ok()
        .and_then(|value| value.host_str().map(str::to_ascii_lowercase))
}

fn upload_too_large(error: &str) -> bool {
    let error = error.to_ascii_lowercase();
    error.contains("413")
        || error.contains("too large")
        || error.contains("request body")
        || error.contains("reduce the size")
        || error.contains("文件过大")
}

fn progress_value(phase: &str, completed: usize, total: usize, message: &str) -> Value {
    json!({
        "progress": {
            "phase": phase,
            "completed": completed,
            "total": total,
            "message": message
        }
    })
}

fn completed_progress(total: usize) -> Value {
    json!({
        "phase": "completed",
        "completed": total,
        "total": total,
        "message": "转写完成"
    })
}

fn extract_transcript_text(value: &Value) -> Option<String> {
    let segments = value.get("segments").and_then(Value::as_array);
    let has_speakers = segments.is_some_and(|segments| {
        segments.iter().any(|segment| {
            segment
                .get("speaker")
                .is_some_and(|speaker| !speaker.is_null())
        })
    });
    if has_speakers {
        let lines = segments?
            .iter()
            .filter_map(|segment| {
                let text = segment.get("text").and_then(Value::as_str)?.trim();
                if text.is_empty() {
                    return None;
                }
                let speaker = segment.get("speaker").and_then(|speaker| match speaker {
                    Value::String(value) => Some(value.clone()),
                    Value::Number(value) => Some(value.to_string()),
                    _ => None,
                });
                Some(match speaker {
                    Some(speaker) => format!("[说话人 {speaker}] {text}"),
                    None => text.to_string(),
                })
            })
            .collect::<Vec<_>>()
            .join("\n");
        if !lines.is_empty() {
            return Some(lines);
        }
    }
    value
        .get("text")
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            segments.map(|segments| {
                segments
                    .iter()
                    .filter_map(|segment| segment.get("text").and_then(Value::as_str))
                    .collect::<Vec<_>>()
                    .join("\n")
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_plan_adds_overlap_without_changing_keep_windows() {
        let plan = chunk_plan(620.0, 300.0);
        assert_eq!(plan.len(), 3);
        assert_eq!(plan[0], (0.0, 301.0, 0.0, 300.0));
        assert_eq!(plan[1], (299.0, 601.0, 1.0, 301.0));
        assert_eq!(plan[2], (599.0, 620.0, 1.0, 21.0));
    }

    #[test]
    fn chunk_transcript_discards_overlap_and_offsets_timestamps() {
        let value = json!({
            "segments": [
                {"start": 0.1, "end": 0.8, "text": "上一段重复"},
                {"start": 1.1, "end": 2.0, "text": "本段正文"}
            ]
        });
        let chunk = AudioChunk {
            path: PathBuf::from("unused.mp3"),
            actual_start: 299.0,
            keep_start: 1.0,
            keep_end: 301.0,
        };
        let (text, segments) = extract_chunk_transcript(&value, &chunk).unwrap();
        assert_eq!(text, "本段正文");
        assert_eq!(segments[0]["start"], 300.1);
        assert_eq!(segments[0]["end"], 301.0);
    }

    #[test]
    fn chunk_transcript_does_not_restore_text_when_all_segments_are_overlap() {
        let value = json!({
            "text": "上一段重复",
            "segments": [{"start": 0.1, "end": 0.8, "text": "上一段重复"}]
        });
        let chunk = AudioChunk {
            path: PathBuf::from("unused.mp3"),
            actual_start: 299.0,
            keep_start: 1.0,
            keep_end: 301.0,
        };
        let (text, segments) = extract_chunk_transcript(&value, &chunk).unwrap();
        assert!(text.is_empty());
        assert!(segments.is_empty());
    }

    #[test]
    fn fallback_merge_removes_exact_chinese_overlap() {
        let merged = merge_transcript_parts(&[
            "面试官问你为什么选择我们公司".into(),
            "为什么选择我们公司候选人开始回答".into(),
        ]);
        assert_eq!(merged, "面试官问你为什么选择我们公司候选人开始回答");
    }

    #[test]
    fn fallback_merge_removes_four_character_chinese_overlap() {
        let merged =
            merge_transcript_parts(&["上一段最后我们公司".into(), "我们公司今天开始面试".into()]);
        assert_eq!(merged, "上一段最后我们公司今天开始面试");
    }

    #[test]
    fn transcription_retry_policy_rejects_permanent_client_errors() {
        assert!(!transcription_status_is_retryable(
            reqwest::StatusCode::UNAUTHORIZED
        ));
        assert!(!transcription_status_is_retryable(
            reqwest::StatusCode::FORBIDDEN
        ));
        assert!(!transcription_status_is_retryable(
            reqwest::StatusCode::NOT_FOUND
        ));
        assert!(transcription_status_is_retryable(
            reqwest::StatusCode::TOO_MANY_REQUESTS
        ));
        assert!(transcription_status_is_retryable(
            reqwest::StatusCode::BAD_GATEWAY
        ));
    }

    #[test]
    fn groq_upload_limit_keeps_safety_margin() {
        let settings = AsrProviderSettings {
            base_url: "https://api.groq.com/openai/v1".into(),
            file_limit_mb: 500,
            ..AsrProviderSettings::default()
        };
        assert_eq!(
            effective_upload_limit_bytes(&settings).unwrap(),
            24 * 1024 * 1024
        );
    }

    #[test]
    fn parses_ffmpeg_duration_and_recognizes_video_extensions() {
        assert_eq!(
            parse_ffmpeg_duration("  Duration: 01:02:03.50, start: 0.000000"),
            Some(3723.5)
        );
        assert!(is_video_file(Path::new("interview.MKV")));
        assert!(is_video_file(Path::new("screen-recording.mp4")));
        assert!(!is_video_file(Path::new("interview.m4a")));
    }

    #[test]
    fn diarized_transcript_keeps_speaker_labels() {
        let value = json!({
            "text": "请介绍你自己。我是一名开发者。",
            "segments": [
                {"speaker": "A", "text": "请介绍你自己。"},
                {"speaker": "B", "text": "我是一名开发者。"}
            ]
        });
        assert_eq!(
            extract_transcript_text(&value).as_deref(),
            Some("[说话人 A] 请介绍你自己。\n[说话人 B] 我是一名开发者。")
        );
    }

    #[test]
    fn plain_transcript_still_prefers_top_level_text() {
        let value = json!({
            "text": "完整转写",
            "segments": [{"text": "片段一"}, {"text": "片段二"}]
        });
        assert_eq!(extract_transcript_text(&value).as_deref(), Some("完整转写"));
    }

    #[test]
    fn ffmpeg_preprocessing_creates_small_playable_chunks_when_available() {
        if Command::new(media_tool_path())
            .arg("-version")
            .output()
            .is_err()
        {
            return;
        }
        let root = env::temp_dir()
            .join("appliedyet-asr-test")
            .join(uuid::Uuid::new_v4().to_string());
        fs::create_dir_all(&root).unwrap();
        let source = root.join("source.wav");
        let generated = Command::new(media_tool_path())
            .args([
                "-hide_banner",
                "-loglevel",
                "error",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=1000:duration=12",
                "-ar",
                "16000",
                "-ac",
                "1",
                "-y",
            ])
            .arg(&source)
            .status()
            .unwrap();
        assert!(generated.success());
        let prepared = prepare_audio_chunks(&source, 5, true).unwrap();
        assert_eq!(prepared.chunks.len(), 3);
        assert!(prepared
            .chunks
            .iter()
            .all(|chunk| fs::metadata(&chunk.path).is_ok_and(|value| value.len() > 0)));
        drop(prepared);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn ffmpeg_preprocessing_extracts_and_mixes_video_audio_tracks_when_available() {
        if Command::new(media_tool_path())
            .arg("-version")
            .output()
            .is_err()
        {
            return;
        }
        let root = env::temp_dir()
            .join("appliedyet-video-test")
            .join(uuid::Uuid::new_v4().to_string());
        fs::create_dir_all(&root).unwrap();
        let source = root.join("screen-recording.mkv");
        let generated = Command::new(media_tool_path())
            .args([
                "-hide_banner",
                "-loglevel",
                "error",
                "-f",
                "lavfi",
                "-i",
                "color=c=black:s=160x90:r=1:d=8",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:duration=8",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=880:duration=8",
                "-map",
                "0:v:0",
                "-map",
                "1:a:0",
                "-map",
                "2:a:0",
                "-c:v",
                "mpeg4",
                "-c:a",
                "pcm_s16le",
                "-y",
            ])
            .arg(&source)
            .status()
            .unwrap();
        assert!(generated.success());
        let source_info = probe_audio_info(&source).unwrap();
        assert_eq!(source_info.audio_streams, 2);
        let prepared = prepare_audio_chunks(&source, 5, true).unwrap();
        assert_eq!(prepared.chunks.len(), 2);
        assert_eq!(
            probe_audio_info(&prepared.chunks[0].path)
                .unwrap()
                .audio_streams,
            1
        );
        drop(prepared);
        let _ = fs::remove_dir_all(root);
    }
}
