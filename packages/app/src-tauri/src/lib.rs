mod common;
mod tools;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, WebviewUrl, WebviewWindowBuilder,
};

/// 显示并聚焦主窗口。
fn show_main(app: &tauri::AppHandle) {
    log::debug!("收到主窗口唤醒请求");
    let task_app = app.clone();
    if let Err(error) = app.run_on_main_thread(move || {
        let window = match task_app.get_webview_window("main") {
            Some(window) => {
                log::debug!("复用现有 main 窗口");
                window
            }
            None => {
                log::warn!("main 窗口已不存在，正在重新创建");
                match WebviewWindowBuilder::new(
                    &task_app,
                    "main",
                    WebviewUrl::App("index.html".into()),
                )
                .title("sbox")
                .inner_size(800.0, 600.0)
                .min_inner_size(600.0, 400.0)
                .resizable(true)
                .build()
                {
                    Ok(window) => window,
                    Err(error) => {
                        log::error!("唤醒主窗口失败（recreate）：{error}");
                        return;
                    }
                }
            }
        };

        // Windows 隐藏或最小化后的恢复顺序应先解除最小化，再显示并聚焦。
        // 托盘与单实例回调不保证运行在 UI 线程，因此统一派发到主线程执行。
        if let Err(error) = window.unminimize() {
            log::error!("唤醒主窗口失败（unminimize）：{error}");
        }
        if let Err(error) = window.show() {
            log::error!("唤醒主窗口失败（show）：{error}");
        }
        if let Err(error) = window.set_focus() {
            log::error!("唤醒主窗口失败（set_focus）：{error}");
        }
        match window.is_visible() {
            Ok(visible) => log::info!("主窗口唤醒完成：visible={visible}"),
            Err(error) => log::error!("读取主窗口可见状态失败：{error}"),
        }
    }) {
        log::error!("唤醒主窗口失败（run_on_main_thread）：{error}");
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
    let mut builder = tauri::Builder::default();

    // 单实例插件必须第一个注册。第二次启动会自行退出，并唤醒已运行的主窗口。
    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            show_main(app);
        }));
    }

    let builder = builder
        .plugin(logging_plugin())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init());

    let builder = tauri_updater_kit::attach_updater(builder);

    builder
        .manage(tools::screenshot::CaptureState::default())
        .setup(|app| {
            log::info!("sbox v{} 启动", app.package_info().version);
            setup_tray(app)?;
            match app.get_webview_window("main") {
                Some(window) => {
                    log::info!("main 窗口初始化完成：visible={:?}", window.is_visible())
                }
                None => log::error!("main 窗口初始化失败：窗口不存在"),
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            tools::autostart::autostart_is_enabled,
            tools::autostart::autostart_set_enabled,
            tools::xiaoai_login::xiaoai_open_login,
            tools::xiaoai_login::xiaoai_logout,
            tools::xiaoai_login::xiaoai_list_devices,
            tools::gdrive_login::gdrive_oauth_login,
            tools::keystore_gen::keystore_check_java,
            tools::keystore_gen::keystore_generate,
            tools::save_file::read_image_file,
            tools::save_file::save_base64_file,
            tools::screenshot::screenshot_clear,
            tools::screenshot::screenshot_crop_pixels,
            tools::screenshot::screenshot_capture,
            tools::screenshot::screenshot_latest,
            tools::screenshot::screenshot_latest_pixels,
            tools::screenshot::screenshot_save_selection,
            tools::ssh_keygen::ssh_key_generate,
            tools::translator::translator_close,
            tools::translator::translator_navigate,
            tools::translator::translator_open,
            tools::translator::translator_reload,
            tools::translator::translator_set_bounds,
        ])
        .run(tauri::generate_context!())
        .expect("error while running sbox");
}
