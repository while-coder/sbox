import { invoke } from '@tauri-apps/api/core'

export interface GdriveCreds {
  clientId: string
  clientSecret: string
  refreshToken: string
  scope: string
  accessToken: string
}

/** 打开系统浏览器走 Google OAuth，回环捕获授权码并换取 refresh token。 */
export async function oauthLogin(clientId: string, clientSecret: string): Promise<GdriveCreds> {
  return await invoke('gdrive_oauth_login', { clientId, clientSecret })
}
