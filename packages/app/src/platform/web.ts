/**
 * 浏览器平台实现：用 Blob + <a download> 触发下载。
 * 浏览器无「取消」概念，落盘恒返回成功。
 */
import type { Platform, SaveItem } from './types'

function triggerDownload(blob: Blob, filename: string): void {
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = filename
  document.body.appendChild(a)
  a.click()
  a.remove()
  // 延迟回收，确保下载已开始
  setTimeout(() => URL.revokeObjectURL(url), 1000)
}

/** 复制到独立 ArrayBuffer，避免传入的是 WASM 共享内存视图。 */
function toBlob(bytes: Uint8Array, mime: string): Blob {
  return new Blob([bytes.slice().buffer], { type: mime || 'application/octet-stream' })
}

export const webPlatform: Platform = {
  async saveBinary(bytes, defaultName, mime) {
    triggerDownload(toBlob(bytes, mime), defaultName)
    return true
  },
  async saveText(text, defaultName) {
    triggerDownload(new Blob([text], { type: 'text/plain;charset=utf-8' }), defaultName)
    return true
  },
  async saveBatch(items: SaveItem[]) {
    for (const it of items) {
      triggerDownload(toBlob(it.bytes, it.mime), it.name)
    }
    return items.length
  },
}
