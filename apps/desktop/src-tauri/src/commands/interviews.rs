use crate::{
    ai,
    db::{
        CreateInterviewQuestion, Database, InterviewSessionRecord, ListQuestionBankInput,
        MergeQuestionInput, QuestionBankDetail, QuestionBankItem, QuestionBankPage,
        QuestionMatchCandidate, SplitQuestionInput,
    },
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SaveQuestionBankInput {
    prompt: String,
    category: String,
    best_answer: String,
    mastery: String,
    #[serde(default)]
    force_new: bool,
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
    input: ListQuestionBankInput,
) -> Result<QuestionBankPage, String> {
    db.list_question_bank_items(&input)
}

#[tauri::command]
pub(crate) fn get_question_bank_item(
    db: tauri::State<'_, Database>,
    id: String,
) -> Result<QuestionBankDetail, String> {
    db.get_question_bank_item(&id)
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
        input.force_new,
    )
}

#[tauri::command]
pub(crate) fn delete_question_bank_item(
    db: tauri::State<'_, Database>,
    id: String,
) -> Result<(), String> {
    db.delete_question_bank_item(&id)
}

#[tauri::command]
pub(crate) fn archive_question_bank_item(
    db: tauri::State<'_, Database>,
    id: String,
) -> Result<(), String> {
    db.archive_question_bank_item(&id)
}

#[tauri::command]
pub(crate) fn restore_question_bank_item(
    db: tauri::State<'_, Database>,
    id: String,
) -> Result<(), String> {
    db.restore_question_bank_item(&id)
}

#[tauri::command]
pub(crate) fn list_question_match_candidates(
    db: tauri::State<'_, Database>,
    prompt: String,
    exclude_id: Option<String>,
) -> Result<Vec<QuestionMatchCandidate>, String> {
    db.list_question_match_candidates(&prompt, exclude_id.as_deref())
}

#[tauri::command]
pub(crate) fn resolve_question_match(
    db: tauri::State<'_, Database>,
    left_id: String,
    right_id: String,
    action: String,
    reason: String,
) -> Result<Option<String>, String> {
    db.resolve_question_match(&left_id, &right_id, &action, &reason)
}

#[tauri::command]
pub(crate) fn merge_question_bank_items(
    db: tauri::State<'_, Database>,
    input: MergeQuestionInput,
) -> Result<String, String> {
    db.merge_question_bank_items(&input)
}

#[tauri::command]
pub(crate) fn split_question_bank_item(
    db: tauri::State<'_, Database>,
    input: SplitQuestionInput,
) -> Result<QuestionBankItem, String> {
    db.split_question_bank_item(&input)
}

#[tauri::command]
pub(crate) fn undo_question_merge(
    db: tauri::State<'_, Database>,
    audit_id: String,
) -> Result<(), String> {
    db.undo_question_merge(&audit_id)
}
