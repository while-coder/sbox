use crate::common::random::random_device_id;
use base64::Engine;
use serde::Serialize;
use sha2::Digest;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::{Duration, Instant};

/// 内置 OAuth 桌面客户端。Desktop app 类型的 client secret 本身不是机密
/// （安全靠用户自己在浏览器登录自己的 Google 账号，refresh token 按用户隔离），
/// 可随程序分发一份共用；界面里填写自定义客户端时可覆盖这两个值。
///
/// 凭证不入库：构建时通过 `GDRIVE_BUILTIN_CLIENT_ID` / `GDRIVE_BUILTIN_CLIENT_SECRET`
/// 注入——本地开发写在 src-tauri/.env.local（已 gitignore，见 build.rs），
/// CI/发布直接设置同名环境变量。未注入时为空，此时须在界面里填写自定义客户端。
const BUILTIN_CLIENT_ID: &str = match option_env!("GDRIVE_BUILTIN_CLIENT_ID") {
    Some(v) => v,
    None => "",
};
const BUILTIN_CLIENT_SECRET: &str = match option_env!("GDRIVE_BUILTIN_CLIENT_SECRET") {
    Some(v) => v,
    None => "",
};

const SCOPE_DRIVE: &str = "https://www.googleapis.com/auth/drive";
const SCOPE_ANDROIDPUBLISHER: &str = "https://www.googleapis.com/auth/androidpublisher";
/// 允许请求的 scope 白名单，防止拼出意外的授权范围。
const ALLOWED_SCOPES: &[&str] = &[SCOPE_DRIVE, SCOPE_ANDROIDPUBLISHER];
const AUTH_ENDPOINT: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";

