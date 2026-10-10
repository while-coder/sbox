/**
 * Tauri 平台实现：原生文件拖放与保存。
 * WebView 的 <a download> 在 Tauri 中不可靠，统一走「保存对话框选路径 → Rust 落盘」。
 */
import { save } from '@tauri-apps/plugin-dialog'
import { basename } from '@tauri-apps/api/path'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { bytesToBase64, stringToBase64 } from '../tools/encode/codec/codec'
import type { Platform } from './types'

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
}
