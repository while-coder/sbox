#[cfg(target_os = "windows")]
mod imp {
    use std::{env, mem, slice};
    use windows::{
        core::PCWSTR,
        Win32::{
            Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS},
            System::Registry::{
                RegCloseKey, RegCreateKeyW, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW,
                RegSetValueExW, HKEY, HKEY_CURRENT_USER, KEY_READ, REG_SZ,
            },
        },
    };

    const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    const VALUE_NAME: &str = "com.wandergame.sbox";

    struct RegKey(HKEY);

    impl Drop for RegKey {
        fn drop(&mut self) {
            unsafe {
                let _ = RegCloseKey(self.0);
            }
        }
    }

    fn to_wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain([0]).collect()
    }

    fn as_bytes(value: &[u16]) -> &[u8] {
        unsafe { slice::from_raw_parts(value.as_ptr().cast(), mem::size_of_val(value)) }
    }

    fn win_err(action: &str, code: windows::Win32::Foundation::WIN32_ERROR) -> String {
        format!("{action}失败: Windows error {}", code.0)
    }

    fn run_command() -> Result<String, String> {
        let exe = env::current_exe().map_err(|e| format!("获取当前程序路径失败: {e}"))?;
        Ok(format!("\"{}\"", exe.display()))
    }

    fn open_run_key_for_read() -> Result<Option<RegKey>, String> {
        let subkey = to_wide(RUN_KEY);
        let mut key = HKEY::default();
        let status = unsafe {
            RegOpenKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR::from_raw(subkey.as_ptr()),
                None,
                KEY_READ,
                &mut key,
            )
        };
        if status == ERROR_FILE_NOT_FOUND {
            return Ok(None);
        }
        if status != ERROR_SUCCESS {
            return Err(win_err("打开开机启动注册表", status));
        }
        Ok(Some(RegKey(key)))
    }

    fn open_or_create_run_key_for_write() -> Result<RegKey, String> {
        let subkey = to_wide(RUN_KEY);
        let mut key = HKEY::default();
        let status = unsafe {
            RegCreateKeyW(
                HKEY_CURRENT_USER,
                PCWSTR::from_raw(subkey.as_ptr()),
                &mut key,
            )
        };
        if status != ERROR_SUCCESS {
            return Err(win_err("创建开机启动注册表", status));
        }
        Ok(RegKey(key))
    }

    pub fn is_enabled() -> Result<bool, String> {
        let Some(key) = open_run_key_for_read()? else {
            return Ok(false);
        };
        let name = to_wide(VALUE_NAME);
        let mut size = 0u32;
        let status = unsafe {
            RegQueryValueExW(
                key.0,
                PCWSTR::from_raw(name.as_ptr()),
                None,
                None,
                None,
                Some(&mut size),
            )
        };
        if status == ERROR_FILE_NOT_FOUND {
            return Ok(false);
        }
        if status != ERROR_SUCCESS {
            return Err(win_err("读取开机启动状态", status));
        }
        Ok(size > 0)
    }

    pub fn set_enabled(enabled: bool) -> Result<(), String> {
        let key = open_or_create_run_key_for_write()?;
        let name = to_wide(VALUE_NAME);
        let status = if enabled {
            let command = to_wide(&run_command()?);
            unsafe {
                RegSetValueExW(
                    key.0,
                    PCWSTR::from_raw(name.as_ptr()),
                    None,
                    REG_SZ,
                    Some(as_bytes(&command)),
                )
            }
        } else {
            unsafe { RegDeleteValueW(key.0, PCWSTR::from_raw(name.as_ptr())) }
        };

        if !enabled && status == ERROR_FILE_NOT_FOUND {
            return Ok(());
        }
        if status != ERROR_SUCCESS {
            return Err(win_err("更新开机启动状态", status));
        }
        Ok(())
    }
}

#[cfg(target_os = "macos")]
mod imp {
    use std::{
        env, fs,
        path::{Path, PathBuf},
    };

    const LABEL: &str = "com.wandergame.sbox";

