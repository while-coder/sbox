/**
 * Tauri 平台实现：注入到 @sbox/tools-core 的平台层。
 * WebView 的 <a download> 在 Tauri 中不可靠，统一走「保存对话框选路径 → Rust 落盘」。
 */
import { save, open } from '@tauri-apps/plugin-dialog'
import { basename, join } from '@tauri-apps/api/path'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { readFile } from '@tauri-apps/plugin-fs'
import { bytesToBase64, stringToBase64, type Platform, type SaveItem } from '@sbox/tools-core'

async function writeBase64(path: string, base64: string): Promise<void> {
  await invoke('save_base64_file', { path, base64: base64.trim() })
}

export const tauriPlatform: Platform = {
  async listenFileDrops(onFiles, onError) {
    return getCurrentWebview().onDragDropEvent(async (event) => {
      if (event.payload.type !== 'drop') return

      const results = await Promise.allSettled(event.payload.paths.map(async (path) => {
        const [bytes, name] = await Promise.all([readFile(path), basename(path)])
        return new File([bytes], name)
      }))
      const files = results
        .filter((result): result is PromiseFulfilledResult<File> => result.status === 'fulfilled')
        .map(result => result.value)
      const failures = results.filter(result => result.status === 'rejected')

      if (files.length) onFiles(files)
      if (failures.length) onError(`有 ${failures.length} 个拖入文件读取失败`)
    })
  },
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
