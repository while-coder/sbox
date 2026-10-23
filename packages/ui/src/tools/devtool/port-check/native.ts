import { native } from '../../../platform/native'

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
  return native.tools.portCheck.listPort(port)
}

export async function killProcess(pid: number): Promise<void> {
  return native.tools.portCheck.killProcess(pid)
}
