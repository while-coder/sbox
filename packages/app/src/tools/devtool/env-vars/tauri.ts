import { invoke } from '@tauri-apps/api/core'

export type EnvVarScope = 'user' | 'system'

export interface EnvVarEntry {
  name: string
  /** 注册表里未展开的原文，保留 %VAR% 引用 */
  rawValue: string
  /** 仅 REG_EXPAND_SZ 且展开成功时提供 */
  expandedValue: string | null
  /** "REG_SZ" | "REG_EXPAND_SZ" | "REG_DWORD" | ... */
  typeName: string
  /** user 范围且类型可编辑时为 true；系统变量恒为 false */
  editable: boolean
}

export interface EnvVarsListResult {
  scope: EnvVarScope
  writable: boolean
  /** 当前进程是否已具备管理员权限（未提权时修改系统变量会弹 UAC） */
  elevated: boolean
  vars: EnvVarEntry[]
}

export async function listEnvVars(scope: EnvVarScope): Promise<EnvVarsListResult> {
  return await invoke('env_vars_list', { scope })
}

export async function setEnvVar(
  scope: EnvVarScope,
  name: string,
  value: string,
  typeName?: string,
): Promise<void> {
  await invoke('env_vars_set', { scope, name, value, typeName })
}

export async function deleteEnvVar(scope: EnvVarScope, name: string): Promise<void> {
  await invoke('env_vars_delete', { scope, name })
}

export interface HostsReadResult {
  content: string
  /** 当前进程是否已具备管理员权限（未提权时保存会弹 UAC） */
  elevated: boolean
}

export async function readHosts(): Promise<HostsReadResult> {
  return await invoke('hosts_read')
}

export async function writeHosts(content: string): Promise<void> {
  await invoke('hosts_write', { content })
}
