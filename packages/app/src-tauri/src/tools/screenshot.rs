use std::fs;
use std::sync::Mutex;
use xcap::image::codecs::png::{CompressionType, FilterType, PngEncoder};
use xcap::image::{ExtendedColorType, ImageEncoder};
use xcap::Monitor;

const PREVIEW_MAX_EDGE: u32 = 2560;

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

pub struct Capture {
    pub meta: CaptureMeta,
    pub rgba: Vec<u8>,
    pub preview_rgba: Option<Vec<u8>>,
}

#[derive(Clone, Copy, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureMeta {
    pub width: u32,
    pub height: u32,
    pub preview_width: u32,
    pub preview_height: u32,
    /// 显示器在虚拟桌面中的逻辑坐标
    pub x: i32,
    pub y: i32,
    /// DPI 缩放因子（物理 / 逻辑）
    pub scale: f32,
}

#[derive(Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectionRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RectMark {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub line_width: u32,
    pub color: [u8; 4],
}

fn preview_size(width: u32, height: u32) -> (u32, u32) {
    let max_edge = width.max(height);
    if max_edge <= PREVIEW_MAX_EDGE {
        return (width, height);
    }
    let preview_width = ((width as u64 * PREVIEW_MAX_EDGE as u64) / max_edge as u64)
        .max(1)
        .try_into()
        .unwrap_or(PREVIEW_MAX_EDGE);
    let preview_height = ((height as u64 * PREVIEW_MAX_EDGE as u64) / max_edge as u64)
        .max(1)
        .try_into()
        .unwrap_or(PREVIEW_MAX_EDGE);
    (preview_width, preview_height)
}

fn downsample_nearest(
    rgba: &[u8],
    width: u32,
    height: u32,
    preview_width: u32,
    preview_height: u32,
) -> Vec<u8> {
    if width == preview_width && height == preview_height {
        return rgba.to_vec();
    }

    let mut out = vec![0; (preview_width as usize) * (preview_height as usize) * 4];
    for py in 0..preview_height {
        let sy = (py as u64 * height as u64 / preview_height as u64) as usize;
        for px in 0..preview_width {
            let sx = (px as u64 * width as u64 / preview_width as u64) as usize;
            let src = (sy * width as usize + sx) * 4;
            let dst = (py as usize * preview_width as usize + px as usize) * 4;
            out[dst..dst + 4].copy_from_slice(&rgba[src..src + 4]);
        }
    }
    out
}

fn clamp_selection(rect: SelectionRect, width: u32, height: u32) -> Result<SelectionRect, String> {
    if rect.width == 0 || rect.height == 0 {
        return Err("选区为空".to_string());
    }

    let x = rect.x.min(width.saturating_sub(1));
    let y = rect.y.min(height.saturating_sub(1));
    let right = rect.x.saturating_add(rect.width).min(width);
    let bottom = rect.y.saturating_add(rect.height).min(height);
    let width = right.saturating_sub(x);
    let height = bottom.saturating_sub(y);
    if width == 0 || height == 0 {
        return Err("选区超出截图范围".to_string());
    }
    Ok(SelectionRect {
        x,
        y,
        width,
        height,
    })
}

fn crop_rgba(
    rgba: &[u8],
    image_width: u32,
    image_height: u32,
    rect: SelectionRect,
) -> Result<Vec<u8>, String> {
    let rect = clamp_selection(rect, image_width, image_height)?;
    let row_bytes = rect.width as usize * 4;
    let mut out = Vec::with_capacity(row_bytes * rect.height as usize);
    for row in rect.y..rect.y + rect.height {
        let start = (row as usize * image_width as usize + rect.x as usize) * 4;
        out.extend_from_slice(&rgba[start..start + row_bytes]);
    }
    Ok(out)
}

fn fill_rgba_rect(
    rgba: &mut [u8],
    image_width: u32,
    left: u32,
    top: u32,
    right: u32,
    bottom: u32,
    color: [u8; 4],
) {
    for y in top..bottom {
        for x in left..right {
            let offset = (y as usize * image_width as usize + x as usize) * 4;
            rgba[offset..offset + 4].copy_from_slice(&color);
        }
    }
}

