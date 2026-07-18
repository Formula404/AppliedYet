use crate::{
    credentials,
    db::{Database, EmailLink, EmailMessage, EmailStats, RawEmail, SyncResult},
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, Utc};
use mailparse::MailHeaderMap;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    panic::{catch_unwind, AssertUnwindSafe},
    time::{Duration, Instant},
};
use tauri_plugin_opener::OpenerExt;
use url::Url;

enum EmailAuth {
    Password(String),
    OAuth2(String),
}

struct FetchBatch {
    messages: Vec<RawEmail>,
    highest_uid: Option<u32>,
    scanned: usize,
}

struct XOAuth2 {
    username: String,
    access_token: String,
}
impl imap::Authenticator for XOAuth2 {
    type Response = String;
    fn process(&self, _challenge: &[u8]) -> Self::Response {
        format!(
            "user={}\u{1}auth=Bearer {}\u{1}\u{1}",
            self.username, self.access_token
        )
    }
}

#[derive(Deserialize)]
struct OAuthTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
}

#[tauri::command]
pub(crate) fn list_email_messages(
    db: tauri::State<'_, Database>,
) -> Result<Vec<EmailMessage>, String> {
    db.list_email_messages()
}

#[tauri::command]
pub(crate) fn get_email_stats(db: tauri::State<'_, Database>) -> Result<EmailStats, String> {
    db.email_stats()
}

#[tauri::command]
pub(crate) async fn sync_emails(db: tauri::State<'_, Database>) -> Result<SyncResult, String> {
    let settings = db.get_provider_settings()?.email;
    let accounts = settings.active_accounts();
    if accounts.is_empty() {
        return Err("请先在设置中添加并启用至少一个邮箱".into());
    }
    let mut total = SyncResult {
        fetched: 0,
        recognized: 0,
        matched: 0,
    };
    let mut errors = Vec::new();
    for account_settings in accounts {
        let account_label = if account_settings.name.trim().is_empty() {
            account_settings.email_address.clone()
        } else {
            account_settings.name.clone()
        };
        match sync_email_account(&db, account_settings).await {
            Ok(result) => {
                total.fetched += result.fetched;
                total.recognized += result.recognized;
                total.matched += result.matched;
            }
            Err(error) => errors.push(format!("{account_label}：{error}")),
        }
    }
    if !errors.is_empty() {
        return Err(format!(
            "部分邮箱检查失败（已读取 {} 封）：{}",
            total.fetched,
            errors.join("；")
        ));
    }
    Ok(total)
}

async fn sync_email_account(
    db: &Database,
    settings: crate::db::EmailAccountSettings,
) -> Result<SyncResult, String> {
    let credential_suffix = if settings.id == "legacy" {
        None
    } else {
        Some(settings.id.as_str())
    };
    let auth = match settings.auth_method.as_str() {
        "oauth2" => {
            if settings.oauth_client_id.trim().is_empty() {
                return Err("使用 OAuth2 前请填写桌面应用 Client ID".into());
            }
            let key = credential_suffix
                .map(|id| format!("email_oauth_refresh_token:{id}"))
                .unwrap_or_else(|| "email_oauth_refresh_token".into());
            let refresh_token = credentials::get_secret(&key)
                .map_err(|_| "请先在邮箱设置中完成 OAuth2 授权".to_string())?;
            EmailAuth::OAuth2(refresh_access_token(&settings, &refresh_token, &key).await?)
        }
        "password" => {
            let key = credential_suffix
                .map(|id| format!("email_password:{id}"))
                .unwrap_or_else(|| "email_password".into());
            EmailAuth::Password(
                credentials::get_secret(&key)
                    .map_err(|_| "请先在设置中保存邮箱授权码或密码".to_string())?,
            )
        }
        _ => return Err("邮箱认证方式无效".into()),
    };
    let account = if settings.email_address.trim().is_empty() {
        settings.username.clone()
    } else {
        settings.email_address.clone()
    };
    let last_uid = db.latest_email_uid(&account)?;
    let batch = tauri::async_runtime::spawn_blocking(move || {
        catch_unwind(AssertUnwindSafe(|| fetch_imap(settings, auth, last_uid)))
            .map_err(|_| "IMAP 客户端处理服务器响应时异常退出，请检查服务商兼容性".to_string())?
    })
    .await
    .map_err(|error| format!("邮件检查任务失败: {error}"))??;
    db.ingest_emails_through(
        batch
            .messages
            .into_iter()
            .map(|mut item| {
                item.account = account.clone();
                item
            })
            .collect(),
        &account,
        batch.highest_uid,
        batch.scanned,
    )
}

