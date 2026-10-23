import type { NativePlatform, NativeTools, FileDialogOptions, Unlisten } from '@sbox/ui/native'
import type { Capture } from '@sbox/ui/screenshot'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { PhysicalPosition, PhysicalSize } from '@tauri-apps/api/dpi'
import { getVersion } from '@tauri-apps/api/app'
import { Image } from '@tauri-apps/api/image'
import { open, save } from '@tauri-apps/plugin-dialog'
import { readText, writeImage } from '@tauri-apps/plugin-clipboard-manager'
import { register, isRegistered, unregisterAll } from '@tauri-apps/plugin-global-shortcut'
import { useTauriUpdater } from '@while-coder/tauri-updater-vue'
import { rustFiles } from './files'
import * as screenshot from './screenshot'
import * as gdriveLogin from './tools/gdrive-login'
import * as systemInfo from './tools/system-info'
import * as xiaoaiLogin from './tools/xiaoai-login'
import * as envVars from './tools/env-vars'
import * as fileLocks from './tools/file-locks'
import * as portCheck from './tools/port-check'
import * as translator from './tools/translator'
import * as batchRename from './tools/batch-rename'
import * as keystore from './tools/keystore'
import * as sshKeygen from './tools/ssh-keygen'

/** Tauri APIs and Rust command names remain inside the desktop host. */
export class NativeRust implements NativePlatform {
  readonly kind = 'rust'
  get windowLabel() { return getCurrentWindow().label }
  readonly tools: NativeTools = { gdriveLogin, systemInfo, xiaoaiLogin, envVars, fileLocks, portCheck, translator, batchRename, keystore, sshKeygen }
  readonly updater = useTauriUpdater()
  readonly screenshot = {
    ...screenshot,
    async ensureOverlay(): Promise<void> { await screenshot.ensureOverlay() },
  }
  listenFileDrops = rustFiles.listenFileDrops
  saveBinary = rustFiles.saveBinary
  saveText = rustFiles.saveText
  getAppVersion() { return getVersion() }
  openExternalUrl(url: string): Promise<void> { return invoke('open_external_url', { url }) }
  pickPaths(options?: FileDialogOptions) { return open(options) }
  selectSavePath(options?: FileDialogOptions) { return save(options) }
  readClipboardText() { return readText() }
  async writeClipboardImage(bytes: Uint8Array, width: number, height: number): Promise<void> {
    const image = await Image.new(bytes, width, height)
    try { await writeImage(image) } finally { await image.close() }
  }
  listen<T>(event: string, handler: (payload: T) => void): Promise<Unlisten> {
    return listen<T>(event, ({ payload }) => handler(payload))
  }
  onPathDrop(handler: (paths: string[]) => void): Promise<Unlisten> {
    return getCurrentWebview().onDragDropEvent(({ payload }) => {
      if (payload.type === 'drop') handler(payload.paths)
    })
  }
  async setOverlayBounds(capture: Capture): Promise<void> {
    const window = getCurrentWindow()
    await window.setPosition(new PhysicalPosition(capture.x, capture.y))
    await window.setSize(new PhysicalSize(capture.width, capture.height))
  }
  async showOverlay(): Promise<void> {
    const window = getCurrentWindow()
    if (!(await window.isVisible())) await window.show()
    await window.setIgnoreCursorEvents(false)
    await window.setFocus()
  }
  getAutostart(): Promise<boolean> { return invoke('autostart_is_enabled') }
  setAutostart(enabled: boolean): Promise<void> { return invoke('autostart_set_enabled', { enabled }) }
  async toggleWindow(): Promise<void> {
    const window = getCurrentWindow()
    if (await window.isVisible()) await window.hide()
    else { await window.show(); await window.setFocus() }
  }
  onCloseRequested(shouldHide: () => boolean): Promise<Unlisten> {
    const window = getCurrentWindow()
    return window.onCloseRequested(async event => {
      if (shouldHide()) { event.preventDefault(); await window.hide() }
    })
  }
  registerShortcut(key: string, onPressed: () => void): Promise<void> {
    return register(key, event => { if (event.state === 'Pressed') onPressed() })
  }
  isShortcutRegistered(key: string) { return isRegistered(key) }
  clearShortcuts() { return unregisterAll() }
}
