use base64::Engine;
use std::{fs, path::Path};
use tauri::ipc::Response;

const MAX_IMAGE_FILE_SIZE: u64 = 512 * 1024 * 1024;
const SUPPORTED_IMAGE_EXTENSIONS: &[&str] = &[
    "heic", "heif", "avif", "png", "jpg", "jpeg", "webp", "gif", "bmp", "ico", "tif", "tiff",
    "tga", "qoi", "pnm", "pbm", "pgm", "ppm", "pam", "dds", "hdr", "exr", "svg",
];

fn is_supported_image(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| {
            SUPPORTED_IMAGE_EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str())
        })
        .unwrap_or(false)
}

/// 读取用户通过原生拖放交给图片转换工具的文件。
/// 仅接受绝对路径、普通文件、受支持的图片扩展名，并限制最大体积。
#[tauri::command(rename_all = "camelCase")]
pub fn read_image_file(path: String) -> Result<Response, String> {
    let path = Path::new(&path);
    if !path.is_absolute() {
        return Err("拖入文件必须使用绝对路径".into());
    }
    if !is_supported_image(path) {
        return Err("不支持的图片格式".into());
    }

    let metadata = fs::metadata(path).map_err(|error| format!("读取文件信息失败: {error}"))?;
    if !metadata.is_file() {
        return Err("拖入路径不是普通文件".into());
    }
    if metadata.len() > MAX_IMAGE_FILE_SIZE {
        return Err("图片文件超过 512 MB 限制".into());
    }

    let bytes = fs::read(path).map_err(|error| format!("读取拖入文件失败: {error}"))?;
    Ok(Response::new(bytes))
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_image_extensions_are_case_insensitive() {
        assert!(is_supported_image(Path::new("C:\\images\\photo.HEIC")));
        assert!(is_supported_image(Path::new("C:\\images\\photo.jpeg")));
        assert!(is_supported_image(Path::new("C:\\images\\vector.SVG")));
        assert!(!is_supported_image(Path::new("C:\\images\\secret.txt")));
    }
}
