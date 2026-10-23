import { invoke } from '@tauri-apps/api/core'
import type { DirEntryInfo, FileMetaInfo, RenameRequestItem, RenameResultItem, RenameBatchResult } from '@sbox/ui/tools/file/batch-rename/native'

export async function listDirEntries(path: string, recursive = false): Promise<DirEntryInfo[]> {
  return await invoke('batch_rename_list_dir', { path, recursive })
}

export async function isDirPath(path: string): Promise<boolean> {
  return await invoke('batch_rename_is_dir', { path })
}

export async function statPaths(paths: string[]): Promise<(FileMetaInfo | null)[]> {
  return await invoke('batch_rename_stat', { paths })
}

export async function executeBatchRename(items: RenameRequestItem[]): Promise<RenameBatchResult> {
  return await invoke('batch_rename_execute', { items })
}