#[derive(Serialize, Clone)]
pub struct GdriveCreds {
    #[serde(rename = "clientId")]
    pub client_id: String,
    #[serde(rename = "clientSecret")]
    pub client_secret: String,
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
    pub scope: String,
    #[serde(rename = "accessToken")]
    pub access_token: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Google OAuth（桌面端 loopback 流程）：
//   1. 本地回环端口起一个一次性 HTTP listener 作为 redirect_uri；
//   2. 系统浏览器打开同意页（Google 拒绝嵌入式 webview，故用系统浏览器）；
//   3. 用户授权后 Google 重定向回 http://127.0.0.1:<port>/?code=...&state=...；
//   4. 用 code + client_secret 向 token 端点换取 refresh_token。
// 需使用 "Desktop app" 类型 OAuth 客户端（Google 自动放行任意端口的回环重定向）。
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command(rename_all = "camelCase")]
pub async fn gdrive_oauth_login(
    client_id: String,
    client_secret: String,
    scopes: Option<Vec<String>>,
) -> Result<GdriveCreds, String> {
    let client_id = client_id.trim().to_string();
    let client_secret = client_secret.trim().to_string();
    // 留空时回退到内置客户端；两者皆无才报错。
    let client_id = if client_id.is_empty() { BUILTIN_CLIENT_ID.to_string() } else { client_id };
    let client_secret = if client_secret.is_empty() { BUILTIN_CLIENT_SECRET.to_string() } else { client_secret };
    if client_id.is_empty() || client_secret.is_empty() {
        return Err("请填写 Client ID 与 Client Secret".into());
    }
    // 校验 scope 白名单并去重；未指定时默认 Drive 完整读写。
    let mut allowed: Vec<&str> = Vec::new();
    for requested in scopes.unwrap_or_default() {
        let requested = requested.trim();
        if requested.is_empty() {
            continue;
        }
        match ALLOWED_SCOPES.iter().find(|s| **s == requested) {
            Some(s) => {
                if !allowed.contains(s) {
                    allowed.push(s);
                }
            }
            None => return Err(format!("不支持的授权范围: {requested}")),
        }
    }
    if allowed.is_empty() {
        allowed.push(SCOPE_DRIVE);
    }
    let scope = allowed.join(" ");

    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| format!("绑定本地端口失败: {e}"))?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    listener
        .set_nonblocking(true)
        .map_err(|e| e.to_string())?;
    let redirect_uri = format!("http://127.0.0.1:{port}");
    let state = random_device_id();
    // PKCE S256：授权码只对本客户端 + 本 verifier 有效，防回环端口上的授权码拦截。
    let (code_verifier, code_challenge) = generate_pkce();

    let auth_url = format!(
        "{AUTH_ENDPOINT}?client_id={}&redirect_uri={}&response_type=code&scope={}&access_type=offline&prompt=consent&state={}&code_challenge={}&code_challenge_method=S256",
        urlencoding::encode(&client_id),
        urlencoding::encode(&redirect_uri),
        urlencoding::encode(&scope),
        urlencoding::encode(&state),
        urlencoding::encode(&code_challenge),
    );

    open::that(&auth_url).map_err(|e| format!("打开系统浏览器失败: {e}"))?;

    let expected_state = state.clone();
    let code = tokio::task::spawn_blocking(move || wait_for_code(listener, &expected_state))
        .await
        .map_err(|e| format!("内部任务错误: {e}"))??;

    let client = reqwest::Client::new();
    let params = [
        ("code", code.as_str()),
        ("client_id", client_id.as_str()),
        ("client_secret", client_secret.as_str()),
        ("redirect_uri", redirect_uri.as_str()),
        ("grant_type", "authorization_code"),
        ("code_verifier", code_verifier.as_str()),
    ];
    let resp = client
        .post(TOKEN_ENDPOINT)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("换取 token 失败: {e}"))?;
    let status = resp.status();
    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("解析 token 响应失败: {e}"))?;
    if !status.is_success() {
        let err = body
            .get("error_description")
            .or_else(|| body.get("error"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        return Err(format!("Google 返回错误: {err}"));
    }

    let refresh_token = body
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if refresh_token.is_empty() {
        return Err(
            "Google 未返回 refresh token（该账号可能此前已授权过本应用）。请到 https://myaccount.google.com/permissions 撤销授权后重试。"
                .into(),
        );
    }
    let access_token = body
        .get("access_token")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let granted_scope = body
        .get("scope")
        .and_then(|v| v.as_str())
        .unwrap_or(&scope)
        .to_string();

    Ok(GdriveCreds {
        client_id,
        client_secret,
        refresh_token,
        scope: granted_scope,
        access_token,
    })
}

/// 生成 PKCE 的 (code_verifier, code_challenge)：
/// verifier 为 96 个非保留字符；challenge = BASE64URL(SHA256(verifier))，无填充。
fn generate_pkce() -> (String, String) {
    use rand::Rng;

    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";
    let mut rng = rand::thread_rng();
    let verifier: String = (0..96)
        .map(|_| CHARS[rng.gen_range(0..CHARS.len())] as char)
        .collect();
    let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(sha2::Sha256::digest(verifier.as_bytes()));
    (verifier, challenge)
}

/// 非阻塞 accept 轮询，等待 Google 的授权回调；500ms 间隔，5 分钟超时。
fn wait_for_code(listener: TcpListener, expected_state: &str) -> Result<String, String> {
    let started = Instant::now();
    let timeout = Duration::from_secs(300);
    loop {
        if started.elapsed() > timeout {
            return Err("登录超时（5 分钟内未完成授权）".into());
        }
        match listener.accept() {
            Ok((mut stream, _)) => {
                let query = read_request_query(&mut stream);
                let params = parse_query(&query);

                if let Some(err) = params.get("error") {
                    respond_html(&mut stream, "授权被拒绝或失败，可关闭本页返回 sbox。");
                    return Err(format!("授权失败: {err}"));
                }
                match params.get("code") {
                    Some(code) => {
                        if params.get("state").map(String::as_str) != Some(expected_state) {
                            respond_html(&mut stream, "state 校验失败，请重试。");
                            return Err("state 校验失败（可能存在 CSRF 风险），请重试".into());
                        }
                        respond_html(&mut stream, "登录成功！已获取授权，可关闭本页返回 sbox。");
                        return Ok(code.clone());
                    }
                    None => {
                        // 非授权回调（如浏览器探测 /favicon.ico），忽略，继续等
                        respond_status(&mut stream, "404 Not Found");
                        continue;
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(200));
                continue;
            }
            Err(e) => return Err(format!("接收回调失败: {e}")),
        }
    }
}

/// 读取请求首行，取出 query 串（GET /path?<query> HTTP/1.1）。
fn read_request_query(stream: &mut TcpStream) -> String {
    let mut buf = [0u8; 4096];
    let n = stream.read(&mut buf).unwrap_or(0);
    let req = String::from_utf8_lossy(&buf[..n]);
    let first = req.lines().next().unwrap_or("");
    let path = first.split_whitespace().nth(1).unwrap_or("");
    path.split_once('?').map(|(_, q)| q.to_string()).unwrap_or_default()
}

fn parse_query(q: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for pair in q.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        let key = urlencoding::decode(k)
            .map(|c| c.into_owned())
            .unwrap_or_else(|_| k.to_string());
        let val = urlencoding::decode(v)
            .map(|c| c.into_owned())
            .unwrap_or_else(|_| v.to_string());
        map.insert(key, val);
    }
    map
}

fn respond_html(stream: &mut TcpStream, message: &str) {
    let html = format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>sbox</title></head>\
         <body style=\"font-family:sans-serif;text-align:center;padding-top:80px;color:#333\">\
         <h2>{message}</h2></body></html>"
    );
    let resp = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        html.as_bytes().len(),
        html
    );
    let _ = stream.write_all(resp.as_bytes());
    let _ = stream.flush();
}

fn respond_status(stream: &mut TcpStream, status: &str) {
    let resp = format!("HTTP/1.1 {status}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
    let _ = stream.write_all(resp.as_bytes());
    let _ = stream.flush();
}