#[tauri::command]
pub(crate) async fn authorize_email_oauth(
    app: tauri::AppHandle,
    account: crate::db::EmailAccountSettings,
) -> Result<(), String> {
    let settings = account;
    if settings.oauth_client_id.trim().is_empty() {
        return Err("请填写 OAuth2 桌面应用 Client ID".into());
    }
    oauth_endpoints(&settings)?;
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .map_err(|error| format!("无法启动 OAuth2 本机回调: {error}"))?;
    let port = listener
        .local_addr()
        .map_err(|error| error.to_string())?
        .port();
    let redirect_uri = format!("http://localhost:{port}");
    let verifier = format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    );
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    let state = uuid::Uuid::new_v4().to_string();
    let authorization_url = build_authorization_url(&settings, &redirect_uri, &state, &challenge)?;
    app.opener()
        .open_url(authorization_url, None::<&str>)
        .map_err(|error| format!("无法打开系统浏览器: {error}"))?;
    let expected_state = state.clone();
    let callback = tauri::async_runtime::spawn_blocking(move || {
        wait_for_oauth_callback(listener, &expected_state)
    })
    .await
    .map_err(|error| format!("OAuth2 回调任务失败: {error}"))??;
    let token = exchange_authorization_code(&settings, &callback, &redirect_uri, &verifier).await?;
    let credential_key = if settings.id == "legacy" {
        "email_oauth_refresh_token".to_string()
    } else {
        format!("email_oauth_refresh_token:{}", settings.id)
    };
    if let Some(refresh_token) = token.refresh_token {
        credentials::set_secret(&credential_key, &refresh_token)?;
    } else if !credentials::has_secret(&credential_key)? {
        return Err("服务商未返回刷新令牌，请撤销应用授权后重试".into());
    }
    Ok(())
}

#[tauri::command]
pub(crate) fn confirm_email_match(
    db: tauri::State<'_, Database>,
    id: String,
) -> Result<(), String> {
    db.confirm_email_match(&id)
}

#[tauri::command]
pub(crate) fn ignore_email(db: tauri::State<'_, Database>, id: String) -> Result<(), String> {
    db.ignore_email(&id)
}

#[tauri::command]
pub(crate) fn rematch_email(db: tauri::State<'_, Database>, id: String) -> Result<(), String> {
    db.rematch_email(&id)
}

#[tauri::command]
pub(crate) fn attach_email_to_application(
    db: tauri::State<'_, Database>,
    email_id: String,
    application_id: String,
) -> Result<(), String> {
    db.attach_email_to_application(&email_id, &application_id)
}

#[tauri::command]
pub(crate) fn create_application_from_email(
    db: tauri::State<'_, Database>,
    email_id: String,
    input: crate::db::CreateApplicationInput,
) -> Result<crate::db::ApplicationListItem, String> {
    db.create_application_from_email(&email_id, input)
}

