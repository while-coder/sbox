import { invoke } from '@tauri-apps/api/core'
import type { EnvVarScope, EnvVarEntry, EnvVarsListResult, HostsReadResult } from '@sbox/ui/tools/devtool/env-vars/native'

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

export async function readHosts(): Promise<HostsReadResult> {
  return await invoke('hosts_read')
}

export async function writeHosts(content: string): Promise<void> {
  await invoke('hosts_write', { content })
}
