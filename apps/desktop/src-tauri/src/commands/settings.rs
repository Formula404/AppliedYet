use crate::{
    credentials,
    db::{AiProviderSettings, AsrProviderSettings, Database, EmailSettings, ProviderSettings},
};
use std::{fs, path::Path};
use tauri::Manager;

#[tauri::command]
pub(crate) fn get_provider_settings(
    db: tauri::State<'_, Database>,
) -> Result<ProviderSettings, String> {
    db.get_provider_settings()
}
#[tauri::command]
pub(crate) fn save_ai_provider_settings(
    db: tauri::State<'_, Database>,
    settings: AiProviderSettings,
) -> Result<(), String> {
    db.save_ai_settings(settings)
}
#[tauri::command]
pub(crate) fn save_asr_provider_settings(
    db: tauri::State<'_, Database>,
    settings: AsrProviderSettings,
) -> Result<(), String> {
    db.save_asr_settings(settings)
}
#[tauri::command]
pub(crate) fn save_email_settings(
    db: tauri::State<'_, Database>,
    settings: EmailSettings,
) -> Result<(), String> {
    db.save_email_settings(settings)
}
#[tauri::command]
pub(crate) fn get_data_location(db: tauri::State<'_, Database>) -> Result<String, String> {
    db.storage_path()
}

#[tauri::command]
pub(crate) fn set_data_location(
    app: tauri::AppHandle,
    db: tauri::State<'_, Database>,
    directory: String,
) -> Result<String, String> {
    let path = db.relocate(Path::new(&directory))?;
    let default_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    fs::create_dir_all(&default_dir).map_err(|error| error.to_string())?;
    fs::write(default_dir.join("data-location.txt"), &path)
        .map_err(|error| format!("无法保存数据位置: {error}"))?;
    Ok(path)
}
#[tauri::command]
pub(crate) fn credential_status(key: String) -> Result<bool, String> {
    credentials::has_secret(&key)
}
#[tauri::command]
pub(crate) fn set_credential(key: String, secret: String) -> Result<(), String> {
    credentials::set_secret(&key, &secret)
}
#[tauri::command]
pub(crate) fn delete_credential(key: String) -> Result<(), String> {
    credentials::delete_secret(&key)
}
