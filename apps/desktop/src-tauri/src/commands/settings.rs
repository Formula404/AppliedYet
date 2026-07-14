use crate::{
    credentials,
    db::{AiProviderSettings, AsrProviderSettings, Database, ProviderSettings},
};

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