fn draw_rect_marks(rgba: &mut [u8], width: u32, height: u32, marks: &[RectMark]) {
    for mark in marks {
        if mark.width == 0 || mark.height == 0 || width == 0 || height == 0 {
            continue;
        }
        let left = mark.x.min(width);
        let top = mark.y.min(height);
        let right = mark.x.saturating_add(mark.width).min(width);
        let bottom = mark.y.saturating_add(mark.height).min(height);
        if left >= right || top >= bottom {
            continue;
        }
        let line_width = mark.line_width.max(1).min(right - left).min(bottom - top);
        let inner_right = right.saturating_sub(line_width);
        let inner_bottom = bottom.saturating_sub(line_width);

        fill_rgba_rect(
            rgba,
            width,
            left,
            top,
            right,
            (top + line_width).min(bottom),
            mark.color,
        );
        fill_rgba_rect(
            rgba,
            width,
            left,
            inner_bottom.max(top),
            right,
            bottom,
            mark.color,
        );
        fill_rgba_rect(
            rgba,
            width,
            left,
            top,
            (left + line_width).min(right),
            bottom,
            mark.color,
        );
        fill_rgba_rect(
            rgba,
            width,
            inner_right.max(left),
            top,
            right,
            bottom,
            mark.color,
        );
    }
}

fn encode_png(rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    PngEncoder::new_with_quality(&mut buf, CompressionType::Fast, FilterType::NoFilter)
        .write_image(rgba, width, height, ExtendedColorType::Rgba8)
        .map_err(|e| e.to_string())?;
    Ok(buf)
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

fn capture_monitor_legacy(monitor: &Monitor) -> Result<(Vec<u8>, u32, u32), String> {
    let image = monitor.capture_image().map_err(|e| e.to_string())?;
    let (width, height) = (image.width(), image.height());
    Ok((image.into_raw(), width, height))
}

#[cfg(not(target_os = "macos"))]
fn capture_monitor(monitor: &Monitor) -> Result<(Vec<u8>, u32, u32), String> {
    capture_monitor_legacy(monitor)
}

#[cfg(target_os = "macos")]
fn capture_monitor(monitor: &Monitor) -> Result<(Vec<u8>, u32, u32), String> {
    use objc2_foundation::{NSOperatingSystemVersion, NSProcessInfo};
    use screencapturekit::screenshot_manager::{CGImageExt, SCScreenshotManager};
    use screencapturekit::shareable_content::SCShareableContent;
    use screencapturekit::stream::{
        configuration::SCStreamConfiguration, content_filter::SCContentFilter,
    };

    // SCScreenshotManager 从 macOS 14 开始可用；旧系统保留原来的 xcap 路径，
    // 避免因为修复授权问题而缩小应用原有的系统兼容范围。
    let supports_screenshot_manager =
        NSProcessInfo::processInfo().isOperatingSystemAtLeastVersion(NSOperatingSystemVersion {
            majorVersion: 14,
            minorVersion: 0,
            patchVersion: 0,
        });
    if !supports_screenshot_manager {
        return capture_monitor_legacy(monitor);
    }

    let display_id = monitor.id().map_err(|e| e.to_string())?;
    let content = SCShareableContent::get().map_err(|e| e.to_string())?;
    let displays = content.displays();
    let display = displays
        .iter()
        .find(|display| display.display_id() == display_id)
        .ok_or_else(|| format!("ScreenCaptureKit 未找到显示器 {display_id}"))?;

    // SCDisplay 的宽高单位是逻辑点，而截图输出尺寸使用物理像素。
    let scale = monitor.scale_factor().map_err(|e| e.to_string())?;
    let scale = if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    };
    let output_width = ((display.width() as f64) * f64::from(scale))
        .round()
        .clamp(1.0, u32::MAX as f64) as u32;
    let output_height = ((display.height() as f64) * f64::from(scale))
        .round()
        .clamp(1.0, u32::MAX as f64) as u32;

    let filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
        .try_build()
        .map_err(|e| e.to_string())?;
    let configuration = SCStreamConfiguration::new()
        .with_width(output_width)
        .with_height(output_height)
        // 旧的 CGWindowListCreateImage 不会把鼠标烘焙进截图，保持原有行为。
        .with_shows_cursor(false);
    let image = SCScreenshotManager::capture_image(&filter, &configuration)
        .map_err(|e| format!("ScreenCaptureKit 截图失败: {e}"))?;
    let width = u32::try_from(image.width()).map_err(|_| "截图宽度过大".to_string())?;
    let height = u32::try_from(image.height()).map_err(|_| "截图高度过大".to_string())?;
    let rgba = image.rgba_data().map_err(|e| e.to_string())?;
    Ok((rgba, width, height))
}

#[tauri::command]
pub fn screenshot_capture(
    app: tauri::AppHandle,
    state: tauri::State<CaptureState>,
) -> Result<(), String> {
    let monitor = pick_monitor(&app)?;
    let (rgba, width, height) = capture_monitor(&monitor)?;
    let (preview_width, preview_height) = preview_size(width, height);
    let preview_rgba = downsample_nearest(&rgba, width, height, preview_width, preview_height);
    let meta = CaptureMeta {
        width,
        height,
        preview_width,
        preview_height,
        x: monitor.x().unwrap_or(0),
        y: monitor.y().unwrap_or(0),
        scale: monitor.scale_factor().unwrap_or(1.0),
    };
    let cap = Capture {
        meta,
        rgba,
        preview_rgba: Some(preview_rgba),
    };
    *state.0.lock().map_err(|e| e.to_string())? = Some(cap);
    Ok(())
}

