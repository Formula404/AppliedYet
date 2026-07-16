const SERVICE_NAME: &str = "com.appliedyet.desktop";
const ALLOWED_KEYS: &[&str] = &[
    "ai_api_key",
    "asr_api_key",
    "email_password",
    "email_oauth_refresh_token",
];

fn entry(key: &str) -> Result<keyring::Entry, String> {
    if !ALLOWED_KEYS.contains(&key) {
        return Err("不支持的凭据类型".to_string());
    }
    keyring::Entry::new(SERVICE_NAME, key).map_err(|error| format!("无法访问系统凭据库: {error}"))
}

pub fn set_secret(key: &str, secret: &str) -> Result<(), String> {
    let secret = secret.trim();
    if secret.is_empty() {
        return Err("凭据不能为空".to_string());
    }
    if secret.len() > 16 * 1024 {
        return Err("凭据长度超过 16 KB 限制".to_string());
    }
    entry(key)?
        .set_password(secret)
        .map_err(|error| format!("保存系统凭据失败: {error}"))
}

pub fn has_secret(key: &str) -> Result<bool, String> {
    match entry(key)?.get_password() {
        Ok(value) => Ok(!value.is_empty()),
        Err(keyring::Error::NoEntry) => Ok(false),
        Err(error) => Err(format!("读取系统凭据状态失败: {error}")),
    }
}

pub fn get_secret(key: &str) -> Result<String, String> {
    entry(key)?.get_password().map_err(|error| match error {
        keyring::Error::NoEntry => "尚未配置 API Key".to_string(),
        other => format!("读取系统凭据失败: {other}"),
    })
}

pub fn delete_secret(key: &str) -> Result<(), String> {
    match entry(key)?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(error) => Err(format!("删除系统凭据失败: {error}")),
    }
}
