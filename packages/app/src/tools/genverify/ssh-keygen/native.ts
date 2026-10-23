import { invoke } from '@tauri-apps/api/core'

export type SshKeyType = 'ed25519' | 'rsa4096'

export interface SshKeyGenerateInput {
  path: string
  keyType: SshKeyType
  comment: string
  passphrase?: string
}

export interface SshKeyGenerateResult {
  keyType: string
  privateKeyPath: string
  publicKeyPath: string
  privateKey: string
  publicKey: string
  fingerprint: string
  encrypted: boolean
}

export async function generateSshKey(input: SshKeyGenerateInput): Promise<SshKeyGenerateResult> {
  return await invoke('ssh_key_generate', {
    path: input.path,
    keyType: input.keyType,
    comment: input.comment,
    passphrase: input.passphrase || null,
  })
}
