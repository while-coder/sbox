/**
 * 系统集成副作用：关闭到托盘 + 全局快捷键（老板键 / 截图）。
 * 在 App.vue 的 onMounted 中初始化一次；快捷键设置变更时 watchBossKey 会自动重注册。
 * 注意：所有全局快捷键统一在 applyShortcuts 里 unregisterAll 后一起注册，
 * 避免某一个单独 unregisterAll 把另一个清掉。
 */
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { register, unregisterAll } from '@tauri-apps/plugin-global-shortcut'
import { settings } from './settings'
import { startScreenshot } from './tools/screenshot/screenshot'

const appWindow = getCurrentWindow()

type Status = { state: 'ok' | 'error' | 'idle'; message: string }

/** 老板键当前注册状态，供设置页展示反馈。 */
export const bossKeyStatus = ref<Status>({ state: 'idle', message: '' })
/** 截图快捷键当前注册状态。 */
export const screenshotKeyStatus = ref<Status>({ state: 'idle', message: '' })
/** 开机启动当前设置状态。 */
export const autostartStatus = ref<Status>({ state: 'idle', message: '' })

export async function refreshAutostart(): Promise<void> {
  try {
    settings.autostart = await invoke<boolean>('autostart_is_enabled')
    autostartStatus.value = { state: 'ok', message: settings.autostart ? '已启用' : '未启用' }
  } catch (e) {
    autostartStatus.value = { state: 'error', message: '读取失败' }
    console.error('读取开机启动状态失败：', e)
  }
}

export async function setAutostart(enabled: boolean, previous = settings.autostart): Promise<boolean> {
  settings.autostart = enabled
  try {
    await invoke('autostart_set_enabled', { enabled })
    autostartStatus.value = { state: 'ok', message: enabled ? '已启用' : '未启用' }
    return true
  } catch (e) {
    settings.autostart = previous
    autostartStatus.value = { state: 'error', message: '设置失败' }
    console.error('设置开机启动失败：', e)
    return false
  }
}

/** 切换主窗口显隐：可见则隐藏到托盘，否则显示并聚焦。 */
async function toggleWindow(): Promise<void> {
  if (await appWindow.isVisible()) {
    await appWindow.hide()
  } else {
    await appWindow.show()
    await appWindow.setFocus()
  }
}

/** 拦截关闭按钮：开启「最小化到托盘」时阻止关闭并隐藏窗口。 */
export async function setupCloseToTray(): Promise<void> {
  await appWindow.onCloseRequested(async (event) => {
    if (settings.closeToTray) {
      event.preventDefault()
      await appWindow.hide()
    }
  })
}

/** 注册单个快捷键，并把结果写入对应状态 ref。 */
async function registerOne(
  enabled: boolean,
  key: string,
  status: typeof bossKeyStatus,
  disabledMsg: string,
  onPressed: () => void,
): Promise<void> {
  if (!enabled) { status.value = { state: 'idle', message: disabledMsg }; return }
  if (!key) { status.value = { state: 'error', message: '未设置快捷键' }; return }
  try {
    await register(key, (event) => { if (event.state === 'Pressed') onPressed() })
    status.value = { state: 'ok', message: `已生效：${key}` }
  } catch (e) {
    status.value = { state: 'error', message: '注册失败，可能与系统或其他程序冲突，请换一组' }
    console.error('注册快捷键失败：', key, e)
  }
}

/** 按当前设置（重新）注册所有全局快捷键。先整体清空再统一注册。 */
export async function applyShortcuts(): Promise<void> {
  await unregisterAll()
  await registerOne(settings.bossKeyEnabled, settings.bossKey, bossKeyStatus, '老板键已停用', () => void toggleWindow())
  await registerOne(settings.screenshotEnabled, settings.screenshotKey, screenshotKeyStatus, '截图快捷键已停用', () => void startScreenshot())
}

/** 兼容旧调用名。 */
export const applyBossKey = applyShortcuts

/** 快捷键相关字段变化时重注册，切换其它设置不受影响。 */
export function watchBossKey(): void {
  watch(
    () => [settings.bossKeyEnabled, settings.bossKey, settings.screenshotEnabled, settings.screenshotKey] as const,
    () => { void applyShortcuts() },
  )
}
