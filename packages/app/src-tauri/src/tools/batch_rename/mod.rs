use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameRequestItem {
    pub from: String,
    pub to: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameResultItem {
    pub from: String,
    pub to: String,
    /// "renamed" | "skipped" | "error"
    pub status: String,
    pub error: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameBatchResult {
    pub items: Vec<RenameResultItem>,
    pub renamed: usize,
    pub skipped: usize,
    pub failed: usize,
}

/// 文件元数据，供前端按大小/时间排序。时间戳为 Unix 毫秒，取不到（平台不支持等）为 null。
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileMeta {
    pub size: u64,
    pub created_ms: Option<i64>,
    pub modified_ms: Option<i64>,
}

/// 批量获取文件元数据，与入参顺序一一对应，单个失败对应项为 null。
#[tauri::command(rename_all = "camelCase")]
pub async fn batch_rename_stat(paths: Vec<String>) -> Result<Vec<Option<FileMeta>>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        Ok(paths.iter().map(|path| stat_one(path)).collect())
    })
    .await
    .map_err(|error| format!("读取文件信息任务失败: {error}"))?
}

fn stat_one(path: &str) -> Option<FileMeta> {
    let meta = std::fs::metadata(path).ok()?;
    let to_ms = |time: std::io::Result<std::time::SystemTime>| {
        time.ok()?
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .map(|d| d.as_millis() as i64)
    };
    Some(FileMeta {
        size: meta.len(),
        created_ms: to_ms(meta.created()),
        modified_ms: to_ms(meta.modified()),
    })
}

/// 判断路径是否为文件夹（拖拽进来的路径需要过滤掉目录）。
#[tauri::command(rename_all = "camelCase")]
pub async fn batch_rename_is_dir(path: String) -> Result<bool, String> {
    Ok(std::path::Path::new(&path).is_dir())
}

/// 列出文件夹的直接子项（recursive=true 时递归收集所有文件，不含子目录），按路径排序。
#[tauri::command(rename_all = "camelCase")]
pub async fn batch_rename_list_dir(path: String, recursive: bool) -> Result<Vec<DirEntry>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        if recursive { walk_dir(path) } else { list_dir(path) }
    })
    .await
    .map_err(|error| format!("读取文件夹任务失败: {error}"))?
}

fn list_dir(path: String) -> Result<Vec<DirEntry>, String> {
    let entries = std::fs::read_dir(&path).map_err(|error| format!("无法读取文件夹 {path}: {error}"))?;
    let mut items: Vec<DirEntry> = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        items.push(DirEntry {
            path: entry.path().to_string_lossy().to_string(),
            name,
            is_dir,
        });
    }
    if items.is_empty() {
        return Err(format!("文件夹为空或无法读取内容: {path}"));
    }
    items.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(items)
}

/// 递归收集文件夹下所有文件（不含子目录条目）。DirEntry::file_type 不跟随符号链接，
/// 因此软链目录不会被展开，天然避免循环。
fn walk_dir(path: String) -> Result<Vec<DirEntry>, String> {
    let mut items: Vec<DirEntry> = Vec::new();
    let mut stack = vec![std::path::PathBuf::from(&path)];
    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            // 单个子目录读取失败（权限等）不中断整体，跳过继续
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
            if is_dir {
                stack.push(entry.path());
            } else {
                items.push(DirEntry {
                    path: entry.path().to_string_lossy().to_string(),
                    name: entry.file_name().to_string_lossy().to_string(),
                    is_dir,
                });
            }
        }
    }
    if items.is_empty() {
        return Err(format!("文件夹为空或无法读取内容: {path}"));
    }
    items.sort_by(|a, b| a.path.to_lowercase().cmp(&b.path.to_lowercase()));
    Ok(items)
}

/// 批量执行重命名。逐项独立处理，单项失败不中断批次；整体只在参数为空时报错。
#[tauri::command(rename_all = "camelCase")]
pub async fn batch_rename_execute(items: Vec<RenameRequestItem>) -> Result<RenameBatchResult, String> {
    if items.is_empty() {
        return Err("没有需要重命名的文件".into());
    }
    tauri::async_runtime::spawn_blocking(move || execute_batch(items))
        .await
        .map_err(|error| format!("重命名任务失败: {error}"))?
}

fn execute_batch(items: Vec<RenameRequestItem>) -> Result<RenameBatchResult, String> {
    let mut results: Vec<RenameResultItem> = Vec::with_capacity(items.len());
    let mut renamed = 0usize;
    let mut skipped = 0usize;
    let mut failed = 0usize;

    for item in items {
        let result = rename_one(&item);
        match result.status.as_str() {
            "renamed" => renamed += 1,
            "skipped" => skipped += 1,
            _ => failed += 1,
        }
        results.push(result);
    }

    Ok(RenameBatchResult {
        items: results,
        renamed,
        skipped,
        failed,
    })
}

fn rename_one(item: &RenameRequestItem) -> RenameResultItem {
    let make_error = |message: String| RenameResultItem {
        from: item.from.clone(),
        to: item.to.clone(),
        status: "error".into(),
        error: Some(message),
    };

    let from = std::path::Path::new(&item.from);
    let to = std::path::Path::new(&item.to);

    if !from.exists() {
        return make_error(format!("源文件不存在: {}", item.from));
    }

    // Windows 文件系统不区分大小写：完全一致视为无变化（跳过），
    // 仅大小写差异视为合法的就地改名（NTFS 支持），其余情况目标已存在则报错
    if item.from == item.to {
        return RenameResultItem {
            from: item.from.clone(),
            to: item.to.clone(),
            status: "skipped".into(),
            error: Some("文件名无变化".into()),
        };
    }

    let case_only = item.from.eq_ignore_ascii_case(&item.to);
    if !case_only && to.symlink_metadata().is_ok() {
        // Rust 的 fs::rename 在 Windows 上带 MOVEFILE_REPLACE_EXISTING，会静默覆盖已存在文件，
        // 必须显式预检（symlink_metadata 同时能发现悬空符号链接）
        return make_error(format!("目标名已存在: {}", item.to));
    }

    match std::fs::rename(from, to) {
        Ok(()) => RenameResultItem {
            from: item.from.clone(),
            to: item.to.clone(),
            status: "renamed".into(),
            error: None,
        },
        Err(error) => {
            let mut message = format!("重命名失败: {error}");
            // 目标被其他进程占用（如正在被编辑器/资源管理器打开）是常见失败原因
            if error.raw_os_error() == Some(32) {
                message.push_str("；文件可能正被其他程序占用，可用「文件占用检查」工具排查");
            }
            make_error(message)
        }
    }
}
