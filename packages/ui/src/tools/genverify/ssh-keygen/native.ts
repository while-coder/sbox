import { native } from '../../../platform/native'

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
  return native.tools.sshKeygen.generateSshKey(input)
}
