import type { Ref } from 'vue'
import type { Platform } from './types'
import type { Capture } from './screenshot-types'
export type Unlisten = () => void
export interface FileDialogOptions {
  title?: string
  defaultPath?: string
  directory?: boolean
  multiple?: boolean
  filters?: { name: string; extensions: string[] }[]
}
export interface NativeTools {
  gdriveLogin: Pick<typeof import('../tools/account/gdrive-login/native'), 'oauthLogin'>
  systemInfo: Pick<typeof import('../tools/account/system-info/native'), 'getSummary' | 'getSection' | 'getPublicIp'>
  xiaoaiLogin: Pick<typeof import('../tools/account/xiaoai-login/native'), 'openLogin' | 'logout' | 'listDevices'>
  envVars: Pick<typeof import('../tools/devtool/env-vars/native'), 'listEnvVars' | 'setEnvVar' | 'deleteEnvVar' | 'readHosts' | 'writeHosts'>
  fileLocks: Pick<typeof import('../tools/devtool/file-locks/native'), 'checkFileLocks'>
  portCheck: Pick<typeof import('../tools/devtool/port-check/native'), 'listPort' | 'killProcess'>
  translator: Pick<typeof import('../tools/devtool/translator/native'), 'openTranslator' | 'navigateTranslator' | 'setTranslatorBounds' | 'reloadTranslator' | 'closeTranslator'>
  batchRename: Pick<typeof import('../tools/file/batch-rename/native'), 'listDirEntries' | 'isDirPath' | 'statPaths' | 'executeBatchRename'>
  keystore: Pick<typeof import('../tools/genverify/keystore/native'), 'checkJava' | 'generateKeystore' | 'fileInfo'>
  sshKeygen: Pick<typeof import('../tools/genverify/ssh-keygen/native'), 'generateSshKey'>
}
export interface NativeUpdater {
  updateStatus: Ref<string>
  updateStatusText: Readonly<Ref<string>>
  checkForUpdate(options?: { silent?: boolean }): Promise<unknown>
}
export interface NativePlatform extends Platform {
  readonly kind: 'rust' | 'web'
  readonly windowLabel: string
  readonly tools: NativeTools
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

export let native: NativePlatform
/** Configure before importing/mounting the shared app. */
export function configureNative(platform: NativePlatform): void { native = platform }
