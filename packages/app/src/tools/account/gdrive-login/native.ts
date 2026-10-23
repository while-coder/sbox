import { invoke } from '@tauri-apps/api/core'

export interface GdriveCreds {
  clientId: string
  clientSecret: string
  refreshToken: string
  scope: string
  accessToken: string
}

/** 可勾选的授权范围（与 Rust 侧白名单一致）。 */
export const GDRIVE_SCOPES = {
  drive: 'https://www.googleapis.com/auth/drive',
  androidpublisher: 'https://www.googleapis.com/auth/androidpublisher',
} as const

/** 打开系统浏览器走 Google OAuth，回环捕获授权码并换取 refresh token。 */
export async function oauthLogin(clientId: string, clientSecret: string, scopes: string[]): Promise<GdriveCreds> {
  return await invoke('gdrive_oauth_login', { clientId, clientSecret, scopes })
}
