use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileLocksCheckResult {
    pub path: String,
    pub is_directory: bool,
    pub registered_resource_count: usize,
    pub resource_limit_reached: bool,
    pub processes: Vec<LockingProcess>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LockingProcess {
    pub process_id: u32,
    pub app_name: String,
    pub executable_path: Option<String>,
    pub service_name: Option<String>,
    pub application_type: String,
    pub restartable: bool,
}

#[tauri::command(rename_all = "camelCase")]
pub async fn file_locks_check(path: String) -> Result<FileLocksCheckResult, String> {
    tauri::async_runtime::spawn_blocking(move || check_file_locks(path))
        .await
        .map_err(|error| format!("检查任务失败: {error}"))?
}

#[cfg(windows)]
fn check_file_locks(path: String) -> Result<FileLocksCheckResult, String> {
    windows_impl::check(path)
}

#[cfg(not(windows))]
fn check_file_locks(_path: String) -> Result<FileLocksCheckResult, String> {
    Err("文件占用检查目前仅支持 Windows 桌面端".into())
}

#[cfg(windows)]
mod windows_impl {
    use super::{FileLocksCheckResult, LockingProcess};
    use std::{
        collections::VecDeque,
        os::windows::ffi::OsStrExt,
        path::{Path, PathBuf},
    };
    use windows::{
        core::{PCWSTR, PWSTR},
        Win32::{
            Foundation::{CloseHandle, WIN32_ERROR},
            System::{
                RestartManager::{
                    RmConsole, RmEndSession, RmExplorer, RmGetList, RmMainWindow,
                    RmOtherWindow, RmRegisterResources, RmService, RmStartSession,
                    RM_PROCESS_INFO, CCH_RM_SESSION_KEY,
                },
                Threading::{
                    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
                    PROCESS_QUERY_LIMITED_INFORMATION,
                },
            },
        },
    };

    // Restart Manager recommends at most 64 paths in each RmRegisterResources call.
    const REGISTER_BATCH_SIZE: usize = 64;
    // A large directory can contain millions of entries. Keep the UI responsive and disclose
    // when the result covers only the first portion of its tree.
    const MAX_SCANNED_ENTRIES: usize = 4096;
    const ERROR_WRITE_FAULT: WIN32_ERROR = WIN32_ERROR(29);
    const ERROR_SEM_TIMEOUT: WIN32_ERROR = WIN32_ERROR(121);
    const ERROR_MORE_DATA: WIN32_ERROR = WIN32_ERROR(234);

    pub fn check(path: String) -> Result<FileLocksCheckResult, String> {
        let requested = PathBuf::from(path.trim());
        if requested.as_os_str().is_empty() {
            return Err("请选择文件或文件夹".into());
        }
        if !requested.exists() {
            return Err(format!("路径不存在: {}", requested.display()));
        }

        let target = requested
            .canonicalize()
            .map_err(|error| format!("无法解析路径 {}: {error}", requested.display()))?;
        let is_directory = target.is_dir();
        let (resources, resource_limit_reached) = collect_resources(&target, is_directory);
        let session = RestartManagerSession::start()?;

        for resource_batch in resources.chunks(REGISTER_BATCH_SIZE) {
            let wide_paths: Vec<Vec<u16>> = resource_batch.iter().map(|path| to_wide(path)).collect();
            let paths: Vec<PCWSTR> = wide_paths.iter().map(|path| PCWSTR(path.as_ptr())).collect();
            let status = unsafe {
                RmRegisterResources(session.0, Some(&paths), None, None)
            };
            if status.0 != 0 {
                return Err(format!("登记待检查路径失败（Windows 错误 {}）", status.0));
            }
        }

        let processes = if resources.is_empty() {
            Vec::new()
        } else {
            get_processes(session.0)?
                .into_iter()
                .map(process_from_info)
                .collect()
        };

        Ok(FileLocksCheckResult {
            path: target.to_string_lossy().into_owned(),
            is_directory,
            registered_resource_count: resources.len(),
            resource_limit_reached,
            processes,
        })
    }

    fn collect_resources(target: &Path, is_directory: bool) -> (Vec<PathBuf>, bool) {
        if !is_directory {
            return (vec![target.to_path_buf()], false);
        }

        // Restart Manager returns ERROR_ACCESS_DENIED when a directory is registered.
        // For folders, look for locks on the contained files instead of registering the
        // folder itself or any of its subdirectories.
        let mut resources = Vec::new();
        let mut pending = VecDeque::from([target.to_path_buf()]);
        let mut scanned_entries = 0;
        let mut limit_reached = false;
        while let Some(directory) = pending.pop_front() {
            let Ok(entries) = std::fs::read_dir(directory) else {
                continue;
            };
            for entry in entries.flatten() {
                if scanned_entries >= MAX_SCANNED_ENTRIES {
                    limit_reached = true;
                    break;
                }
                scanned_entries += 1;
                let path = entry.path();
                let Ok(file_type) = entry.file_type() else {
                    continue;
                };
                // pnpm and similar tools use Junctions on Windows. A junction can be
                // reported as a non-directory entry by `file_type`, but Restart Manager
                // resolves it to a directory and then returns ERROR_ACCESS_DENIED.
                if file_type.is_symlink() || path.is_dir() && !file_type.is_dir() {
                    continue;
                }
                if file_type.is_dir() {
                    pending.push_back(path);
                } else {
                    resources.push(path);
                }
            }
            if limit_reached {
                break;
            }
        }
        (resources, limit_reached)
    }

    fn get_processes(session: u32) -> Result<Vec<RM_PROCESS_INFO>, String> {
        // The list can change while it is being read, so retry a few times if it grows.
        for _ in 0..3 {
            let mut needed = 0;
            let mut supplied = 0;
            let mut reboot_reasons = 0;
            let first_status = unsafe {
                RmGetList(
                    session,
                    &mut needed,
                    &mut supplied,
                    None,
                    &mut reboot_reasons,
                )
            };
            if first_status.0 == 0 {
                return Ok(Vec::new());
            }
            if first_status != ERROR_MORE_DATA {
                return Err(format!("读取占用进程失败（Windows 错误 {}）", first_status.0));
            }

            let mut items = vec![RM_PROCESS_INFO::default(); needed as usize];
            supplied = needed;
            let second_status = unsafe {
                RmGetList(
                    session,
                    &mut needed,
                    &mut supplied,
                    Some(items.as_mut_ptr()),
                    &mut reboot_reasons,
                )
            };
            if second_status.0 == 0 {
                items.truncate(supplied as usize);
                return Ok(items);
            }
            if second_status != ERROR_MORE_DATA {
                return Err(format!("读取占用进程失败（Windows 错误 {}）", second_status.0));
            }
        }
        Err("占用进程列表持续变化，请稍后重试".into())
    }

    fn process_from_info(info: RM_PROCESS_INFO) -> LockingProcess {
        let executable_path = process_executable_path(info.Process.dwProcessId);
        let app_name = from_wide(&info.strAppName);
        let service_name = from_wide(&info.strServiceShortName);
        LockingProcess {
            process_id: info.Process.dwProcessId,
            app_name: if app_name.is_empty() {
                executable_path
                    .as_ref()
                    .and_then(|path| Path::new(path).file_name())
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "未知应用".into())
            } else {
                app_name
            },
            executable_path,
            service_name: (!service_name.is_empty()).then_some(service_name),
            application_type: application_type_name(info.ApplicationType.0).into(),
            restartable: info.bRestartable.0 != 0,
        }
    }

    fn process_executable_path(process_id: u32) -> Option<String> {
        let handle = unsafe {
            OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id).ok()?
        };
        let mut buffer = vec![0_u16; 32_768];
        let mut length = buffer.len() as u32;
        let result = unsafe {
            QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, PWSTR(buffer.as_mut_ptr()), &mut length)
        };
        let _ = unsafe { CloseHandle(handle) };
        result.ok()?;
        Some(String::from_utf16_lossy(&buffer[..length as usize]))
    }

    fn application_type_name(application_type: i32) -> &'static str {
        match application_type {
            value if value == RmMainWindow.0 => "主窗口应用",
            value if value == RmOtherWindow.0 => "窗口应用",
            value if value == RmService.0 => "服务",
            value if value == RmExplorer.0 => "资源管理器",
            value if value == RmConsole.0 => "控制台程序",
            _ => "其他程序",
        }
    }

    fn to_wide(path: &Path) -> Vec<u16> {
        path.as_os_str().encode_wide().chain(Some(0)).collect()
    }

    fn from_wide(value: &[u16]) -> String {
        let length = value.iter().position(|character| *character == 0).unwrap_or(value.len());
        String::from_utf16_lossy(&value[..length])
    }

    struct RestartManagerSession(u32);

    impl RestartManagerSession {
        fn start() -> Result<Self, String> {
            for attempt in 0..3 {
                let mut session = 0;
                let mut key = [0_u16; CCH_RM_SESSION_KEY as usize + 1];
                let status = unsafe { RmStartSession(&mut session, None, PWSTR(key.as_mut_ptr())) };
                if status.0 == 0 {
                    return Ok(Self(session));
                }
                if status != ERROR_WRITE_FAULT && status != ERROR_SEM_TIMEOUT || attempt == 2 {
                    return Err(format!("启动 Windows 占用检查会话失败（错误 {}）", status.0));
                }
                std::thread::sleep(std::time::Duration::from_millis(150));
            }
            unreachable!("the loop returns after the final attempt")
        }
    }

    impl Drop for RestartManagerSession {
        fn drop(&mut self) {
            let _ = unsafe { RmEndSession(self.0) };
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::time::{SystemTime, UNIX_EPOCH};

        #[test]
        fn directory_check_registers_contained_files_not_directories() {
            let (root, temporary) = match std::env::var_os("SBOX_FILE_LOCKS_TEST_PATH") {
                Some(path) => (PathBuf::from(path), false),
                None => {
                    let unique = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_nanos();
                    let root = std::env::temp_dir().join(format!(
                        "sbox-file-locks-{}-{unique}",
                        std::process::id()
                    ));
                    let nested = root.join("nested");
                    std::fs::create_dir_all(&nested).unwrap();
                    std::fs::write(root.join("first.txt"), "first").unwrap();
                    std::fs::write(nested.join("second.txt"), "second").unwrap();
                    (root, true)
                }
            };

            let result = check(root.to_string_lossy().into_owned());
            if temporary {
                let _ = std::fs::remove_dir_all(&root);
            }

            let result = result.expect("directory check should not register a directory");
            assert!(result.is_directory);
            if temporary {
                assert_eq!(result.registered_resource_count, 2);
            }
        }
    }
}
