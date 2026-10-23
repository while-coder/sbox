import { ref } from 'vue'
import type { NativePlatform, NativeTools } from './native'
import packageInfo from '../../../../package.json'

async function unavailable(): Promise<never> { throw new Error('此功能需要桌面客户端') }
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
  readonly windowLabel = 'main'
  readonly tools: NativeTools = {
    gdriveLogin: { oauthLogin: unavailable },
    systemInfo: { getSummary: unavailable, getSection: unavailable, getPublicIp: unavailable },
    xiaoaiLogin: { openLogin: unavailable, logout: unavailable, listDevices: unavailable },
    envVars: { listEnvVars: unavailable, setEnvVar: unavailable, deleteEnvVar: unavailable, readHosts: unavailable, writeHosts: unavailable },
    fileLocks: { checkFileLocks: unavailable },
    portCheck: { listPort: unavailable, killProcess: unavailable },
    translator: { openTranslator: unavailable, navigateTranslator: unavailable, setTranslatorBounds: unavailable, reloadTranslator: unavailable, closeTranslator: unavailable },
    batchRename: { listDirEntries: unavailable, isDirPath: unavailable, statPaths: unavailable, executeBatchRename: unavailable },
    keystore: { checkJava: unavailable, generateKeystore: unavailable, fileInfo: unavailable },
    sshKeygen: { generateSshKey: unavailable },
  }
  readonly updater = {
    updateStatus: ref('unsupported'),
    updateStatusText: ref('浏览器版本无需安装更新'),
    checkForUpdate: unavailable,
  }
  readonly screenshot = {
    getLatestCapture: unavailable, getLatestCapturePixels: unavailable,
    cropSelectionPixels: unavailable, saveSelection: unavailable,
    clearCapture: unavailable, ensureOverlay: unavailable,
    startScreenshot: unavailable, finishScreenshot: unavailable,
  }
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
  async readClipboardText() { return navigator.clipboard.readText() }
  async writeClipboardImage(bytes: Uint8Array, width: number, height: number): Promise<void> {
    const canvas = document.createElement('canvas')
    canvas.width = width
    canvas.height = height
    const context = canvas.getContext('2d')
    if (!context) throw new Error('无法创建图片画布')
    context.putImageData(new ImageData(Uint8ClampedArray.from(bytes), width, height), 0, 0)
    const blob = await new Promise<Blob>((resolve, reject) => canvas.toBlob(
      blob => blob ? resolve(blob) : reject(new Error('图片编码失败')), 'image/png'))
    await navigator.clipboard.write([new ClipboardItem({ 'image/png': blob })])
  }
  async listen<T>(_event: string, _handler: (payload: T) => void) { return () => {} }
  async onPathDrop(_handler: (paths: string[]) => void) { return () => {} }
  pickPaths = unavailable
  selectSavePath = unavailable
  setOverlayBounds = unavailable
  showOverlay = unavailable
  getAutostart = unavailable
  setAutostart = unavailable
  toggleWindow = unavailable
  onCloseRequested = unavailable
  registerShortcut = unavailable
  isShortcutRegistered = unavailable
  clearShortcuts = unavailable
}
