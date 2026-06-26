/**
 * Tauri 平台实现：注入到 @sbox/tools-core 的平台层。
 * WebView 的 <a download> 在 Tauri 中不可靠，统一走「保存对话框选路径 → Rust 落盘」。
 */
import { save, open } from '@tauri-apps/plugin-dialog'
import { join } from '@tauri-apps/api/path'
import { invoke } from '@tauri-apps/api/core'
import { bytesToBase64, stringToBase64, type Platform, type SaveItem } from '@sbox/tools-core'

async function writeBase64(path: string, base64: string): Promise<void> {
  await invoke('save_base64_file', { path, base64: base64.trim() })
}

export const tauriPlatform: Platform = {
  async saveBinary(bytes, defaultName) {
    const path = await save({ defaultPath: defaultName })
    if (!path) return false
    await writeBase64(path, bytesToBase64(bytes))
    return true
  },
  async saveText(text, defaultName) {
    const path = await save({ defaultPath: defaultName })
    if (!path) return false
    await writeBase64(path, stringToBase64(text))
    return true
  },
  async saveBatch(items: SaveItem[]) {
    // 桌面端：弹一次目录选择，全部落盘到该目录
    const dir = await open({ directory: true, title: '选择保存目录' })
    if (!dir || typeof dir !== 'string') return 0
    let n = 0
    for (const it of items) {
      const path = await join(dir, it.name)
      await writeBase64(path, bytesToBase64(it.bytes))
      n += 1
    }
    return n
  },
}
