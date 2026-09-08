/**
 * Tauri 平台实现：注入到 @sbox/tools-core 的平台层。
 * WebView 的 <a download> 在 Tauri 中不可靠，统一走「保存对话框选路径 → Rust 落盘」。
 */
import { save, open } from '@tauri-apps/plugin-dialog'
import { basename, join } from '@tauri-apps/api/path'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { bytesToBase64, stringToBase64, type Platform, type SaveItem } from '@sbox/tools-core'

async function writeBase64(path: string, base64: string): Promise<void> {
  await invoke('save_base64_file', { path, base64: base64.trim() })
}

async function readDroppedFile(path: string): Promise<Uint8Array> {
  const response = await invoke<ArrayBuffer | number[]>('read_image_file', { path })
  return response instanceof ArrayBuffer ? new Uint8Array(response) : Uint8Array.from(response)
}

export const tauriPlatform: Platform = {
  async listenFileDrops(onFiles, onError) {
    return getCurrentWebview().onDragDropEvent(async (event) => {
      if (event.payload.type !== 'drop') return

      const results = await Promise.allSettled(event.payload.paths.map(async (path) => {
        try {
          const [bytes, name] = await Promise.all([readDroppedFile(path), basename(path)])
          return new File([bytes], name)
        } catch (error) {
          throw error instanceof Error ? error : new Error(`${path}: ${String(error)}`)
        }
      }))
      const files = results
        .filter((result): result is PromiseFulfilledResult<File> => result.status === 'fulfilled')
        .map(result => result.value)
      const failures = results.filter(result => result.status === 'rejected')

      if (files.length) onFiles(files)
      if (failures.length) {
        console.error('读取拖入文件失败：', failures.map(result => result.reason))
        const first = failures[0].reason
        onError(`有 ${failures.length} 个拖入文件读取失败：${first instanceof Error ? first.message : String(first)}`)
      }
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
