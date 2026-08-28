use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostsReadResult {
    pub content: String,
    /// 当前进程是否已具备管理员权限（未提权时保存会弹 UAC）
    pub elevated: bool,
}

#[tauri::command(rename_all = "camelCase")]
pub async fn hosts_read() -> Result<HostsReadResult, String> {
    tauri::async_runtime::spawn_blocking(hosts_read_impl)
        .await
        .map_err(|error| format!("读取 hosts 文件失败: {error}"))?
}

#[tauri::command(rename_all = "camelCase")]
pub async fn hosts_write(content: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || hosts_write_impl(&content))
        .await
        .map_err(|error| format!("保存 hosts 文件失败: {error}"))?
}

#[cfg(windows)]
fn hosts_read_impl() -> Result<HostsReadResult, String> {
    windows_impl::read()
}

#[cfg(not(windows))]
fn hosts_read_impl() -> Result<HostsReadResult, String> {
    Err("Hosts 编辑目前仅支持 Windows 桌面端".into())
}

#[cfg(windows)]
fn hosts_write_impl(content: &str) -> Result<(), String> {
    windows_impl::write(content)
}

#[cfg(not(windows))]
fn hosts_write_impl(_content: &str) -> Result<(), String> {
    Err("Hosts 编辑目前仅支持 Windows 桌面端".into())
}

#[cfg(windows)]
pub(super) mod windows_impl {
    use super::HostsReadResult;
    use std::{fs, os::windows::process::CommandExt, path::PathBuf};

    fn hosts_path() -> PathBuf {
        let system_root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into());
        PathBuf::from(system_root).join(r"System32\drivers\etc\hosts")
    }

    pub fn read() -> Result<HostsReadResult, String> {
        let bytes = fs::read(hosts_path()).map_err(|error| format!("读取 hosts 文件失败: {error}"))?;
        // hosts 通常为 UTF-8；遇到非 UTF-8 字节时 lossy 展示，保存会以 UTF-8 写回
        let content = String::from_utf8_lossy(&bytes).into_owned();
        Ok(HostsReadResult {
            content,
            elevated: super::super::env_vars::windows_impl::is_elevated(),
        })
    }

    pub fn write(content: &str) -> Result<(), String> {
        // 非管理员：通过 UAC 提权子进程执行（备份、写入、刷 DNS 都在子进程完成）
        if !super::super::env_vars::windows_impl::is_elevated() {
            return super::super::env_vars::windows_impl::elevated_apply(
                &super::super::env_vars::windows_impl::ElevatedOp::WriteHosts {
                    content: content.into(),
                },
            );
        }

        let path = hosts_path();
        // 修改前先备份，改坏时可直接改名回滚
        if path.exists() {
            fs::copy(&path, path.with_extension("bak"))
                .map_err(|error| format!("备份 hosts 文件失败: {error}"))?;
        }
        let write_result = fs::write(&path, content.as_bytes());
        if let Err(error) = write_result {
            return Err(if error.kind() == std::io::ErrorKind::PermissionDenied {
                "保存 hosts 文件失败：需要管理员权限，请以管理员身份运行 sbox 后重试".into()
            } else {
                format!("保存 hosts 文件失败: {error}")
            });
        }
        flush_dns_cache();
        Ok(())
    }

    /// 尽力刷新 DNS 缓存，让 hosts 修改立即对新解析生效；失败只记日志。
    fn flush_dns_cache() {
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let result = std::process::Command::new("ipconfig")
            .arg("/flushdns")
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        match result {
            Ok(output) if output.status.success() => log::debug!("已刷新 DNS 解析缓存"),
            Ok(output) => log::warn!("刷新 DNS 缓存失败（exit {:?}），可手动执行 ipconfig /flushdns", output.status.code()),
            Err(error) => log::warn!("刷新 DNS 缓存失败: {error}，可手动执行 ipconfig /flushdns"),
        }
    }
}
