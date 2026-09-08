import { invoke } from '@tauri-apps/api/core'

export interface PortSocket {
  protocol: 'TCP' | 'UDP'
  localAddress: string
  state: string
  pid: number
}

export interface PortProcess {
  pid: number
  processName: string
  executablePath?: string
  sockets: PortSocket[]
}

export interface PortCheckResult {
  port: number
  processes: PortProcess[]
}

export async function listPort(port: number): Promise<PortCheckResult> {
  return await invoke('port_check_list', { port })
}

export async function killProcess(pid: number): Promise<void> {
  return await invoke('port_check_kill', { pid })
}