fn fetch_imap(
    settings: crate::db::EmailAccountSettings,
    auth: EmailAuth,
    last_uid: u32,
) -> Result<FetchBatch, String> {
    let host = settings.imap_host.trim().to_string();
    let username = settings.username.trim().to_string();
    let port = u16::try_from(settings.imap_port).map_err(|_| "IMAP 端口无效".to_string())?;
    if settings.use_tls {
        let tls = native_tls::TlsConnector::builder()
            .build()
            .map_err(|error| format!("无法初始化 TLS: {error}"))?;
        if requires_imap_id(&host) {
            let tcp = TcpStream::connect((host.as_str(), port))
                .map_err(|error| format!("无法连接 IMAP 服务器: {error}"))?;
            tcp.set_read_timeout(Some(Duration::from_secs(30))).ok();
            tcp.set_write_timeout(Some(Duration::from_secs(30))).ok();
            let mut stream = tls
                .connect(&host, tcp)
                .map_err(|error| format!("IMAP TLS 握手失败: {error}"))?;
            send_pre_auth_client_id(&mut stream)?;
            let mut client = imap::Client::new(PreloadedStream::with_greeting(stream));
            client
                .read_greeting()
                .map_err(|error| format!("初始化 IMAP 会话失败: {error}"))?;
            let mut session = authenticate(client, &username, auth)?;
            let result = fetch_session(&mut session, &settings.email_address, last_uid);
            let _ = session.logout();
            result
        } else {
            let client = imap::connect((host.as_str(), port), &host, &tls)
                .map_err(|error| format!("无法连接 IMAP 服务器: {error}"))?;
            let mut session = authenticate(client, &username, auth)?;
            let result = fetch_session(&mut session, &settings.email_address, last_uid);
            let _ = session.logout();
            result
        }
    } else {
        let mut stream = TcpStream::connect((host.as_str(), port))
            .map_err(|error| format!("无法连接 IMAP 服务器: {error}"))?;
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(30)))
            .ok();
        stream
            .set_write_timeout(Some(std::time::Duration::from_secs(30)))
            .ok();
        if requires_imap_id(&host) {
            send_pre_auth_client_id(&mut stream)?;
            let mut client = imap::Client::new(PreloadedStream::with_greeting(stream));
            client
                .read_greeting()
                .map_err(|error| format!("初始化 IMAP 会话失败: {error}"))?;
            let mut session = authenticate(client, &username, auth)?;
            let result = fetch_session(&mut session, &settings.email_address, last_uid);
            let _ = session.logout();
            result
        } else {
            let mut client = imap::Client::new(stream);
            client
                .read_greeting()
                .map_err(|error| format!("读取 IMAP 欢迎消息失败: {error}"))?;
            let mut session = authenticate(client, &username, auth)?;
            let result = fetch_session(&mut session, &settings.email_address, last_uid);
            let _ = session.logout();
            result
        }
    }
}

