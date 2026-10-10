/**
 * 平台注入点。宿主在启动时（createApp 之前）调用 setPlatform 注入实现；
 * 工具内部用 getPlatform 取用。默认回退到 web 实现，便于纯前端调试。
 */
import type { Platform } from './types'
import { webPlatform } from './web'

export type { Platform, SaveItem } from './types'
export { webPlatform } from './web'

let current: Platform = webPlatform

/** 宿主注入平台实现。 */
export function setPlatform(p: Platform): void {
  current = p
}

/** 工具内取用当前平台实现。 */
export function getPlatform(): Platform {
  return current
}
