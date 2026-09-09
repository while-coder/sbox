mod common;
mod logging;
mod tools;
mod tray;

use tauri::Manager;
use tray::{setup_tray, show_main};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 提权子进程模式：父进程以 runas 拉起本程序执行单次系统级写入，完成后立即退出。
    // 必须在单实例插件初始化之前处理，避免子进程唤醒已运行的主窗口。
    let mut args = std::env::args();
    if args.nth(1).as_deref() == Some("--apply-env") {
        let op_file = args.next().unwrap_or_default();
        let result_file = args.next().unwrap_or_default();
        std::process::exit(tools::env_vars::apply_elevated_cli(&op_file, &result_file));
    }

    let mut builder = tauri::Builder::default();

    // 单实例插件必须第一个注册。第二次启动会自行退出，并唤醒已运行的主窗口。
    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            show_main(app);
        }));
    }

    let builder = builder
        .plugin(logging::logging_plugin())
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
            tools::external::open_external_url,
            tools::env_vars::env_vars_list,
            tools::env_vars::env_vars_set,
            tools::env_vars::env_vars_delete,
            tools::hosts::hosts_read,
            tools::hosts::hosts_write,
            tools::file_locks::file_locks_check,
            tools::port_check::port_check_list,
            tools::port_check::port_check_kill,
            tools::keystore_gen::keystore_check_java,
            tools::keystore_gen::keystore_generate,
            tools::keystore_info::keystore_key_hash,
            tools::keystore_info::keystore_info_list,
            tools::save_file::read_image_file,
            tools::save_file::save_base64_file,
            tools::screenshot::screenshot_clear,
            tools::screenshot::screenshot_crop_pixels,
            tools::screenshot::screenshot_capture,
            tools::screenshot::screenshot_latest,
            tools::screenshot::screenshot_latest_pixels,
            tools::screenshot::screenshot_overlay_disable_animation,
            tools::screenshot::screenshot_save_selection,
            tools::ssh_keygen::ssh_key_generate,
            tools::system_info::system_info_summary,
            tools::system_info::system_info_section,
            tools::system_info::system_public_ip,
            tools::translator::translator_close,
            tools::translator::translator_navigate,
            tools::translator::translator_open,
            tools::translator::translator_reload,
            tools::translator::translator_set_bounds,
        ])
        .run(tauri::generate_context!())
        .expect("error while running sbox");
}