    fn home_dir() -> Result<PathBuf, String> {
        env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or_else(|| "无法获取 HOME 目录".to_string())
    }

    fn plist_path() -> Result<PathBuf, String> {
        Ok(home_dir()?
            .join("Library")
            .join("LaunchAgents")
            .join(format!("{LABEL}.plist")))
    }

    fn escape_xml(value: &str) -> String {
        value
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }

    fn plist_content(exe: &Path) -> String {
        let exe = escape_xml(&exe.display().to_string());
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>{LABEL}</string>
  <key>ProgramArguments</key>
  <array>
    <string>{exe}</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
</dict>
</plist>
"#
        )
    }

    pub fn is_enabled() -> Result<bool, String> {
        Ok(plist_path()?.is_file())
    }

    pub fn set_enabled(enabled: bool) -> Result<(), String> {
        let path = plist_path()?;
        if enabled {
            let exe = env::current_exe().map_err(|e| format!("获取当前程序路径失败: {e}"))?;
            let parent = path
                .parent()
                .ok_or_else(|| "开机启动配置路径无效".to_string())?;
            fs::create_dir_all(parent).map_err(|e| format!("创建 LaunchAgents 目录失败: {e}"))?;
            fs::write(&path, plist_content(&exe))
                .map_err(|e| format!("写入 LaunchAgent 失败: {e}"))?;
        } else if path.exists() {
            fs::remove_file(&path).map_err(|e| format!("删除 LaunchAgent 失败: {e}"))?;
        }
        Ok(())
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
mod imp {
    use std::{env, fs, path::PathBuf};

    const APP_ID: &str = "com.wandergame.sbox";

    fn config_home() -> Result<PathBuf, String> {
        if let Some(path) = env::var_os("XDG_CONFIG_HOME").filter(|value| !value.is_empty()) {
            return Ok(PathBuf::from(path));
        }
        env::var_os("HOME")
            .map(|home| PathBuf::from(home).join(".config"))
            .ok_or_else(|| "无法获取 HOME 目录".to_string())
    }

    fn desktop_path() -> Result<PathBuf, String> {
        Ok(config_home()?
            .join("autostart")
            .join(format!("{APP_ID}.desktop")))
    }

    fn escape_desktop_value(value: &str) -> String {
        value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('`', "\\`")
            .replace('$', "\\$")
    }

    fn desktop_content() -> Result<String, String> {
        let exe = env::current_exe().map_err(|e| format!("获取当前程序路径失败: {e}"))?;
        let exe = escape_desktop_value(&exe.display().to_string());
        Ok(format!(
            r#"[Desktop Entry]
Type=Application
Name=sbox
Exec="{exe}"
Terminal=false
X-GNOME-Autostart-enabled=true
"#
        ))
    }

    pub fn is_enabled() -> Result<bool, String> {
        Ok(desktop_path()?.is_file())
    }

    pub fn set_enabled(enabled: bool) -> Result<(), String> {
        let path = desktop_path()?;
        if enabled {
            let parent = path
                .parent()
                .ok_or_else(|| "开机启动配置路径无效".to_string())?;
            fs::create_dir_all(parent).map_err(|e| format!("创建 autostart 目录失败: {e}"))?;
            fs::write(&path, desktop_content()?)
                .map_err(|e| format!("写入 autostart desktop 文件失败: {e}"))?;
        } else if path.exists() {
            fs::remove_file(&path).map_err(|e| format!("删除 autostart desktop 文件失败: {e}"))?;
        }
        Ok(())
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos", unix)))]
mod imp {
    pub fn is_enabled() -> Result<bool, String> {
        Ok(false)
    }

    pub fn set_enabled(enabled: bool) -> Result<(), String> {
        if enabled {
            Err("当前平台暂不支持开机启动设置".to_string())
        } else {
            Ok(())
        }
    }
}

#[tauri::command]
pub fn autostart_is_enabled() -> Result<bool, String> {
    imp::is_enabled()
}

#[tauri::command]
pub fn autostart_set_enabled(enabled: bool) -> Result<(), String> {
    imp::set_enabled(enabled)
}
