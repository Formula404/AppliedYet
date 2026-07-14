use tauri::Manager;

#[tauri::command]
fn app_info() -> serde_json::Value {
    serde_json::json!({
        "name": "投了吗",
        "version": env!("CARGO_PKG_VERSION"),
        "storage": "local-first"
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![app_info])
        .setup(|app| {
            #[cfg(target_os = "windows")]
            if let Some(wv) = app.get_webview_window("main") {
                let _ = wv.set_shadow(true);
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("启动投了吗失败");
}
