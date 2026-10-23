import { native } from '../../../platform/native'

export interface XiaoaiCreds {
  userId: string
  passToken: string
  deviceId: string
}

export interface XiaoaiDevice {
  deviceID: string
  miotDID: string
  name: string
  alias: string
}

export async function openLogin(): Promise<XiaoaiCreds> {
  return native.tools.xiaoaiLogin.openLogin()
}

export async function logout(): Promise<void> {
  return native.tools.xiaoaiLogin.logout()
}

export async function listDevices(creds: XiaoaiCreds): Promise<XiaoaiDevice[]> {
  return native.tools.xiaoaiLogin.listDevices(creds)
}
