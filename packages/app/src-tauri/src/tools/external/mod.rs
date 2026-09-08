use url::Url;

/// 在系统默认浏览器中打开一个经过白名单校验的外部链接。
#[tauri::command]
pub fn open_external_url(url: String) -> Result<(), String> {
    let parsed = Url::parse(&url).map_err(|_| "链接地址无效".to_string())?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err("只允许打开 HTTP 或 HTTPS 链接".into());
    }

    open::that(parsed.as_str()).map_err(|error| format!("无法在默认浏览器中打开链接: {error}"))
}
