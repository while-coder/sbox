import { invoke } from '@tauri-apps/api/core'

export interface DirEntryInfo {
  name: string
  path: string
  isDir: boolean
}

export interface RenameRequestItem {
  /** 完整路径 */
  from: string
  /** 完整路径 */
  to: string
}

export interface RenameResultItem {
  from: string
  to: string
  /** "renamed" | "skipped" | "error" */
  status: string
  error?: string | null
}

export interface RenameBatchResult {
  items: RenameResultItem[]
  renamed: number
  skipped: number
  failed: number
}

/** 列出文件夹的直接子项（含子目录，不递归）。 */
export async function listDirEntries(path: string): Promise<DirEntryInfo[]> {
  return await invoke('batch_rename_list_dir', { path })
}

/** 判断路径是否为文件夹。 */
export async function isDirPath(path: string): Promise<boolean> {
  return await invoke('batch_rename_is_dir', { path })
}

/** 批量执行重命名，逐项返回结果，单项失败不中断批次。 */
export async function executeBatchRename(items: RenameRequestItem[]): Promise<RenameBatchResult> {
  return await invoke('batch_rename_execute', { items })
}
