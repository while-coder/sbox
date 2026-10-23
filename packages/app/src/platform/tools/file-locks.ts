import { invoke } from '@tauri-apps/api/core'
import type { LockingProcess, FileLocksCheckResult } from '@sbox/ui/tools/devtool/file-locks/native'

export async function checkFileLocks(path: string): Promise<FileLocksCheckResult> {
  return await invoke('file_locks_check', { path })
}
