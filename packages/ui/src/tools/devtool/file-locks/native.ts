import { native } from '../../../platform/native'

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
  return native.tools.fileLocks.checkFileLocks(path)
}
