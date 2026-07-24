use crate::{
    ai, asr,
    db::{Database, InterviewSessionRecord, ProcessingJobResult, ProcessingJobSummary},
    document,
};

#[tauri::command]
pub(crate) async fn parse_document(
    db: tauri::State<'_, Database>,
    application_id: Option<String>,
    path: String,
) -> Result<ProcessingJobResult, String> {
    document::parse_document(&db, application_id.as_deref(), &path)
}
#[tauri::command]
pub(crate) async fn transcribe_audio(
    db: tauri::State<'_, Database>,
    application_id: Option<String>,
    path: String,
) -> Result<ProcessingJobResult, String> {
    asr::transcribe_audio(&db, application_id.as_deref(), &path).await
}

#[tauri::command]
pub(crate) fn list_processing_jobs(
    db: tauri::State<'_, Database>,
    limit: Option<i64>,
) -> Result<Vec<ProcessingJobSummary>, String> {
    db.list_processing_jobs(limit.unwrap_or(30))
}

#[tauri::command]
pub(crate) fn get_processing_job_text(
    db: tauri::State<'_, Database>,
    job_id: String,
) -> Result<String, String> {
    db.get_processing_job_text(&job_id)
}

#[tauri::command]
pub(crate) fn update_processing_job_text(
    db: tauri::State<'_, Database>,
    job_id: String,
    text: String,
) -> Result<(), String> {
    db.update_processing_job_text(&job_id, &text)
}

#[tauri::command]
pub(crate) fn delete_processing_job(
    db: tauri::State<'_, Database>,
    job_id: String,
) -> Result<(), String> {
    db.delete_processing_job(&job_id)
}

#[tauri::command]
pub(crate) async fn import_processing_job(
    db: tauri::State<'_, Database>,
    job_id: String,
    confirm_ai_send: bool,
) -> Result<InterviewSessionRecord, String> {
    let (application_id, transcript) = db.begin_processing_job_import(&job_id)?;
    match ai::import_interview_transcript(&db, &application_id, &transcript, confirm_ai_send).await
    {
        Ok(session) => {
            db.finish_processing_job_import(&job_id, Some(&session.id), None)?;
            Ok(session)
        }
        Err(error) => {
            match db.finish_processing_job_import(&job_id, None, Some(&error)) {
                Ok(()) => Err(error),
                Err(status_error) => Err(format!(
                    "{error}\n生成失败状态保存失败：{status_error}。请刷新记录后重试；若状态仍显示处理中，请重启应用以自动恢复。"
                )),
            }
        }
    }
}
