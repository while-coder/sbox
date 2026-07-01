mod common;
mod tools;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};

/// 显示并聚焦主窗口。
fn show_main(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.set_focus();
    }
}

/// 构建系统托盘：左键点击显示窗口，菜单提供 显示 / 设置 / 退出。
fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "显示 sbox", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "设置", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &settings, &quit])?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("sbox")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_main(app),
            "settings" => {
                show_main(app);
                let _ = app.emit("open-settings", ());
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

/// 构建日志插件：
/// - 三路输出：stdout（dev 控制台）、webview（前端 devtools console）、文件（系统日志目录）；
/// - dev 记到 Debug，release 记到 Info，并压低三方库噪声；
/// - 本地时区时间戳，单文件超 10 MB 轮转，保留最近 3 份历史归档（外加当前活动文件，最多 4 个），避免无限增长。
///   日志文件位置（Windows）：%LOCALAPPDATA%/<bundle-id>/logs/。
fn logging_plugin() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    use tauri_plugin_log::{RotationStrategy, Target, TargetKind, TimezoneStrategy};

    let level = if cfg!(debug_assertions) {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    tauri_plugin_log::Builder::new()
        .level(level)
        // 三方库默认只在出问题时才需要，压到 Warn 减少噪声
        .level_for("tao", log::LevelFilter::Warn)
        .level_for("reqwest", log::LevelFilter::Warn)
        .targets([
            Target::new(TargetKind::Stdout),
            Target::new(TargetKind::Webview),
            Target::new(TargetKind::LogDir { file_name: None }),
        ])
        .timezone_strategy(TimezoneStrategy::UseLocal)
        .max_file_size(10_000_000)
        .rotation_strategy(RotationStrategy::KeepSome(3))
        .build()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(logging_plugin())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(tools::screenshot::CaptureState::default())
        .setup(|app| {
            log::info!("sbox v{} 启动", app.package_info().version);
            setup_tray(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            tools::xiaoai_login::xiaoai_open_login,
            tools::xiaoai_login::xiaoai_logout,
            tools::xiaoai_login::xiaoai_list_devices,
            tools::gdrive_login::gdrive_oauth_login,
            tools::keystore_gen::keystore_check_java,
            tools::keystore_gen::keystore_generate,
            tools::save_file::save_base64_file,
            tools::screenshot::screenshot_clear,
            tools::screenshot::screenshot_crop_pixels,
            tools::screenshot::screenshot_capture,
            tools::screenshot::screenshot_latest,
            tools::screenshot::screenshot_latest_pixels,
            tools::screenshot::screenshot_save_selection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running sbox");
}
