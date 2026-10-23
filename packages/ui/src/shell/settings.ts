/**
 * 应用设置 —— 单一数据源，持久化到 localStorage。
 * 设置变更通过 watch 触发副作用（重注册老板键），见 system.ts。
 */
import { reactive } from 'vue'

export interface Settings {
  /** 是否随系统开机自动启动 */
  autostart: boolean
  /** 点击关闭按钮（X）时最小化到托盘而非退出 */
  closeToTray: boolean
  /** 是否启用老板键（全局快捷键） */
  bossKeyEnabled: boolean
  /** 老板键组合，Tauri 加速键语法，如 "CommandOrControl+Shift+H" */
  bossKey: string
  /** 是否启用截图全局快捷键 */
  screenshotEnabled: boolean
  /** 截图快捷键组合 */
  screenshotKey: string
  /** 截图时是否先隐藏 sbox 主窗口，避免把自己截进去 */
  screenshotHideSelf: boolean
}

const STORAGE_KEY = 'sbox.settings'

const DEFAULTS: Settings = {
  autostart: false,
  closeToTray: true,
  bossKeyEnabled: true,
  bossKey: 'CommandOrControl+Shift+H',
  screenshotEnabled: true,
  screenshotKey: 'CommandOrControl+Shift+A',
  screenshotHideSelf: true,
}

/** 全局响应式设置对象，组件与副作用共享同一引用。 */
export const settings = reactive<Settings>({ ...DEFAULTS })

/** 从 localStorage 读取并与默认值合并，缺字段用默认补齐。 */
export function loadSettings(): void {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (raw) Object.assign(settings, DEFAULTS, JSON.parse(raw) as Partial<Settings>)
  } catch {
    // 损坏的存储忽略，回落默认值
  }
}

/** 持久化当前设置。任意字段改动后调用。 */
export function saveSettings(): void {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(settings))
}
