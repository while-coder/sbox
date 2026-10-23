import { invoke } from '@tauri-apps/api/core'
import type { PortSocket, PortProcess, PortCheckResult } from '@sbox/ui/tools/devtool/port-check/native'

export async function listPort(port: number): Promise<PortCheckResult> {
  return await invoke('port_check_list', { port })
}

export async function killProcess(pid: number): Promise<void> {
  return await invoke('port_check_kill', { pid })
}
