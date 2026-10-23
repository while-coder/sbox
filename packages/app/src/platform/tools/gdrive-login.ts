import { invoke } from '@tauri-apps/api/core'
import type { GdriveCreds } from '@sbox/ui/tools/account/gdrive-login/native'

export async function oauthLogin(clientId: string, clientSecret: string, scopes: string[]): Promise<GdriveCreds> {
  return await invoke('gdrive_oauth_login', { clientId, clientSecret, scopes })
}
