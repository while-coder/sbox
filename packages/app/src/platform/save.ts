/**
 * 文件保存（Tauri）。
 * WebView 里的 <a download> 在 Tauri 中不可靠，统一用「保存对话框选路径 → Rust 落盘」。
 */
import { save } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { stringToBase64 } from '@sbox/tools-core'

/** 保存二进制（base64）。返回 false 表示用户取消。 */
export async function saveBase64File(base64: string, defaultName: string): Promise<boolean> {
  const path = await save({ defaultPath: defaultName })
  if (!path) return false
  await invoke('save_base64_file', { path, base64: base64.trim() })
  return true
}

/** 保存文本（UTF-8）。返回 false 表示用户取消。 */
export async function saveTextFile(text: string, defaultName: string): Promise<boolean> {
  return saveBase64File(stringToBase64(text), defaultName)
}
