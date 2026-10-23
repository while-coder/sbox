import type { NativePlatform } from './native'
import packageInfo from '../../../../package.json'

function download(blob: Blob, name: string): boolean {
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = name
  document.body.append(link)
  link.click()
  link.remove()
  setTimeout(() => URL.revokeObjectURL(url), 1000)
  return true
}

/** Browser implementation; desktop-only operations fail explicitly. */
export class NativeWeb implements NativePlatform {
  readonly kind = 'web'
  async getAppVersion() { return packageInfo.version }
  async openExternalUrl(url: string): Promise<void> {
    const link = document.createElement('a')
    link.href = url
    link.target = '_blank'
    link.rel = 'noopener noreferrer'
    link.click()
  }
  async saveBinary(bytes: Uint8Array, name: string) {
    return download(new Blob([Uint8Array.from(bytes)]), name)
  }
  async saveText(text: string, name: string) {
    return download(new Blob([text], { type: 'text/plain;charset=utf-8' }), name)
  }
}
