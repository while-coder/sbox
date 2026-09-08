import { invoke } from '@tauri-apps/api/core'

export interface LockingProcess {
  processId: number
  appName: string
  executablePath?: string
  serviceName?: string
  applicationType: string
  restartable: boolean
}

export interface FileLocksCheckResult {
  path: string
  isDirectory: boolean
  registeredResourceCount: number
  resourceLimitReached: boolean
  processes: LockingProcess[]
}

export async function checkFileLocks(path: string): Promise<FileLocksCheckResult> {
  return await invoke('file_locks_check', { path })
}
