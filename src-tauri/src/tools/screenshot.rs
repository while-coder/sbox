use base64::Engine;
use std::io::Cursor;
use std::sync::Mutex;
use xcap::image::{DynamicImage, ImageFormat};
use xcap::Monitor;

/// 最近一次捕获的全屏快照，供覆盖层窗口读取。
#[derive(Default)]
pub struct CaptureState(pub Mutex<Option<Capture>>);

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Capture {
    /// PNG 的 base64（物理像素）
    pub base64: String,
    pub width: u32,
    pub height: u32,
    /// 显示器在虚拟桌面中的逻辑坐标
    pub x: i32,
    pub y: i32,
    /// DPI 缩放因子（物理 / 逻辑）
    pub scale: f32,
}

fn encode_png(img: xcap::image::RgbaImage) -> Result<String, String> {
    let mut buf = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(img)
        .write_to(&mut buf, ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(base64::engine::general_purpose::STANDARD.encode(buf.into_inner()))
}

/// 捕获主显示器（point 给定时捕获该坐标所在显示器），存入状态并返回。
#[tauri::command]
pub fn screenshot_capture(
    state: tauri::State<CaptureState>,
    point: Option<(i32, i32)>,
) -> Result<Capture, String> {
    let monitor = match point {
        Some((x, y)) => Monitor::from_point(x, y).map_err(|e| e.to_string())?,
        None => Monitor::all()
            .map_err(|e| e.to_string())?
            .into_iter()
            .find(|m| m.is_primary().unwrap_or(false))
            .ok_or_else(|| "未找到主显示器".to_string())?,
    };
    let img = monitor.capture_image().map_err(|e| e.to_string())?;
    let cap = Capture {
        width: img.width(),
        height: img.height(),
        base64: encode_png(img)?,
        x: monitor.x().unwrap_or(0),
        y: monitor.y().unwrap_or(0),
        scale: monitor.scale_factor().unwrap_or(1.0),
    };
    *state.0.lock().map_err(|e| e.to_string())? = Some(cap.clone());
    Ok(cap)
}

/// 覆盖层窗口读取最近一次捕获。
#[tauri::command]
pub fn screenshot_latest(state: tauri::State<CaptureState>) -> Option<Capture> {
    state.0.lock().ok().and_then(|g| g.clone())
}
