import { invoke } from '@tauri-apps/api/core'
import type { XiaoaiCreds, XiaoaiDevice } from '@sbox/ui/tools/account/xiaoai-login/native'

export async function openLogin(): Promise<XiaoaiCreds> {
  return await invoke('xiaoai_open_login')
}

export async function logout(): Promise<void> {
  await invoke('xiaoai_logout')
}

export async function listDevices(creds: XiaoaiCreds): Promise<XiaoaiDevice[]> {
  return await invoke('xiaoai_list_devices', { ...creds })
}
