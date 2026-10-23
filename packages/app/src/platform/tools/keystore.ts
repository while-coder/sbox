import { invoke } from '@tauri-apps/api/core'
import type { JavaCheck, DName, GenerateInput, GenerateResult, KeystoreEntry, FileInfoInput, FileInfoResult } from '@sbox/ui/tools/genverify/keystore/native'

export async function checkJava(): Promise<JavaCheck> {
  return await invoke('keystore_check_java')
}

export async function generateKeystore(input: GenerateInput): Promise<GenerateResult> {
  return await invoke('keystore_generate', { ...input })
}

export async function fileInfo(input: FileInfoInput): Promise<FileInfoResult> {
  return await invoke('keystore_file_info', { input })
}
