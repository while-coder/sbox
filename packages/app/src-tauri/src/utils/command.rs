//! 外部命令执行辅助。

use std::process::{Command, Output};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// 运行外部命令并收集输出。
///
/// Windows 上附加 `CREATE_NO_WINDOW`，避免 GUI 应用调命令行工具时闪现控制台窗口；
/// 其他平台原样执行。命令本身（参数、环境变量、stdin 等）由调用方构建。
pub fn run(command: &mut Command) -> std::io::Result<Output> {
    #[cfg(windows)]
    {
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    command.output()
}
