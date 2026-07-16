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
    let default_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    fs::create_dir_all(&default_dir).map_err(|error| error.to_string())?;
    let pointer = default_dir.join("data-location.txt");
    db.relocate(Path::new(&directory), |target| {
        fs::write(&pointer, target.to_string_lossy().as_bytes())
            .map_err(|error| format!("无法保存数据位置: {error}"))
    })
}

#[tauri::command]
pub(crate) fn backup_database(
    db: tauri::State<'_, Database>,
    path: String,
) -> Result<String, String> {
    db.backup_to(Path::new(&path))
}

#[tauri::command]
pub(crate) fn restore_database(
    app: tauri::AppHandle,
    db: tauri::State<'_, Database>,
    path: String,
) -> Result<String, String> {
    let default_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    fs::create_dir_all(&default_dir).map_err(|error| error.to_string())?;
    let pointer = default_dir.join("data-location.txt");
    db.restore_from(Path::new(&path), |target| {
        fs::write(&pointer, target.to_string_lossy().as_bytes())
            .map_err(|error| format!("无法保存恢复后的数据位置: {error}"))
    })
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
