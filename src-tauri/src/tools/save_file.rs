use base64::Engine;
use std::fs;

/// 把 base64 内容写入指定路径（路径由前端的保存对话框选定）。
/// 前端 WebView 的 <a download> 在 Tauri 中不可靠，统一走此命令落盘。
#[tauri::command]
pub fn save_base64_file(path: String, base64: String) -> Result<(), String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(base64.trim())
        .map_err(|e| format!("base64 解码失败: {e}"))?;
    fs::write(&path, bytes).map_err(|e| format!("写入失败: {e}"))?;
    Ok(())
}
