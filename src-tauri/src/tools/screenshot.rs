use base64::Engine;
use std::sync::Mutex;
use xcap::image::codecs::png::{CompressionType, FilterType, PngEncoder};
use xcap::image::{ExtendedColorType, ImageEncoder, RgbaImage};
use xcap::Monitor;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MonitorBounds {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

impl MonitorBounds {
    fn contains(self, x: i32, y: i32) -> bool {
        let left = self.x as i64;
        let top = self.y as i64;
        let right = left + self.width as i64;
        let bottom = top + self.height as i64;
        let x = x as i64;
        let y = y as i64;

        x >= left && x < right && y >= top && y < bottom
    }
}

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

fn monitor_bounds(monitor: &Monitor) -> Result<MonitorBounds, String> {
    Ok(MonitorBounds {
        x: monitor.x().map_err(|e| e.to_string())?,
        y: monitor.y().map_err(|e| e.to_string())?,
        width: monitor.width().map_err(|e| e.to_string())?,
        height: monitor.height().map_err(|e| e.to_string())?,
    })
}

fn find_monitor_containing_point(monitors: &[Monitor], x: i32, y: i32) -> Option<Monitor> {
    monitors
        .iter()
        .find(|monitor| monitor_bounds(monitor).is_ok_and(|bounds| bounds.contains(x, y)))
        .cloned()
}

#[cfg(target_os = "windows")]
fn cursor_position(_: &tauri::AppHandle) -> Result<(i32, i32), String> {
    use windows::Win32::Foundation::POINT;
    use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

    let mut point = POINT::default();
    unsafe { GetCursorPos(&mut point).map_err(|e| e.to_string())? };
    Ok((point.x, point.y))
}

#[cfg(not(target_os = "windows"))]
fn cursor_position(app: &tauri::AppHandle) -> Result<(i32, i32), String> {
    let pos = app.cursor_position().map_err(|e| e.to_string())?;
    Ok((pos.x.round() as i32, pos.y.round() as i32))
}

/// 选取鼠标所在的显示器（多屏时截当前屏）；取不到则回落主显示器。
fn pick_monitor(app: &tauri::AppHandle) -> Result<Monitor, String> {
    let monitors = Monitor::all().map_err(|e| e.to_string())?;
    if let Ok((x, y)) = cursor_position(app) {
        if let Some(monitor) = find_monitor_containing_point(&monitors, x, y) {
            return Ok(monitor);
        }

        if let Ok(monitor) = Monitor::from_point(x, y) {
            return Ok(monitor);
        }
    }
    monitors
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

#[cfg(test)]
mod tests {
    use super::MonitorBounds;

    #[test]
    fn bounds_include_points_inside_the_target_monitor() {
        let bounds = MonitorBounds {
            x: 1920,
            y: 0,
            width: 2560,
            height: 1440,
        };

        assert!(bounds.contains(1920, 0));
        assert!(bounds.contains(4479, 1439));
        assert!(!bounds.contains(1919, 0));
        assert!(!bounds.contains(4480, 0));
    }

    #[test]
    fn bounds_handle_monitors_left_of_primary() {
        let bounds = MonitorBounds {
            x: -1280,
            y: 0,
            width: 1280,
            height: 1024,
        };

        assert!(bounds.contains(-1280, 0));
        assert!(bounds.contains(-1, 1023));
        assert!(!bounds.contains(0, 0));
    }
}
