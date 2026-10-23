import type { Platform } from './types'
export interface NativePlatform extends Platform {
  readonly kind: 'rust' | 'web'
  getAppVersion(): Promise<string>
  openExternalUrl(url: string): Promise<void>
}
export let native: NativePlatform
export function configureNative(platform: NativePlatform): void { native = platform }
