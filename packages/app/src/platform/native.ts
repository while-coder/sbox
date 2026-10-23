import type { Ref } from 'vue'
import { configureNative as configureSharedNative, type NativePlatform as SharedNative } from '@sbox/ui/native'
import type { Capture } from './screenshot-types'
export type Unlisten = () => void
export interface FileDialogOptions {
  title?: string
  defaultPath?: string
  directory?: boolean
  multiple?: boolean
  filters?: { name: string; extensions: string[] }[]
}
export interface NativeUpdater {
  updateStatus: Ref<string>
  updateStatusText: Readonly<Ref<string>>
  checkForUpdate(options?: { silent?: boolean }): Promise<unknown>
}
export interface DesktopNative extends SharedNative {
  readonly kind: 'rust' | 'web'
  readonly windowLabel: string
  readonly updater: NativeUpdater
  readonly screenshot: Pick<typeof import('../tools/media/screenshot/screenshot'),
    'getLatestCapture' | 'getLatestCapturePixels' | 'cropSelectionPixels' | 'saveSelection' |
    'clearCapture' | 'ensureOverlay' | 'startScreenshot' | 'finishScreenshot'>
  getAppVersion(): Promise<string>
  openExternalUrl(url: string): Promise<void>
  pickPaths(options?: FileDialogOptions): Promise<string | string[] | null>
  selectSavePath(options?: FileDialogOptions): Promise<string | null>
  readClipboardText(): Promise<string>
  writeClipboardImage(bytes: Uint8Array, width: number, height: number): Promise<void>
  listen<T>(event: string, handler: (payload: T) => void): Promise<Unlisten>
  onPathDrop(handler: (paths: string[]) => void): Promise<Unlisten>
  setOverlayBounds(capture: Capture): Promise<void>
  showOverlay(): Promise<void>
  getAutostart(): Promise<boolean>
  setAutostart(enabled: boolean): Promise<void>
  toggleWindow(): Promise<void>
  onCloseRequested(shouldHide: () => boolean): Promise<Unlisten>
  registerShortcut(key: string, onPressed: () => void): Promise<void>
  isShortcutRegistered(key: string): Promise<boolean>
  clearShortcuts(): Promise<void>
}

export let native: DesktopNative
/** Configure before importing/mounting the shared app. */
export function configureDesktopNative(platform: DesktopNative): void {
  native = platform
  configureSharedNative(platform)
}
