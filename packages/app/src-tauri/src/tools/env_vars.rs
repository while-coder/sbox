use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvVarEntry {
    pub name: String,
    /// 注册表里未展开的原文，保留 %VAR% 引用
    pub raw_value: String,
    /// 仅 REG_EXPAND_SZ 且展开成功时提供
    pub expanded_value: Option<String>,
    /// "REG_SZ" | "REG_EXPAND_SZ" | "REG_DWORD" | ...，原样透传给前端
    pub type_name: String,
    /// user 范围且类型可编辑时为 true；系统变量恒为 false
    pub editable: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvVarsListResult {
    pub scope: String,
    pub writable: bool,
    /// 当前进程是否已具备管理员权限（未提权时修改系统变量会弹 UAC）
    pub elevated: bool,
    pub vars: Vec<EnvVarEntry>,
}

#[tauri::command(rename_all = "camelCase")]
pub async fn env_vars_list(scope: String) -> Result<EnvVarsListResult, String> {
    tauri::async_runtime::spawn_blocking(move || list_env_vars(scope))
        .await
        .map_err(|error| format!("读取环境变量失败: {error}"))?
}

#[tauri::command(rename_all = "camelCase")]
pub async fn env_vars_set(
    scope: String,
    name: String,
    value: String,
    type_name: Option<String>,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || set_env_var(&scope, &name, &value, type_name.as_deref()))
        .await
        .map_err(|error| format!("保存环境变量失败: {error}"))?
}

#[tauri::command(rename_all = "camelCase")]
pub async fn env_vars_delete(scope: String, name: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || delete_env_var(&scope, &name))
        .await
        .map_err(|error| format!("删除环境变量失败: {error}"))?
}

#[cfg(windows)]
fn list_env_vars(scope: String) -> Result<EnvVarsListResult, String> {
    windows_impl::list(&scope)
}

#[cfg(not(windows))]
fn list_env_vars(_scope: String) -> Result<EnvVarsListResult, String> {
    Err("环境变量管理目前仅支持 Windows 桌面端".into())
}

#[cfg(windows)]
fn set_env_var(scope: &str, name: &str, value: &str, type_name: Option<&str>) -> Result<(), String> {
    windows_impl::set(scope, name, value, type_name)
}

#[cfg(not(windows))]
fn set_env_var(_scope: &str, _name: &str, _value: &str, _type_name: Option<&str>) -> Result<(), String> {
    Err("环境变量管理目前仅支持 Windows 桌面端".into())
}

#[cfg(windows)]
fn delete_env_var(scope: &str, name: &str) -> Result<(), String> {
    windows_impl::delete(scope, name)
}

#[cfg(not(windows))]
fn delete_env_var(_scope: &str, _name: &str) -> Result<(), String> {
    Err("环境变量管理目前仅支持 Windows 桌面端".into())
}

/// 提权子进程入口：由父进程以 runas 拉起，执行单次操作并把结果写入 result 文件。
/// 返回进程退出码，0 表示成功。
pub fn apply_elevated_cli(op_file: &str, result_file: &str) -> i32 {
    #[cfg(windows)]
    {
        match windows_impl::apply_elevated(op_file, result_file) {
            Ok(()) => 0,
            Err(_) => 1,
        }
    }
    #[cfg(not(windows))]
    {
        let _ = (op_file, result_file);
        1
    }
}

