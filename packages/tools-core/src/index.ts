/** @sbox/tools-core 统一出口。 */
export type { ToolDef, CategoryKey } from './registry'
export { CATEGORIES, WEB_TOOLS, toolsByCategory, searchTools } from './registry'

export type { Platform } from './platform'
export { getPlatform, setPlatform, webPlatform } from './platform'

export { default as HomeView } from './views/HomeView.vue'

// 供宿主复用的纯逻辑工具函数
export { stringToBase64, bytesToBase64 } from './tools/codec/codec'
