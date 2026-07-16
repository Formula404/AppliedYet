#[tauri::command]
pub(crate) fn get_system_proxy() -> Option<String> {
    system_proxy()
}

#[cfg(target_os = "windows")]
fn system_proxy() -> Option<String> {
    use winreg::{enums::HKEY_CURRENT_USER, RegKey};

    let internet_settings = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings")
        .ok()?;
    let enabled = internet_settings.get_value::<u32, _>("ProxyEnable").ok()?;
    if enabled == 0 {
        return None;
    }
    let proxy_server = internet_settings
        .get_value::<String, _>("ProxyServer")
        .ok()?;
    normalize_proxy_server(&proxy_server)
}

#[cfg(not(target_os = "windows"))]
fn system_proxy() -> Option<String> {
    None
}

fn normalize_proxy_server(value: &str) -> Option<String> {
    let entries: Vec<_> = value
        .split(';')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .collect();
    let candidate = entries
        .iter()
        .find_map(|entry| entry.strip_prefix("https="))
        .or_else(|| entries.iter().find_map(|entry| entry.strip_prefix("http=")))
        .or_else(|| entries.iter().find(|entry| !entry.contains('=')).copied())?
        .trim();
    if candidate.is_empty() || candidate.chars().any(char::is_whitespace) {
        return None;
    }
    if candidate.starts_with("http://") || candidate.starts_with("https://") {
        Some(candidate.to_string())
    } else {
        Some(format!("http://{candidate}"))
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_proxy_server;

    #[test]
    fn normalizes_windows_proxy_server_formats() {
        assert_eq!(
            normalize_proxy_server("127.0.0.1:7890").as_deref(),
            Some("http://127.0.0.1:7890")
        );
        assert_eq!(
            normalize_proxy_server("http=127.0.0.1:8080;https=127.0.0.1:7890").as_deref(),
            Some("http://127.0.0.1:7890")
        );
        assert_eq!(normalize_proxy_server("socks=127.0.0.1:1080"), None);
    }
}