/// 覆盖层窗口读取最近一次捕获的元数据。
#[tauri::command]
pub fn screenshot_latest(state: tauri::State<CaptureState>) -> Option<CaptureMeta> {
    state
        .0
        .lock()
        .ok()
        .and_then(|g| g.as_ref().map(|c| c.meta.clone()))
}

/// 覆盖层窗口读取最近一次捕获的预览 RGBA 像素。
#[tauri::command]
pub fn screenshot_latest_pixels(
    state: tauri::State<CaptureState>,
) -> Result<tauri::ipc::Response, String> {
    let mut guard = state.0.lock().map_err(|e| e.to_string())?;
    let pixels = guard
        .as_mut()
        .ok_or_else(|| "暂无截图".to_string())?
        .preview_rgba
        .take()
        .ok_or_else(|| "截图预览像素已读取".to_string())?;
    Ok(tauri::ipc::Response::new(pixels))
}

/// 按原图裁剪选区并合成矩形标记，返回 RGBA 原始像素。
#[tauri::command]
pub fn screenshot_crop_pixels(
    state: tauri::State<CaptureState>,
    selection: SelectionRect,
    marks: Vec<RectMark>,
) -> Result<tauri::ipc::Response, String> {
    let guard = state.0.lock().map_err(|e| e.to_string())?;
    let capture = guard.as_ref().ok_or_else(|| "暂无截图".to_string())?;
    let selection = clamp_selection(selection, capture.meta.width, capture.meta.height)?;
    let mut pixels = crop_rgba(
        &capture.rgba,
        capture.meta.width,
        capture.meta.height,
        selection,
    )?;
    draw_rect_marks(&mut pixels, selection.width, selection.height, &marks);
    Ok(tauri::ipc::Response::new(pixels))
}

/// 按原图裁剪选区、合成矩形标记并保存为 PNG。
#[tauri::command]
pub fn screenshot_save_selection(
    state: tauri::State<CaptureState>,
    path: String,
    selection: SelectionRect,
    marks: Vec<RectMark>,
) -> Result<(), String> {
    let guard = state.0.lock().map_err(|e| e.to_string())?;
    let capture = guard.as_ref().ok_or_else(|| "暂无截图".to_string())?;
    let selection = clamp_selection(selection, capture.meta.width, capture.meta.height)?;
    let mut pixels = crop_rgba(
        &capture.rgba,
        capture.meta.width,
        capture.meta.height,
        selection,
    )?;
    draw_rect_marks(&mut pixels, selection.width, selection.height, &marks);
    let png = encode_png(&pixels, selection.width, selection.height)?;
    fs::write(&path, png).map_err(|e| format!("写入失败: {e}"))?;
    Ok(())
}

/// 清理最近一次截图，释放原图内存。
#[tauri::command]
pub fn screenshot_clear(state: tauri::State<CaptureState>) -> Result<(), String> {
    *state.0.lock().map_err(|e| e.to_string())? = None;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        clamp_selection, draw_rect_marks, preview_size, MonitorBounds, RectMark, SelectionRect,
    };

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

    #[test]
    fn preview_size_caps_large_retina_capture() {
        assert_eq!(preview_size(5120, 2880), (2560, 1440));
        assert_eq!(preview_size(1920, 1080), (1920, 1080));
    }

    #[test]
    fn clamp_selection_keeps_rect_inside_capture() {
        let rect = clamp_selection(
            SelectionRect {
                x: 90,
                y: 90,
                width: 20,
                height: 30,
            },
            100,
            100,
        )
        .unwrap();

        assert_eq!(rect.x, 90);
        assert_eq!(rect.y, 90);
        assert_eq!(rect.width, 10);
        assert_eq!(rect.height, 10);
    }

    #[test]
    fn rect_marks_draw_an_outline_without_filling_the_center() {
        let mut pixels = vec![0; 5 * 5 * 4];
        draw_rect_marks(
            &mut pixels,
            5,
            5,
            &[RectMark {
                x: 1,
                y: 1,
                width: 3,
                height: 3,
                line_width: 1,
                color: [255, 0, 0, 255],
            }],
        );

        let pixel = |x: usize, y: usize| {
            let offset = (y * 5 + x) * 4;
            <[u8; 4]>::try_from(&pixels[offset..offset + 4]).unwrap()
        };
        assert_eq!(pixel(1, 1), [255, 0, 0, 255]);
        assert_eq!(pixel(3, 3), [255, 0, 0, 255]);
        assert_eq!(pixel(2, 2), [0, 0, 0, 0]);
        assert_eq!(pixel(0, 0), [0, 0, 0, 0]);
    }
}