#[cfg(windows)]
pub(super) mod windows_impl {
    use super::{EnvVarEntry, EnvVarsListResult};
    use std::{mem, slice};
    use windows::{
        core::{PCWSTR, PWSTR},
        Win32::{
            Foundation::{
                CloseHandle, GetLastError, HANDLE, ERROR_ACCESS_DENIED, ERROR_CANCELLED,
                ERROR_FILE_NOT_FOUND, ERROR_MORE_DATA, ERROR_NO_MORE_ITEMS, ERROR_SUCCESS,
                LPARAM, WAIT_TIMEOUT, WPARAM,
            },
            Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY},
            System::Registry::{
                RegCloseKey, RegCreateKeyW, RegDeleteValueW, RegEnumValueW, RegGetValueW,
                RegOpenKeyExW, RegQueryInfoKeyW, RegQueryValueExW, RegSetValueExW, HKEY,
                HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_SET_VALUE, REG_DWORD,
                REG_EXPAND_SZ, REG_SZ, REG_VALUE_TYPE, RRF_RT_REG_SZ,
            },
            System::Threading::{
                GetCurrentProcess, GetExitCodeProcess, OpenProcessToken, WaitForSingleObject,
            },
            UI::Shell::{
                SEE_MASK_FLAG_NO_UI, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW, ShellExecuteExW,
            },
            UI::WindowsAndMessaging::{
                HWND_BROADCAST, SMTO_ABORTIFHUNG, SW_HIDE, SendMessageTimeoutW, WM_SETTINGCHANGE,
            },
        },
    };

    const USER_SUBKEY: &str = "Environment";
    const SYSTEM_SUBKEY: &str = r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment";
    // Windows 进程环境块单个变量的上限（UTF-16 单元数）
    const MAX_VALUE_UTF16_LEN: usize = 32_767;
    // 提权子进程最长等待 5 分钟（用户需先点 UAC，再等写入完成）
    const ELEVATED_TIMEOUT_MS: u32 = 300_000;

    /// 通过 UAC 提权子进程执行的操作。子进程按 action 分发，重复入口只会走正常写路径。
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(tag = "action", rename_all = "snake_case")]
    pub(in crate::tools) enum ElevatedOp {
        SetEnv {
            scope: String,
            name: String,
            value: String,
            type_name: Option<String>,
        },
        DeleteEnv {
            scope: String,
            name: String,
        },
        WriteHosts {
            content: String,
        },
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    struct ElevatedResult {
        ok: bool,
        #[serde(default)]
        message: String,
    }

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

    pub fn list(scope: &str) -> Result<EnvVarsListResult, String> {
        let (root, subkey) = scope_root(scope)?;
        // 两个范围都可写：非管理员改系统变量时会走 UAC 提权子进程
        let elevated = is_elevated();
        let Some(key) = open_for_read(root, subkey)? else {
            return Ok(EnvVarsListResult {
                scope: scope.into(),
                writable: true,
                elevated,
                vars: Vec::new(),
            });
        };
        let vars = enumerate(key.0, true);
        Ok(EnvVarsListResult {
            scope: scope.into(),
            writable: true,
            elevated,
            vars,
        })
    }

    pub fn set(scope: &str, name: &str, value: &str, type_name: Option<&str>) -> Result<(), String> {
        let (root, subkey) = scope_root(scope)?;
        validate_value_name(name)?;
        if value.encode_utf16().count() > MAX_VALUE_UTF16_LEN {
            return Err(format!("值过长（最多 {MAX_VALUE_UTF16_LEN} 个字符）"));
        }

        // 非管理员改系统变量：通过 UAC 提权子进程执行（弹一次授权框）
        if scope == "system" && !is_elevated() {
            return elevated_apply(&ElevatedOp::SetEnv {
                scope: scope.into(),
                name: name.into(),
                value: value.into(),
                type_name: type_name.map(str::to_string),
            });
        }

        let key = open_or_create(root, subkey)?;
        let name_wide = to_wide(name);
        // 请求未指定类型时沿用现有类型，避免 REG_EXPAND_SZ 被降级成 REG_SZ 导致 %引用% 失效
        let value_type = resolve_value_type(key.0, &name_wide, type_name)?;
        let data = encode_data(value, value_type)?;
        let status = unsafe {
            RegSetValueExW(
                key.0,
                PCWSTR::from_raw(name_wide.as_ptr()),
                None,
                value_type,
                Some(&data),
            )
        };
        if status == ERROR_ACCESS_DENIED {
            return Err(admin_required_error());
        }
        if status != ERROR_SUCCESS {
            return Err(win_err("保存环境变量", status));
        }
        broadcast_env_change();
        Ok(())
    }

    pub fn delete(scope: &str, name: &str) -> Result<(), String> {
        let (root, subkey) = scope_root(scope)?;
        validate_value_name(name)?;

        // 非管理员删系统变量：通过 UAC 提权子进程执行
        if scope == "system" && !is_elevated() {
            return elevated_apply(&ElevatedOp::DeleteEnv {
                scope: scope.into(),
                name: name.into(),
            });
        }

        let Some(key) = open_for_write(root, subkey)? else {
            return Ok(());
        };
        let name_wide = to_wide(name);
        let status = unsafe { RegDeleteValueW(key.0, PCWSTR::from_raw(name_wide.as_ptr())) };
        if status == ERROR_FILE_NOT_FOUND {
            return Ok(());
        }
        if status == ERROR_ACCESS_DENIED {
            return Err(admin_required_error());
        }
        if status != ERROR_SUCCESS {
            return Err(win_err("删除环境变量", status));
        }
        broadcast_env_change();
        Ok(())
    }

    fn scope_root(scope: &str) -> Result<(HKEY, &'static str), String> {
        match scope {
            "user" => Ok((HKEY_CURRENT_USER, USER_SUBKEY)),
            "system" => Ok((HKEY_LOCAL_MACHINE, SYSTEM_SUBKEY)),
            _ => Err("未知的环境变量范围".into()),
        }
    }

    /// 当前进程是否以管理员身份运行。
    pub(in crate::tools) fn is_elevated() -> bool {
        unsafe {
            let mut token = HANDLE::default();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
                return false;
            }
            let mut elevation = TOKEN_ELEVATION::default();
            let mut returned = 0u32;
            let result = GetTokenInformation(
                token,
                TokenElevation,
                Some((&mut elevation as *mut TOKEN_ELEVATION).cast()),
                mem::size_of::<TOKEN_ELEVATION>() as u32,
                &mut returned,
            );
            let _ = CloseHandle(token);
            result.is_ok() && elevation.TokenIsElevated != 0
        }
    }

    fn admin_required_error() -> String {
        "需要管理员权限，请以管理员身份运行 sbox 后重试".into()
    }

    /// 以 runas 拉起自身作为提权子进程执行单个操作，等待其完成并回读结果。
    /// 用户在 UAC 弹窗点“否”会得到“已取消管理员授权”。
    pub(in crate::tools) fn elevated_apply(op: &ElevatedOp) -> Result<(), String> {
        let op_json =
            serde_json::to_vec(op).map_err(|error| format!("序列化提权操作失败: {error}"))?;
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let pid = std::process::id();
        let op_path = std::env::temp_dir().join(format!("sbox-elevated-{pid}-{stamp}.json"));
        let result_path =
            std::env::temp_dir().join(format!("sbox-elevated-result-{pid}-{stamp}.json"));
        std::fs::write(&op_path, op_json)
            .map_err(|error| format!("写入提权操作文件失败: {error}"))?;

        let outcome = run_elevated_child(&op_path, &result_path);
        let _ = std::fs::remove_file(&op_path);
        let _ = std::fs::remove_file(&result_path);
        outcome
    }

    fn run_elevated_child(op_path: &std::path::Path, result_path: &std::path::Path) -> Result<(), String> {
        let exe = std::env::current_exe().map_err(|error| format!("获取当前程序路径失败: {error}"))?;
        let verb = to_wide("runas");
        let file = to_wide(&exe.display().to_string());
        let params = to_wide(&format!(
            "--apply-env \"{}\" \"{}\"",
            op_path.display(),
            result_path.display()
        ));

        let mut info = SHELLEXECUTEINFOW::default();
        info.cbSize = mem::size_of::<SHELLEXECUTEINFOW>() as u32;
        info.fMask = SEE_MASK_NOCLOSEPROCESS | SEE_MASK_FLAG_NO_UI;
        info.lpVerb = PCWSTR::from_raw(verb.as_ptr());
        info.lpFile = PCWSTR::from_raw(file.as_ptr());
        info.lpParameters = PCWSTR::from_raw(params.as_ptr());
        info.nShow = SW_HIDE.0;
        if let Err(error) = unsafe { ShellExecuteExW(&mut info) } {
            return Err(if unsafe { GetLastError() } == ERROR_CANCELLED {
                "已取消管理员授权".into()
            } else {
                format!("启动提权进程失败: {error}")
            });
        }

        let handle = info.hProcess;
        if handle.is_invalid() {
            return Err("启动提权进程失败：未获得进程句柄".into());
        }
        let wait = unsafe { WaitForSingleObject(handle, ELEVATED_TIMEOUT_MS) };
        if wait == WAIT_TIMEOUT {
            let _ = unsafe { CloseHandle(handle) };
            return Err("提权进程执行超时".into());
        }
        let mut exit_code = 0u32;
        let read_code = unsafe { GetExitCodeProcess(handle, &mut exit_code) };
        let _ = unsafe { CloseHandle(handle) };
        read_code.map_err(|error| format!("读取提权进程结果失败: {error}"))?;

        if exit_code == 0 {
            return Ok(());
        }
        // 子进程失败时把具体错误写进了 result 文件
        let message = std::fs::read_to_string(result_path)
            .ok()
            .and_then(|text| serde_json::from_str::<ElevatedResult>(&text).ok())
            .filter(|result| !result.ok && !result.message.is_empty())
            .map(|result| result.message)
            .unwrap_or_else(|| "系统环境变量修改失败".into());
        Err(message)
    }

    /// 提权子进程入口：执行 op 文件中的单个操作，把结果写入 result 文件。
    /// 此时进程已具备管理员权限，set/delete/write 走正常写路径并负责广播。
    pub(in crate::tools) fn apply_elevated(op_file: &str, result_file: &str) -> Result<(), String> {
        let data = std::fs::read(op_file).map_err(|error| format!("读取提权操作文件失败: {error}"))?;
        let op: ElevatedOp =
            serde_json::from_slice(&data).map_err(|error| format!("解析提权操作失败: {error}"))?;
        let outcome = match &op {
            ElevatedOp::SetEnv { scope, name, value, type_name } => {
                set(scope, name, value, type_name.as_deref())
            }
            ElevatedOp::DeleteEnv { scope, name } => delete(scope, name),
            ElevatedOp::WriteHosts { content } => crate::tools::hosts::windows_impl::write(content),
        };
        let result = ElevatedResult {
            ok: outcome.is_ok(),
            message: outcome.err().unwrap_or_default(),
        };
        std::fs::write(
            result_file,
            serde_json::to_vec(&result).map_err(|error| format!("序列化结果失败: {error}"))?,
        )
        .map_err(|error| format!("写入结果文件失败: {error}"))?;
        let _ = std::fs::remove_file(op_file);
        Ok(())
    }

    fn validate_value_name(name: &str) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err("变量名不能为空".into());
        }
        if name.trim() != name {
            return Err("变量名首尾不能包含空白字符".into());
        }
        if name.contains('=') || name.contains('\0') {
            return Err("变量名不能包含 = 或空字符".into());
        }
        if name.encode_utf16().count() > 16383 {
            return Err("变量名过长".into());
        }
        Ok(())
    }

    fn open_for_read(root: HKEY, subkey: &str) -> Result<Option<RegKey>, String> {
        let subkey_wide = to_wide(subkey);
        let mut key = HKEY::default();
        let status = unsafe {
            RegOpenKeyExW(
                root,
                PCWSTR::from_raw(subkey_wide.as_ptr()),
                None,
                KEY_READ,
                &mut key,
            )
        };
        if status == ERROR_FILE_NOT_FOUND {
            return Ok(None);
        }
        if status != ERROR_SUCCESS {
            return Err(win_err("打开环境变量注册表键", status));
        }
        Ok(Some(RegKey(key)))
    }

    fn open_for_write(root: HKEY, subkey: &str) -> Result<Option<RegKey>, String> {
        let subkey_wide = to_wide(subkey);
        let mut key = HKEY::default();
        let status = unsafe {
            RegOpenKeyExW(
                root,
                PCWSTR::from_raw(subkey_wide.as_ptr()),
                None,
                KEY_READ | KEY_SET_VALUE,
                &mut key,
            )
        };
        if status == ERROR_FILE_NOT_FOUND {
            return Ok(None);
        }
        if status == ERROR_ACCESS_DENIED {
            return Err(admin_required_error());
        }
        if status != ERROR_SUCCESS {
            return Err(win_err("打开环境变量注册表键", status));
        }
        Ok(Some(RegKey(key)))
    }

    fn open_or_create(root: HKEY, subkey: &str) -> Result<RegKey, String> {
        let subkey_wide = to_wide(subkey);
        let mut key = HKEY::default();
        let status = unsafe {
            RegCreateKeyW(root, PCWSTR::from_raw(subkey_wide.as_ptr()), &mut key)
        };
        if status == ERROR_ACCESS_DENIED {
            return Err(admin_required_error());
        }
        if status != ERROR_SUCCESS {
            return Err(win_err("创建环境变量注册表键", status));
        }
        Ok(RegKey(key))
    }

    fn enumerate(key: HKEY, writable: bool) -> Vec<EnvVarEntry> {
        let mut value_count = 0u32;
        let mut max_name_len = 0u32;
        let status = unsafe {
            RegQueryInfoKeyW(
                key,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(&mut value_count),
                Some(&mut max_name_len),
                None,
                None,
                None,
            )
        };
        if status != ERROR_SUCCESS {
            log::warn!("读取环境变量列表信息失败（Windows 错误 {}）", status.0);
            return Vec::new();
        }

        let mut name_buf = vec![0u16; max_name_len as usize + 2];
        let mut vars = Vec::new();
        for index in 0..value_count {
            let mut name_len = name_buf.len() as u32;
            let status = unsafe {
                RegEnumValueW(
                    key,
                    index,
                    Some(PWSTR(name_buf.as_mut_ptr())),
                    &mut name_len,
                    None,
                    None,
                    None,
                    None,
                )
            };
            if status == ERROR_NO_MORE_ITEMS {
                break;
            }
            if status != ERROR_SUCCESS {
                log::warn!("枚举环境变量第 {index} 项失败（Windows 错误 {}）", status.0);
                continue;
            }
            match query_entry(key, &name_buf[..name_len as usize]) {
                Ok(Some(mut entry)) => {
                    entry.editable &= writable;
                    vars.push(entry);
                }
                // 枚举期间该值被外部删除，跳过即可
                Ok(None) => {}
                Err(error) => {
                    let name = String::from_utf16_lossy(&name_buf[..name_len as usize]);
                    log::warn!("读取环境变量 {name} 失败: {error}");
                }
            }
        }
        vars
    }

    fn query_entry(key: HKEY, name_wide: &[u16]) -> Result<Option<EnvVarEntry>, String> {
        let name_ptr = PCWSTR::from_raw(name_wide.as_ptr());
        let mut value_type = REG_VALUE_TYPE::default();
        let mut size = 0u32;
        let status = unsafe {
            RegQueryValueExW(
                key,
                name_ptr,
                None,
                Some(&mut value_type),
                None,
                Some(&mut size),
            )
        };
        if status == ERROR_FILE_NOT_FOUND {
            return Ok(None);
        }
        if status != ERROR_SUCCESS {
            return Err(win_err("读取变量信息", status));
        }

        let mut data = vec![0u8; size as usize];
        let mut attempts = 0;
        loop {
            let mut data_len = data.len() as u32;
            let status = unsafe {
                RegQueryValueExW(
                    key,
                    name_ptr,
                    None,
                    Some(&mut value_type),
                    Some(data.as_mut_ptr()),
                    Some(&mut data_len),
                )
            };
            if status == ERROR_SUCCESS {
                data.truncate(data_len as usize);
                break;
            }
            if status == ERROR_MORE_DATA {
                // 枚举期间值被并发修改，按新大小重试
                attempts += 1;
                if attempts > 2 {
                    return Err("变量值在读取过程中持续变化，请刷新后重试".into());
                }
                data.resize(data_len as usize, 0);
                continue;
            }
            if status == ERROR_FILE_NOT_FOUND {
                return Ok(None);
            }
            return Err(win_err("读取变量值", status));
        }

        let type_name = type_name_of(value_type);
        let (raw_value, expanded_value, type_editable) = if value_type == REG_DWORD {
            let number = if data.len() >= 4 {
                u32::from_le_bytes([data[0], data[1], data[2], data[3]]).to_string()
            } else {
                String::new()
            };
            (number, None, false)
        } else if value_type == REG_SZ {
            (decode_string(&data), None, true)
        } else if value_type == REG_EXPAND_SZ {
            (decode_string(&data), expanded_of(key, name_wide), true)
        } else {
            // REG_MULTI_SZ / REG_BINARY 等类型暂不支持编辑，只展示类型名
            (String::new(), None, false)
        };

        Ok(Some(EnvVarEntry {
            name: String::from_utf16_lossy(name_wide),
            raw_value,
            expanded_value,
            type_name,
            editable: type_editable,
        }))
    }

    /// 用 RegGetValueW（默认展开语义）取 REG_EXPAND_SZ 的展开后值。
    /// 展开失败（如存在未闭合的 %）时返回 None，前端不展示展开区。
    fn expanded_of(key: HKEY, name_wide: &[u16]) -> Option<String> {
        let name_ptr = PCWSTR::from_raw(name_wide.as_ptr());
        let mut data = vec![0u8; 2048];
        let mut attempts = 0;
        loop {
            let mut size = data.len() as u32;
            let status = unsafe {
                RegGetValueW(
                    key,
                    PCWSTR::null(),
                    name_ptr,
                    RRF_RT_REG_SZ,
                    None,
                    Some(data.as_mut_ptr().cast()),
                    Some(&mut size),
                )
            };
            if status == ERROR_SUCCESS {
                return Some(decode_string(&data[..size as usize]));
            }
            if status == ERROR_MORE_DATA && attempts < 3 {
                attempts += 1;
                data.resize(size as usize, 0);
                continue;
            }
            return None;
        }
    }

    fn resolve_value_type(
        key: HKEY,
        name_wide: &[u16],
        requested: Option<&str>,
    ) -> Result<REG_VALUE_TYPE, String> {
        if let Some(type_name) = requested {
            return match type_name {
                "REG_SZ" => Ok(REG_SZ),
                "REG_EXPAND_SZ" => Ok(REG_EXPAND_SZ),
                other => Err(format!("不支持的值类型: {other}")),
            };
        }
        let mut existing = REG_VALUE_TYPE::default();
        let status = unsafe {
            RegQueryValueExW(
                key,
                PCWSTR::from_raw(name_wide.as_ptr()),
                None,
                Some(&mut existing),
                None,
                None,
            )
        };
        if status == ERROR_FILE_NOT_FOUND {
            return Ok(REG_SZ);
        }
        if status != ERROR_SUCCESS {
            return Err(win_err("读取变量类型", status));
        }
        if existing == REG_SZ || existing == REG_EXPAND_SZ {
            Ok(existing)
        } else {
            Err("该变量类型暂不支持编辑".into())
        }
    }

    fn encode_data(value: &str, value_type: REG_VALUE_TYPE) -> Result<Vec<u8>, String> {
        if value_type == REG_SZ || value_type == REG_EXPAND_SZ {
            let wide = to_wide(value);
            return Ok(as_bytes(&wide).to_vec());
        }
        if value_type == REG_DWORD {
            let number: u32 = value
                .trim()
                .parse()
                .map_err(|_| "数字变量的值必须是 0 到 4294967295 之间的整数".to_string())?;
            return Ok(number.to_le_bytes().to_vec());
        }
        Err("该变量类型暂不支持编辑".into())
    }

    fn type_name_of(value_type: REG_VALUE_TYPE) -> String {
        match value_type.0 {
            0 => "REG_NONE".into(),
            1 => "REG_SZ".into(),
            2 => "REG_EXPAND_SZ".into(),
            3 => "REG_BINARY".into(),
            4 => "REG_DWORD".into(),
            5 => "REG_DWORD_BIG_ENDIAN".into(),
            6 => "REG_LINK".into(),
            7 => "REG_MULTI_SZ".into(),
            11 => "REG_QWORD".into(),
            other => format!("REG_TYPE({other})"),
        }
    }

    fn decode_string(data: &[u8]) -> String {
        let units: Vec<u16> = data
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect();
        let length = units
            .iter()
            .position(|unit| *unit == 0)
            .unwrap_or(units.len());
        String::from_utf16_lossy(&units[..length])
    }

    /// 广播 WM_SETTINGCHANGE，让资源管理器与新启动的进程立即拿到最新环境变量。
    /// 尽力而为：广播失败只记日志，不影响保存结果。
    fn broadcast_env_change() {
        // lParam 需指向以 NUL 结尾的 "Environment" 宽字符串，且在调用期间存活
        let label: Vec<u16> = "Environment".encode_utf16().chain([0]).collect();
        let mut result = 0usize;
        let lresult = unsafe {
            SendMessageTimeoutW(
                HWND_BROADCAST,
                WM_SETTINGCHANGE,
                WPARAM(0),
                LPARAM(label.as_ptr() as isize),
                SMTO_ABORTIFHUNG,
                2000,
                Some(&mut result),
            )
        };
        if lresult.0 == 0 {
            log::warn!(
                "广播环境变量变更失败（Windows 错误 {}）",
                unsafe { GetLastError().0 }
            );
        } else {
            log::debug!("已广播环境变量变更，新启动的程序将读取到最新值");
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn validate_value_name_rejects_invalid_names() {
            assert!(validate_value_name("").is_err());
            assert!(validate_value_name("   ").is_err());
            assert!(validate_value_name(" A").is_err());
            assert!(validate_value_name("A ").is_err());
            assert!(validate_value_name("A=B").is_err());
            assert!(validate_value_name("A\0B").is_err());
            assert!(validate_value_name("PATH").is_ok());
            assert!(validate_value_name("SBOX_TEST_1").is_ok());
        }

        #[test]
        fn type_name_covers_common_registry_types() {
            assert_eq!(type_name_of(REG_SZ), "REG_SZ");
            assert_eq!(type_name_of(REG_EXPAND_SZ), "REG_EXPAND_SZ");
            assert_eq!(type_name_of(REG_DWORD), "REG_DWORD");
            assert_eq!(type_name_of(REG_VALUE_TYPE(7)), "REG_MULTI_SZ");
            assert_eq!(type_name_of(REG_VALUE_TYPE(99)), "REG_TYPE(99)");
        }

        #[test]
        fn encode_data_appends_terminating_nul_for_strings() {
            let encoded = encode_data("abc", REG_SZ).unwrap();
            let units: Vec<u8> = "abc".encode_utf16().chain([0]).flat_map(|u| u.to_le_bytes()).collect();
            assert_eq!(encoded, units);
        }

        #[test]
        fn encode_data_parses_dword_values() {
            assert_eq!(encode_data(" 42 ", REG_DWORD).unwrap(), 42u32.to_le_bytes());
            assert!(encode_data("abc", REG_DWORD).is_err());
            assert!(encode_data("-1", REG_DWORD).is_err());
            assert!(encode_data("a", REG_VALUE_TYPE(7)).is_err());
        }

        #[test]
        fn decode_string_truncates_at_nul_and_is_lossy() {
            let wide: Vec<u16> = "ab".encode_utf16().chain([0, 9, 9]).collect();
            let mut data = Vec::new();
            for unit in &wide {
                data.extend_from_slice(&unit.to_le_bytes());
            }
            assert_eq!(decode_string(&data), "ab");
            assert_eq!(decode_string(&[0xD8, 0xD8]), String::from_utf16_lossy(&[0xD8D8]));
        }

        #[test]
        fn resolve_value_type_requires_supported_type_when_requested() {
            assert!(matches!(
                resolve_value_type(HKEY::default(), &to_wide("nope"), Some("REG_MULTI_SZ")),
                Err(_)
            ));
        }
    }
}
