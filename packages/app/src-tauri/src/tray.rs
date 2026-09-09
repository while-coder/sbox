//! 主窗口唤醒与系统托盘。

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, WebviewUrl, WebviewWindowBuilder,
};

/// 显示并聚焦主窗口。
pub(crate) fn show_main(app: &tauri::AppHandle) {
    log::debug!("收到主窗口唤醒请求");
    let task_app = app.clone();
    if let Err(error) = app.run_on_main_thread(move || {
        // 主窗口可能挂载了翻译等子 webview，此时 get_webview_window 会因
        // 窗口内存在多 webview 而返回 None，必须用 get_window 拿窗口句柄；
        // 只有窗口彻底不存在时才走重建。
        let window = match task_app.get_window("main") {
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
                    Ok(window) => {
                        // 新建窗口不存在最小化状态，直接显示并聚焦即可。
                        if let Err(error) = window.show() {
                            log::error!("唤醒主窗口失败（show）：{error}");
                        }
                        if let Err(error) = window.set_focus() {
                            log::error!("唤醒主窗口失败（set_focus）：{error}");
                        }
                        log::info!("主窗口已重建并唤醒");
                        return;
                    }
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
pub(crate) fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
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
