import { invoke } from '@tauri-apps/api/core'
import type { SshKeyType, SshKeyGenerateInput, SshKeyGenerateResult } from '@sbox/ui/tools/genverify/ssh-keygen/native'

export async function generateSshKey(input: SshKeyGenerateInput): Promise<SshKeyGenerateResult> {
  return await invoke('ssh_key_generate', {
    path: input.path,
    keyType: input.keyType,
    comment: input.comment,
    passphrase: input.passphrase || null,
  })
}
