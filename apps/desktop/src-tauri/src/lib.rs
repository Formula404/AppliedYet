use tauri::Manager;
use tauri_plugin_opener::OpenerExt;

mod ai;
mod asr;
mod commands;
mod credentials;
mod database;
use database as db;
mod document;
mod experience;
mod http;
mod resume;
mod resume_ai;

use db::Database;

#[tauri::command]
fn app_info() -> serde_json::Value {
    serde_json::json!({ "name": "投了吗", "version": env!("CARGO_PKG_VERSION"), "storage": "local-first" })
}

#[tauri::command]
fn open_external_url(app: tauri::AppHandle, url: String) -> Result<(), String> {
    let parsed = validate_external_url(&url)?;
    app.opener()
        .open_url(parsed.as_str(), None::<&str>)
        .map_err(|error| format!("无法打开系统浏览器: {error}"))
}

fn validate_external_url(url: &str) -> Result<url::Url, String> {
    if url.chars().count() > 2048 || url.chars().any(char::is_control) {
        return Err("外部链接无效或过长".into());
    }
    let parsed = url::Url::parse(url).map_err(|_| "外部链接格式无效".to_string())?;
    if !matches!(parsed.scheme(), "http" | "https" | "mailto")
        || (matches!(parsed.scheme(), "http" | "https")
            && (parsed.host_str().is_none()
                || !parsed.username().is_empty()
                || parsed.password().is_some()))
    {
        return Err("只允许打开 HTTP、HTTPS 或邮件链接".into());
    }
    Ok(parsed)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            app_info,
            open_external_url,
            commands::applications::list_applications,
            commands::applications::get_activity_summary,
            commands::applications::get_analytics,
            commands::applications::export_applications_excel,
            commands::applications::create_application,
            commands::applications::update_application_stage,
            commands::applications::get_application_detail,
            commands::applications::update_application_detail,
            commands::applications::create_application_task,
            commands::applications::set_application_task_status,
            commands::applications::create_application_event,
            commands::applications::get_dashboard,
            commands::applications::update_application_task,
            commands::applications::delete_application_task,
            commands::applications::set_application_archived,
            commands::applications::delete_archived_application,
            commands::applications::revert_application_event,
            commands::applications::update_application_event_time,
            commands::applications::list_due_task_reminders,
            commands::applications::mark_task_reminder_delivered,
            commands::applications::release_task_reminder_delivery,
            commands::settings::get_provider_settings,
            commands::settings::save_ai_provider_settings,
            commands::settings::save_asr_provider_settings,
            commands::settings::save_email_settings,
            commands::settings::get_data_location,
            commands::settings::set_data_location,
            commands::settings::backup_database,
            commands::settings::restore_database,
            commands::settings::credential_status,
            commands::settings::set_credential,
            commands::settings::delete_credential,
            commands::email::list_email_messages,
            commands::email::get_email_stats,
            commands::email::sync_emails,
            commands::email::authorize_email_oauth,
            commands::email::confirm_email_match,
            commands::email::ignore_email,
            commands::email::rematch_email,
            commands::experience::list_interview_experience_sources,
            commands::experience::import_interview_experience_link,
            commands::experience::create_manual_interview_experience,
            commands::experience::analyze_interview_experience_link,
            commands::experience::delete_interview_experience_source,
            commands::experience::update_interview_experience_questions,
            commands::interviews::list_interview_sessions,
            commands::interviews::create_mock_interview_session,
            commands::interviews::update_interview_session_answer,
            commands::interviews::update_interview_session_progress,
            commands::interviews::complete_interview_session,
            commands::interviews::generate_interview_review,
            commands::interviews::import_interview_transcript,
            commands::interviews::delete_interview_session,
            commands::interviews::list_question_bank_items,
            commands::interviews::save_question_bank_item,
            commands::interviews::delete_question_bank_item,
            commands::ai::test_ai_provider,
            commands::ai::generate_interview_preparation,
            commands::ai::get_latest_interview_preparation,
            commands::ai::list_application_ai_calls,
            commands::ai::generate_resume_questions,
            commands::processing::parse_document,
            commands::processing::transcribe_audio,
            commands::resumes::list_resume_profiles,
            commands::resumes::import_resume_profile,
            commands::resumes::update_resume_profile,
            commands::resumes::set_primary_resume_profile,
            commands::resumes::delete_resume_profile,
            commands::resumes::duplicate_resume_profile,
            commands::resumes::set_resume_profile_archived,
            commands::resumes::create_blank_resume_profile
        ])
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let pointer = data_dir.join("data-location.txt");
            let database_path = std::fs::read_to_string(pointer)
                .ok()
                .map(|value| std::path::PathBuf::from(value.trim()))
                .filter(|path| path.exists())
                .unwrap_or_else(|| data_dir.join("applied-yet.sqlite3"));
            let database = Database::open(&database_path).map_err(std::io::Error::other)?;
            app.manage(database);
            #[cfg(target_os = "windows")]
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_shadow(true);
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("启动投了吗失败");
}

#[cfg(test)]
mod tests {
    use super::validate_external_url;

    #[test]
    fn external_url_allowlist_rejects_unsafe_schemes_and_credentials() {
        assert!(validate_external_url("https://example.com/jobs").is_ok());
        assert!(validate_external_url("mailto:jobs@example.com").is_ok());
        assert!(validate_external_url("javascript:alert(1)").is_err());
        assert!(validate_external_url("https://user:secret@example.com").is_err());
    }
}
