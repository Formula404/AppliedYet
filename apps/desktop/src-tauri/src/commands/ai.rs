use crate::{
    ai,
    db::{AiCallSummary, Database, StoredInterviewPreparation},
};

#[tauri::command]
pub(crate) async fn test_ai_provider(
    db: tauri::State<'_, Database>,
) -> Result<ai::ProviderConnectionResult, String> {
    ai::test_connection(&db).await
}
#[tauri::command]
pub(crate) async fn generate_interview_preparation(
    db: tauri::State<'_, Database>,
    application_id: String,
) -> Result<StoredInterviewPreparation, String> {
    ai::generate_interview_preparation(&db, &application_id).await
}
#[tauri::command]
pub(crate) fn get_latest_interview_preparation(
    db: tauri::State<'_, Database>,
    application_id: String,
) -> Result<Option<StoredInterviewPreparation>, String> {
    db.latest_interview_preparation(&application_id)
}
#[tauri::command]
pub(crate) fn list_application_ai_calls(
    db: tauri::State<'_, Database>,
    application_id: String,
) -> Result<Vec<AiCallSummary>, String> {
    db.list_ai_calls(&application_id)
}

#[tauri::command]
pub(crate) async fn generate_resume_questions(
    db: tauri::State<'_, Database>,
    application_id: String,
    count: i64,
) -> Result<Vec<ai::PredictedQuestion>, String> {
    ai::generate_resume_questions(&db, &application_id, count).await
}
