use tauri::Manager;

mod ai;
mod asr;
mod commands;
mod credentials;
mod database;
use database as db;
mod document;
mod resume;
mod resume_ai;

use db::Database;

#[tauri::command]
fn app_info() -> serde_json::Value {
    serde_json::json!({ "name": "投了吗", "version": env!("CARGO_PKG_VERSION"), "storage": "local-first" })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            app_info,
            commands::applications::list_applications,
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
            commands::applications::revert_application_event,
            commands::applications::list_due_task_reminders,
            commands::applications::mark_task_reminder_delivered,
            commands::settings::get_provider_settings,
            commands::settings::save_ai_provider_settings,
            commands::settings::save_asr_provider_settings,
            commands::settings::credential_status,
            commands::settings::set_credential,
            commands::settings::delete_credential,
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
            let database = Database::open(&data_dir.join("applied-yet.sqlite3"))
                .map_err(std::io::Error::other)?;
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
