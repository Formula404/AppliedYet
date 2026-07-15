use crate::{
    db::{Database, InterviewExperienceSource},
    experience,
};
use std::collections::HashSet;

#[tauri::command]
pub(crate) fn list_interview_experience_sources(
    db: tauri::State<'_, Database>,
    application_id: Option<String>,
) -> Result<Vec<InterviewExperienceSource>, String> {
    db.list_interview_experience_sources(application_id.as_deref())
}

#[tauri::command]
pub(crate) fn import_interview_experience_link(
    db: tauri::State<'_, Database>,
    application_id: String,
    url: String,
) -> Result<InterviewExperienceSource, String> {
    let url = url.trim();
    let parsed = url::Url::parse(url).map_err(|_| "请输入有效的网页地址".to_string())?;
    if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
        return Err("只支持 http:// 或 https:// 网页".into());
    }
    let host = parsed.host_str().unwrap_or("网页");
    db.create_interview_experience_link(&application_id, url, &format!("{host} · 面经帖子"))
}

#[tauri::command]
pub(crate) fn create_manual_interview_experience(
    db: tauri::State<'_, Database>,
    application_id: String,
    title: String,
    questions: Vec<String>,
) -> Result<InterviewExperienceSource, String> {
    let mut seen = HashSet::new();
    let questions: Vec<String> = questions
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty() && seen.insert(value.clone()))
        .take(100)
        .collect();
    if questions.is_empty() {
        return Err("请至少填写一道面试问题".into());
    }
    let title = title.trim();
    let title = if title.is_empty() {
        "人工整理面经"
    } else {
        title
    };
    db.create_manual_interview_experience(&application_id, title, &questions)
}

#[tauri::command]
pub(crate) async fn analyze_interview_experience_link(
    db: tauri::State<'_, Database>,
    id: String,
) -> Result<InterviewExperienceSource, String> {
    let source = db.get_interview_experience_source(&id)?;
    if source.source != "link" {
        return Err("人工录入的面经不需要网页分析".into());
    }
    let url = source
        .url
        .as_deref()
        .ok_or_else(|| "面经链接缺失".to_string())?;
    match experience::fetch_and_extract(url).await {
        Ok(result) if result.questions.is_empty() => db.update_interview_experience_analysis(
            &id,
            result.title.as_deref(),
            &[],
            Some("网页已读取，但没有识别到明确的面试问题；可改用人工录入。"),
        ),
        Ok(result) => db.update_interview_experience_analysis(
            &id,
            result.title.as_deref(),
            &result.questions,
            None,
        ),
        Err(error) => db.update_interview_experience_analysis(&id, None, &[], Some(&error)),
    }
}

#[tauri::command]
pub(crate) fn delete_interview_experience_source(
    db: tauri::State<'_, Database>,
    id: String,
) -> Result<(), String> {
    db.delete_interview_experience_source(&id)
}

#[tauri::command]
pub(crate) fn update_interview_experience_questions(
    db: tauri::State<'_, Database>,
    id: String,
    questions: Vec<String>,
) -> Result<InterviewExperienceSource, String> {
    let mut seen = HashSet::new();
    let questions: Vec<String> = questions
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty() && seen.insert(value.clone()))
        .take(100)
        .collect();
    db.update_interview_experience_questions(&id, &questions)
}
