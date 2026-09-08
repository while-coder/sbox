use serde::Deserialize;
use tauri::{
    webview::WebviewBuilder, AppHandle, LogicalPosition, LogicalSize, Manager, Url, WebviewUrl,
};

const WEBVIEW_LABEL: &str = "translator-webview";

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TranslationProvider {
    Google,
    Baidu,
    Bing,
    Youdao,
    Deepl,
    Tencent,
}

impl TranslationProvider {
    fn url(self) -> &'static str {
        match self {
            Self::Google => "https://translate.google.com/?sl=auto&tl=zh-CN&op=translate",
            Self::Baidu => "https://fanyi.baidu.com/",
            Self::Bing => "https://www.bing.com/translator",
            Self::Youdao => "https://fanyi.youdao.com/",
            Self::Deepl => "https://www.deepl.com/translator",
            Self::Tencent => "https://fanyi.qq.com/",
        }
    }
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationWebviewBounds {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl TranslationWebviewBounds {
    fn validate(self) -> Result<Self, String> {
        if !self.x.is_finite()
            || !self.y.is_finite()
            || !self.width.is_finite()
            || !self.height.is_finite()
            || self.width < 1.0
            || self.height < 1.0
        {
            return Err("翻译网页区域无效".into());
        }
        Ok(self)
    }

    fn position(self) -> LogicalPosition<f64> {
        LogicalPosition::new(self.x.max(0.0), self.y.max(0.0))
    }

    fn size(self) -> LogicalSize<f64> {
        LogicalSize::new(self.width, self.height)
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn translator_open(
    app: AppHandle,
    provider: TranslationProvider,
    bounds: TranslationWebviewBounds,
) -> Result<(), String> {
    let bounds = bounds.validate()?;
    let url = parse_provider_url(provider)?;

    if let Some(webview) = app.get_webview(WEBVIEW_LABEL) {
        webview
            .set_bounds(tauri::Rect {
                position: bounds.position().into(),
                size: bounds.size().into(),
            })
            .map_err(|e| format!("调整翻译网页失败: {e}"))?;
        return webview
            .navigate(url)
            .map_err(|e| format!("打开翻译网页失败: {e}"));
    }

    let window = app
        .get_window("main")
        .ok_or_else(|| "找不到主窗口".to_string())?;
    let builder = WebviewBuilder::new(WEBVIEW_LABEL, WebviewUrl::External(url))
        .focused(true)
        .on_navigation(is_allowed_translation_url);

    window
        .add_child(builder, bounds.position(), bounds.size())
        .map_err(|e| format!("创建翻译网页失败: {e}"))?;
    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn translator_navigate(
    app: AppHandle,
    provider: TranslationProvider,
) -> Result<(), String> {
    let webview = app
        .get_webview(WEBVIEW_LABEL)
        .ok_or_else(|| "翻译网页尚未打开".to_string())?;
    webview
        .navigate(parse_provider_url(provider)?)
        .map_err(|e| format!("切换翻译服务失败: {e}"))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn translator_set_bounds(
    app: AppHandle,
    bounds: TranslationWebviewBounds,
) -> Result<(), String> {
    let bounds = bounds.validate()?;
    if let Some(webview) = app.get_webview(WEBVIEW_LABEL) {
        webview
            .set_bounds(tauri::Rect {
                position: bounds.position().into(),
                size: bounds.size().into(),
            })
            .map_err(|e| format!("调整翻译网页失败: {e}"))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn translator_reload(app: AppHandle) -> Result<(), String> {
    let webview = app
        .get_webview(WEBVIEW_LABEL)
        .ok_or_else(|| "翻译网页尚未打开".to_string())?;
    webview
        .reload()
        .map_err(|e| format!("刷新翻译网页失败: {e}"))
}

#[tauri::command]
pub async fn translator_close(app: AppHandle) -> Result<(), String> {
    if let Some(webview) = app.get_webview(WEBVIEW_LABEL) {
        webview
            .close()
            .map_err(|e| format!("关闭翻译网页失败: {e}"))?;
    }
    Ok(())
}

fn parse_provider_url(provider: TranslationProvider) -> Result<Url, String> {
    provider
        .url()
        .parse()
        .map_err(|e| format!("翻译服务地址无效: {e}"))
}

fn is_allowed_translation_url(url: &Url) -> bool {
    if url.scheme() == "about" {
        return true;
    }
    if url.scheme() != "https" {
        return false;
    }

    matches!(
        url.host_str(),
        Some(
            "translate.google.com"
                | "translate.google.cn"
                | "consent.google.com"
                | "accounts.google.com"
                | "fanyi.baidu.com"
                | "passport.baidu.com"
                | "www.bing.com"
                | "cn.bing.com"
                | "login.live.com"
                | "account.live.com"
                | "fanyi.youdao.com"
                | "www.deepl.com"
                | "account.deepl.com"
                | "fanyi.qq.com"
        )
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_urls_are_allowed() {
        for provider in [
            TranslationProvider::Google,
            TranslationProvider::Baidu,
            TranslationProvider::Bing,
            TranslationProvider::Youdao,
            TranslationProvider::Deepl,
            TranslationProvider::Tencent,
        ] {
            assert!(is_allowed_translation_url(
                &parse_provider_url(provider).unwrap()
            ));
        }
    }

    #[test]
    fn unrelated_and_insecure_navigation_is_blocked() {
        assert!(!is_allowed_translation_url(
            &"https://example.com/".parse().unwrap()
        ));
        assert!(!is_allowed_translation_url(
            &"http://translate.google.com/".parse().unwrap()
        ));
    }

    #[test]
    fn provider_api_values_are_stable() {
        assert!(matches!(
            serde_json::from_str::<TranslationProvider>("\"google\"").unwrap(),
            TranslationProvider::Google
        ));
        assert!(matches!(
            serde_json::from_str::<TranslationProvider>("\"baidu\"").unwrap(),
            TranslationProvider::Baidu
        ));
        assert!(matches!(
            serde_json::from_str::<TranslationProvider>("\"bing\"").unwrap(),
            TranslationProvider::Bing
        ));
        assert!(matches!(
            serde_json::from_str::<TranslationProvider>("\"youdao\"").unwrap(),
            TranslationProvider::Youdao
        ));
        assert!(matches!(
            serde_json::from_str::<TranslationProvider>("\"deepl\"").unwrap(),
            TranslationProvider::Deepl
        ));
        assert!(matches!(
            serde_json::from_str::<TranslationProvider>("\"tencent\"").unwrap(),
            TranslationProvider::Tencent
        ));
    }
}
