use tauri::Manager;

mod db;

use db::{
    ApplicationDetail, ApplicationEvent, ApplicationListItem, ApplicationTask,
    CreateApplicationInput, CreateEventInput, CreateTaskInput, Database,
    UpdateApplicationDetailInput,
};

#[tauri::command]
fn app_info() -> serde_json::Value {
    serde_json::json!({
        "name": "投了吗",
        "version": env!("CARGO_PKG_VERSION"),
        "storage": "local-first"
    })
}

#[tauri::command]
fn list_applications(
    database: tauri::State<'_, Database>,
) -> Result<Vec<ApplicationListItem>, String> {
    database.list_applications()
}

#[tauri::command]
fn create_application(
    database: tauri::State<'_, Database>,
    input: CreateApplicationInput,
) -> Result<ApplicationListItem, String> {
    database.create_application(input)
}

#[tauri::command]
fn update_application_stage(
    database: tauri::State<'_, Database>,
    id: String,
    stage: String,
) -> Result<(), String> {
    database.update_application_stage(&id, &stage)
}

#[tauri::command]
fn get_application_detail(
    database: tauri::State<'_, Database>,
    id: String,
) -> Result<ApplicationDetail, String> {
    database.get_application_detail(&id)
}

#[tauri::command]
fn update_application_detail(
    database: tauri::State<'_, Database>,
    id: String,
    input: UpdateApplicationDetailInput,
) -> Result<ApplicationDetail, String> {
    database.update_application_detail(&id, input)
}

#[tauri::command]
fn create_application_task(
    database: tauri::State<'_, Database>,
    application_id: String,
    input: CreateTaskInput,
) -> Result<ApplicationTask, String> {
    database.create_task(&application_id, input)
}

#[tauri::command]
fn set_application_task_status(
    database: tauri::State<'_, Database>,
    task_id: String,
    status: String,
) -> Result<ApplicationTask, String> {
    database.set_task_status(&task_id, &status)
}

#[tauri::command]
fn create_application_event(
    database: tauri::State<'_, Database>,
    application_id: String,
    input: CreateEventInput,
) -> Result<ApplicationEvent, String> {
    database.create_event(&application_id, input)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            app_info,
            list_applications,
            create_application,
            update_application_stage,
            get_application_detail,
            update_application_detail,
            create_application_task,
            set_application_task_status,
            create_application_event
        ])
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let database = Database::open(&data_dir.join("applied-yet.sqlite3"))
                .map_err(std::io::Error::other)?;
            app.manage(database);

            #[cfg(target_os = "windows")]
            if let Some(wv) = app.get_webview_window("main") {
                let _ = wv.set_shadow(true);
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("启动投了吗失败");
}
