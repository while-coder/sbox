/** 工具使用的桌面平台入口。 */
import type { Platform } from './types'
import { tauriPlatform } from './platform-tauri'

export type { Platform } from './types'

/** 工具内取用当前平台实现。 */
export function getPlatform(): Platform {
  return tauriPlatform
}
