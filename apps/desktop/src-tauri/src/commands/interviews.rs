use crate::{
    ai,
    db::{CreateInterviewQuestion, Database, InterviewSessionRecord, QuestionBankItem},
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SaveQuestionBankInput {
    prompt: String,
    category: String,
    best_answer: String,
    mastery: String,
}

#[tauri::command]
pub(crate) fn list_interview_sessions(
    db: tauri::State<'_, Database>,
) -> Result<Vec<InterviewSessionRecord>, String> {
    db.list_interview_sessions()
}

#[tauri::command]
pub(crate) fn create_mock_interview_session(
    db: tauri::State<'_, Database>,
    application_id: String,
    questions: Vec<CreateInterviewQuestion>,
) -> Result<InterviewSessionRecord, String> {
    db.create_interview_session(
        &application_id,
        "模拟面试",
        "技术综合模拟",
        "进行中",
        &questions,
    )
}

#[tauri::command]
pub(crate) fn update_interview_session_answer(
    db: tauri::State<'_, Database>,
    session_id: String,
    question_id: String,
    answer: String,
) -> Result<(), String> {
    db.update_interview_session_answer(&session_id, &question_id, &answer)
}

#[tauri::command]
pub(crate) fn update_interview_session_progress(
    db: tauri::State<'_, Database>,
    id: String,
    question_index: i64,
) -> Result<(), String> {
    db.update_interview_session_progress(&id, question_index)
}

#[tauri::command]
pub(crate) fn complete_interview_session(
    db: tauri::State<'_, Database>,
    id: String,
) -> Result<InterviewSessionRecord, String> {
    db.complete_interview_session(&id)
}

#[tauri::command]
pub(crate) async fn generate_interview_review(
    db: tauri::State<'_, Database>,
    id: String,
    confirm_ai_send: bool,
) -> Result<InterviewSessionRecord, String> {
    ai::generate_interview_review(&db, &id, confirm_ai_send).await
}

#[tauri::command]
pub(crate) async fn import_interview_transcript(
    db: tauri::State<'_, Database>,
    application_id: String,
    transcript: String,
    confirm_ai_send: bool,
) -> Result<InterviewSessionRecord, String> {
    ai::import_interview_transcript(&db, &application_id, &transcript, confirm_ai_send).await
}

#[tauri::command]
pub(crate) fn delete_interview_session(
    db: tauri::State<'_, Database>,
    id: String,
) -> Result<(), String> {
    db.delete_interview_session(&id)
}

#[tauri::command]
pub(crate) fn list_question_bank_items(
    db: tauri::State<'_, Database>,
) -> Result<Vec<QuestionBankItem>, String> {
    db.list_question_bank_items()
}

#[tauri::command]
pub(crate) fn save_question_bank_item(
    db: tauri::State<'_, Database>,
    id: Option<String>,
    input: SaveQuestionBankInput,
) -> Result<QuestionBankItem, String> {
    db.save_question_bank_item(
        id.as_deref(),
        &input.prompt,
        &input.category,
        &input.best_answer,
        &input.mastery,
    )
}

#[tauri::command]
pub(crate) fn delete_question_bank_item(
    db: tauri::State<'_, Database>,
    id: String,
) -> Result<(), String> {
    db.delete_question_bank_item(&id)
}