struct PreloadedStream<T> {
    prefix: &'static [u8],
    offset: usize,
    inner: T,
}
impl<T> PreloadedStream<T> {
    fn with_greeting(inner: T) -> Self {
        Self {
            prefix: b"* OK IMAP server ready\r\n",
            offset: 0,
            inner,
        }
    }
}
impl<T: Read> Read for PreloadedStream<T> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        if self.offset < self.prefix.len() {
            let length = buffer.len().min(self.prefix.len() - self.offset);
            buffer[..length].copy_from_slice(&self.prefix[self.offset..self.offset + length]);
            self.offset += length;
            return Ok(length);
        }
        self.inner.read(buffer)
    }
}
impl<T: Write> Write for PreloadedStream<T> {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        self.inner.write(buffer)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

fn send_pre_auth_client_id<T: Read + Write>(stream: &mut T) -> Result<(), String> {
    let greeting = read_imap_line(stream)?;
    if !greeting.to_ascii_uppercase().starts_with("* OK") {
        return Err(format!("IMAP 服务器欢迎消息异常: {}", greeting.trim()));
    }
    stream.write_all(b"a0 ID (\"name\" \"AppliedYet\" \"version\" \"0.1.0\" \"vendor\" \"AppliedYet\" \"support-email\" \"support@appliedyet.local\")\r\n").map_err(|error| format!("发送 IMAP 客户端 ID 失败: {error}"))?;
    stream
        .flush()
        .map_err(|error| format!("发送 IMAP 客户端 ID 失败: {error}"))?;
    let mut total = 0usize;
    loop {
        let line = read_imap_line(stream)?;
        total += line.len();
        if total > 64 * 1024 {
            return Err("IMAP ID 响应过大".into());
        }
        if line.starts_with("a0 ") {
            if line.to_ascii_uppercase().starts_with("A0 OK") {
                return Ok(());
            }
            return Err(format!("网易邮箱拒绝客户端身份声明: {}", line.trim()));
        }
    }
}

fn read_imap_line<T: Read>(stream: &mut T) -> Result<String, String> {
    let mut data = Vec::new();
    let mut byte = [0u8; 1];
    while data.len() < 16 * 1024 {
        let read = stream
            .read(&mut byte)
            .map_err(|error| format!("读取 IMAP 响应失败: {error}"))?;
        if read == 0 {
            return Err("IMAP 服务器提前断开连接".into());
        }
        data.push(byte[0]);
        if byte[0] == b'\n' {
            return String::from_utf8(data).map_err(|_| "IMAP 响应不是有效 UTF-8".into());
        }
    }
    Err("IMAP 单行响应过长".into())
}

fn authenticate<T: Read + Write>(
    client: imap::Client<T>,
    username: &str,
    auth: EmailAuth,
) -> Result<imap::Session<T>, String> {
    match auth {
        EmailAuth::Password(password) => client
            .login(username, password)
            .map_err(|(error, _)| format!("邮箱登录失败，请检查用户名和授权码: {error}")),
        EmailAuth::OAuth2(access_token) => {
            let authenticator = XOAuth2 {
                username: username.to_string(),
                access_token,
            };
            client
                .authenticate("XOAUTH2", &authenticator)
                .map_err(|(error, _)| {
                    format!("邮箱 OAuth2 登录失败，授权可能已过期或缺少 IMAP 权限: {error}")
                })
        }
    }
}

fn oauth_endpoints(
    settings: &crate::db::EmailAccountSettings,
) -> Result<(String, String, String), String> {
    let provider = settings.provider.to_lowercase();
    if provider.contains("gmail") || settings.imap_host.eq_ignore_ascii_case("imap.gmail.com") {
        return Ok((
            "https://accounts.google.com/o/oauth2/v2/auth".into(),
            "https://oauth2.googleapis.com/token".into(),
            "https://mail.google.com/".into(),
        ));
    }
    if provider.contains("outlook")
        || settings
            .imap_host
            .eq_ignore_ascii_case("outlook.office365.com")
    {
        let tenant = settings.oauth_tenant.trim();
        if tenant.is_empty()
            || !tenant
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.')
        {
            return Err("Microsoft Tenant 值无效".into());
        }
        return Ok((
            format!("https://login.microsoftonline.com/{tenant}/oauth2/v2.0/authorize"),
            format!("https://login.microsoftonline.com/{tenant}/oauth2/v2.0/token"),
            "offline_access https://outlook.office.com/IMAP.AccessAsUser.All".into(),
        ));
    }
    Err("当前 OAuth2 连接器支持 Gmail 和 Outlook/Microsoft 365".into())
}

fn build_authorization_url(
    settings: &crate::db::EmailAccountSettings,
    redirect_uri: &str,
    state: &str,
    challenge: &str,
) -> Result<String, String> {
    let (authorization_endpoint, _, scope) = oauth_endpoints(settings)?;
    let mut url = Url::parse(&authorization_endpoint).map_err(|error| error.to_string())?;
    url.query_pairs_mut()
        .append_pair("client_id", settings.oauth_client_id.trim())
        .append_pair("redirect_uri", redirect_uri)
        .append_pair("response_type", "code")
        .append_pair("scope", &scope)
        .append_pair("state", state)
        .append_pair("code_challenge", challenge)
        .append_pair("code_challenge_method", "S256");
    if authorization_endpoint.contains("google.com") {
        url.query_pairs_mut()
            .append_pair("access_type", "offline")
            .append_pair("prompt", "consent");
    }
    Ok(url.into())
}

fn wait_for_oauth_callback(listener: TcpListener, expected_state: &str) -> Result<String, String> {
    listener
        .set_nonblocking(true)
        .map_err(|error| error.to_string())?;
    let deadline = Instant::now() + Duration::from_secs(180);
    loop {
        match listener.accept() {
            Ok((mut stream, _)) => {
                stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
                let mut buffer = [0u8; 8192];
                let length = stream
                    .read(&mut buffer)
                    .map_err(|error| format!("读取 OAuth2 回调失败: {error}"))?;
                let request = String::from_utf8_lossy(&buffer[..length]);
                match parse_oauth_callback(&request, expected_state) {
                    OAuthCallback::Ignore => {
                        let _ = reply_oauth(&mut stream, false);
                    }
                    OAuthCallback::Error(error) => {
                        let _ = reply_oauth(&mut stream, false);
                        return Err(error);
                    }
                    OAuthCallback::Code(code) => {
                        let _ = reply_oauth(&mut stream, true);
                        return Ok(code);
                    }
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                if Instant::now() >= deadline {
                    return Err("OAuth2 授权等待超时，请重试".into());
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(error) => return Err(format!("OAuth2 回调失败: {error}")),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum OAuthCallback {
    Ignore,
    Code(String),
    Error(String),
}

fn parse_oauth_callback(request: &str, expected_state: &str) -> OAuthCallback {
    let Some(path) = request.lines().next().and_then(|line| {
        let mut parts = line.split_whitespace();
        (parts.next() == Some("GET"))
            .then(|| parts.next())
            .flatten()
    }) else {
        return OAuthCallback::Ignore;
    };
    let Ok(url) = Url::parse(&format!("http://localhost{path}")) else {
        return OAuthCallback::Ignore;
    };
    let parameter = |name: &str| {
        url.query_pairs()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.into_owned())
    };
    if parameter("state").as_deref() != Some(expected_state) {
        return OAuthCallback::Ignore;
    }
    if let Some(error) = parameter("error") {
        OAuthCallback::Error(format!("OAuth2 授权被拒绝: {error}"))
    } else if let Some(code) = parameter("code") {
        OAuthCallback::Code(code)
    } else {
        OAuthCallback::Error("OAuth2 回调缺少授权码".into())
    }
}

fn reply_oauth(stream: &mut TcpStream, success: bool) -> std::io::Result<()> {
    let message = if success {
        "授权成功，可以关闭此窗口并返回“投了吗”。"
    } else {
        "授权未完成，请返回应用重试。"
    };
    let body = format!(
        "<!doctype html><meta charset=utf-8><title>投了吗 OAuth2</title><h2>{message}</h2>"
    );
    write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Security-Policy: default-src 'none'\r\nX-Content-Type-Options: nosniff\r\nCache-Control: no-store\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body)
}

async fn exchange_authorization_code(
    settings: &crate::db::EmailAccountSettings,
    code: &str,
    redirect_uri: &str,
    verifier: &str,
) -> Result<OAuthTokenResponse, String> {
    let (_, token_endpoint, _) = oauth_endpoints(settings)?;
    request_token(
        &token_endpoint,
        &[
            ("client_id", settings.oauth_client_id.trim()),
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri),
            ("code_verifier", verifier),
        ],
    )
    .await
}

async fn refresh_access_token(
    settings: &crate::db::EmailAccountSettings,
    refresh_token: &str,
    credential_key: &str,
) -> Result<String, String> {
    let (_, token_endpoint, scope) = oauth_endpoints(settings)?;
    let token = request_token(
        &token_endpoint,
        &[
            ("client_id", settings.oauth_client_id.trim()),
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("scope", &scope),
        ],
    )
    .await?;
    if let Some(next_refresh) = token.refresh_token.as_deref() {
        credentials::set_secret(credential_key, next_refresh)?;
    }
    Ok(token.access_token)
}

async fn request_token(
    endpoint: &str,
    form: &[(&str, &str)],
) -> Result<OAuthTokenResponse, String> {
    let response = reqwest::Client::new()
        .post(endpoint)
        .form(form)
        .send()
        .await
        .map_err(|error| format!("OAuth2 令牌请求失败: {error}"))?;
    let (status, body) = crate::http::read_response_bytes(response, 1024 * 1024, "OAuth2").await?;
    let body = String::from_utf8_lossy(&body);
    if !status.is_success() {
        return Err(format!(
            "OAuth2 服务返回错误 ({status}): {}",
            body.chars().take(400).collect::<String>()
        ));
    }
    serde_json::from_str(&body).map_err(|error| format!("无法解析 OAuth2 令牌响应: {error}"))
}

fn fetch_session<T: Read + Write>(
    session: &mut imap::Session<T>,
    account: &str,
    last_uid: u32,
) -> Result<FetchBatch, String> {
    const MAX_EMAIL_BYTES: u32 = 10 * 1024 * 1024;
    session.select("INBOX").map_err(|error| {
        let detail = error.to_string();
        if detail.to_ascii_lowercase().contains("unsafe login") {
            "网易邮箱仍将本次连接判定为不安全登录。请确认已开启 IMAP、使用客户端授权码而非网页登录密码，并在网页版或邮箱大师中完成第三方登录安全验证；若仍失败请联系 kefu@188.com。".to_string()
        } else { format!("无法打开收件箱: {detail}") }
    })?;
    let query = if last_uid == 0 {
        "ALL".to_string()
    } else {
        format!("UID {}:*", last_uid.saturating_add(1))
    };
    let found = session
        .uid_search(query)
        .map_err(|error| format!("无法查询新邮件: {error}"))?;
    let mut uids: Vec<u32> = found.into_iter().filter(|uid| *uid > last_uid).collect();
    uids.sort_unstable();
    if last_uid == 0 && uids.len() > 100 {
        uids = uids.split_off(uids.len() - 100);
    }
    let highest_uid = uids.last().copied();
    let mut messages = Vec::new();
    for uid in uids {
        let sizes = session
            .uid_fetch(uid.to_string(), "(UID RFC822.SIZE)")
            .map_err(|error| format!("读取邮件 {uid} 大小失败: {error}"))?;
        let declared_size = sizes
            .iter()
            .find(|item| item.uid == Some(uid))
            .or_else(|| sizes.iter().next())
            .and_then(|item| item.size);
        if declared_size.is_some_and(|size| size > MAX_EMAIL_BYTES) {
            continue;
        }
        let fetched = session
            .uid_fetch(uid.to_string(), "(UID RFC822)")
            .map_err(|error| format!("读取邮件 {uid} 失败: {error}"))?;
        let Some(fetch) = fetched
            .iter()
            .find(|item| item.uid == Some(uid))
            .or_else(|| fetched.iter().next())
        else {
            continue;
        };
        let Some(body) = fetch.body() else { continue };
        if body.len() > MAX_EMAIL_BYTES as usize {
            continue;
        }
        let Ok(parsed) = mailparse::parse_mail(body) else {
            continue;
        };
        let sender = parsed.headers.get_first_value("From").unwrap_or_default();
        let subject = parsed
            .headers
            .get_first_value("Subject")
            .unwrap_or_else(|| "（无主题）".into());
        let message_id = parsed.headers.get_first_value("Message-ID");
        let received_at = parsed
            .headers
            .get_first_value("Date")
            .and_then(|value| mailparse::dateparse(&value).ok())
            .and_then(|timestamp| DateTime::<Utc>::from_timestamp(timestamp, 0))
            .unwrap_or_else(Utc::now)
            .to_rfc3339();
        let mut body_text = plain_text(&parsed);
        let links = collect_links(&parsed);
        if body_text.chars().count() > 100_000 {
            body_text = body_text.chars().take(100_000).collect();
        }
        messages.push(RawEmail {
            account: account.into(),
            mailbox: "INBOX".into(),
            uid,
            message_id,
            sender,
            subject,
            received_at,
            body_text,
            links,
        });
    }
    let fetched = messages.len();
    Ok(FetchBatch {
        messages,
        highest_uid,
        scanned: fetched,
    })
}

fn requires_imap_id(host: &str) -> bool {
    let host = host.trim().to_ascii_lowercase();
    [
        "imap.163.com",
        "imap.126.com",
        "imap.188.com",
        "imap.yeah.net",
        "imap.vip.163.com",
        "imap.vip.126.com",
    ]
    .iter()
    .any(|candidate| host == *candidate)
}

#[cfg(test)]
fn is_local_imap_host(host: &str) -> bool {
    matches!(
        host.trim()
            .trim_end_matches('.')
            .to_ascii_lowercase()
            .as_str(),
        "localhost" | "127.0.0.1" | "::1"
    )
}

fn plain_text(mail: &mailparse::ParsedMail<'_>) -> String {
    if mail.ctype.mimetype.eq_ignore_ascii_case("text/plain") {
        return mail.get_body().unwrap_or_default();
    }
    for part in &mail.subparts {
        let text = plain_text(part);
        if !text.trim().is_empty() {
            return text;
        }
    }
    if mail.ctype.mimetype.eq_ignore_ascii_case("text/html") {
        let html = mail.get_body().unwrap_or_default();
        return strip_html(&html);
    }
    String::new()
}

fn strip_html(html: &str) -> String {
    let mut output = String::with_capacity(html.len());
    let mut in_tag = false;
    for character in html.chars() {
        match character {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                output.push(' ');
            }
            _ if !in_tag => output.push(character),
            _ => {}
        }
    }
    output
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

fn collect_links(mail: &mailparse::ParsedMail<'_>) -> Vec<EmailLink> {
    let mut links = Vec::new();
    collect_links_into(mail, &mut links);
    links.sort_by(|left, right| left.url.cmp(&right.url));
    links.dedup_by(|left, right| left.url == right.url);
    links.truncate(100);
    links
}

fn collect_links_into(mail: &mailparse::ParsedMail<'_>, links: &mut Vec<EmailLink>) {
    if mail.ctype.mimetype.eq_ignore_ascii_case("text/html") {
        if let Ok(html) = mail.get_body() {
            extract_html_links(&html, links);
        }
    }
    for part in &mail.subparts {
        collect_links_into(part, links);
    }
}

fn extract_html_links(html: &str, links: &mut Vec<EmailLink>) {
    let bytes = html.as_bytes();
    let mut index = 0usize;
    while index + 5 <= bytes.len() {
        if !bytes[index..index + 5].eq_ignore_ascii_case(b"href=") {
            index += 1;
            continue;
        }
        let mut start = index + 5;
        while start < bytes.len() && bytes[start].is_ascii_whitespace() {
            start += 1;
        }
        let quote = bytes.get(start).copied();
        let (value_start, terminator) = if matches!(quote, Some(b'\'' | b'\"')) {
            (start + 1, quote.unwrap())
        } else {
            (start, b' ')
        };
        let mut end = value_start;
        while end < bytes.len()
            && bytes[end] != terminator
            && (terminator != b' ' || !bytes[end].is_ascii_whitespace() && bytes[end] != b'>')
        {
            end += 1;
        }
        if let Ok(raw) = std::str::from_utf8(&bytes[value_start..end]) {
            let url = decode_html_entities(raw.trim());
            if is_safe_link(&url) {
                links.push(EmailLink {
                    label: format!("邮件链接 {}", links.len() + 1),
                    url,
                });
            }
        }
        index = end.saturating_add(1);
    }
}

fn decode_html_entities(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&#38;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

fn is_safe_link(value: &str) -> bool {
    Url::parse(value)
        .ok()
        .is_some_and(|url| matches!(url.scheme(), "http" | "https" | "mailto"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use imap::Authenticator;

    #[test]
    fn google_authorization_uses_pkce_and_offline_access() {
        let settings = crate::db::EmailAccountSettings {
            provider: "Gmail".into(),
            imap_host: "imap.gmail.com".into(),
            oauth_client_id: "client-id".into(),
            auth_method: "oauth2".into(),
            ..Default::default()
        };
        let url = build_authorization_url(
            &settings,
            "http://localhost:12345",
            "state-value",
            "challenge-value",
        )
        .unwrap();
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("access_type=offline"));
        assert!(url.contains("mail.google.com"));
    }

    #[test]
    fn xoauth2_payload_contains_user_and_bearer_token() {
        let auth = XOAuth2 {
            username: "me@example.com".into(),
            access_token: "token".into(),
        };
        assert_eq!(
            auth.process(&[]),
            "user=me@example.com\u{1}auth=Bearer token\u{1}\u{1}"
        );
    }

    #[test]
    fn netease_hosts_require_rfc2971_client_id() {
        assert!(requires_imap_id("imap.163.com"));
        assert!(requires_imap_id("IMAP.188.COM"));
        assert!(requires_imap_id("imap.yeah.net"));
        assert!(!requires_imap_id("imap.gmail.com"));
        assert!(is_local_imap_host("LOCALHOST."));
        assert!(is_local_imap_host("::1"));
        assert!(!is_local_imap_host("localhost.evil.example"));
    }

    #[test]
    fn extracts_safe_href_from_html_mail() {
        let parsed = mailparse::parse_mail(b"Content-Type: text/html; charset=utf-8\r\n\r\n<a href=\"https://example.com/test?a=1&amp;b=2\">start</a><a href=\"javascript:alert(1)\">bad</a>").unwrap();
        let links = collect_links(&parsed);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].url, "https://example.com/test?a=1&b=2");
    }

    #[test]
    fn oauth_callback_ignores_unrelated_local_requests() {
        assert_eq!(
            parse_oauth_callback("GET /favicon.ico HTTP/1.1\r\n\r\n", "expected"),
            OAuthCallback::Ignore
        );
        assert_eq!(
            parse_oauth_callback("GET /?code=attack&state=wrong HTTP/1.1\r\n\r\n", "expected"),
            OAuthCallback::Ignore
        );
        assert_eq!(
            parse_oauth_callback(
                "GET /?code=valid&state=expected HTTP/1.1\r\n\r\n",
                "expected"
            ),
            OAuthCallback::Code("valid".into())
        );
    }
}
