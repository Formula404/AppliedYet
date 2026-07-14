use crate::{
    asr,
    db::{Database, ProcessingJobResult},
    document,
};

#[tauri::command]
pub(crate) fn parse_document(
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
