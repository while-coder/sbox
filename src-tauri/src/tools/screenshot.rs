use base64::Engine;
use std::sync::Mutex;
use xcap::image::codecs::png::{CompressionType, FilterType, PngEncoder};
use xcap::image::{ExtendedColorType, ImageEncoder, RgbaImage};
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

/// 用快速压缩 + 无过滤编码 PNG —— 全屏图默认压缩很慢，这里以体积换速度。
fn encode_png(img: RgbaImage) -> Result<String, String> {
    let (w, h) = (img.width(), img.height());
    let mut buf = Vec::new();
    PngEncoder::new_with_quality(&mut buf, CompressionType::Fast, FilterType::NoFilter)
        .write_image(img.as_raw(), w, h, ExtendedColorType::Rgba8)
        .map_err(|e| e.to_string())?;
    Ok(base64::engine::general_purpose::STANDARD.encode(&buf))
}

/// 捕获主显示器（point 给定时捕获该坐标所在显示器），存入状态并返回。
/// 选取鼠标所在的显示器（多屏时截当前屏）；取不到则回落主显示器。
fn pick_monitor(app: &tauri::AppHandle) -> Result<Monitor, String> {
    if let Ok(pos) = app.cursor_position() {
        if let Ok(m) = Monitor::from_point(pos.x as i32, pos.y as i32) {
            return Ok(m);
        }
    }
    Monitor::all()
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|m| m.is_primary().unwrap_or(false))
        .ok_or_else(|| "未找到显示器".to_string())
}

#[tauri::command]
pub fn screenshot_capture(
    app: tauri::AppHandle,
    state: tauri::State<CaptureState>,
) -> Result<(), String> {
    let monitor = pick_monitor(&app)?;
    let img = monitor.capture_image().map_err(|e| e.to_string())?;
    let cap = Capture {
        width: img.width(),
        height: img.height(),
        base64: encode_png(img)?,
        x: monitor.x().unwrap_or(0),
        y: monitor.y().unwrap_or(0),
        scale: monitor.scale_factor().unwrap_or(1.0),
    };
    // 只存状态，由覆盖层窗口按需取一次，避免把大 base64 多传一程
    *state.0.lock().map_err(|e| e.to_string())? = Some(cap);
    Ok(())
}

/// 覆盖层窗口读取最近一次捕获。
#[tauri::command]
pub fn screenshot_latest(state: tauri::State<CaptureState>) -> Option<Capture> {
    state.0.lock().ok().and_then(|g| g.clone())
}
