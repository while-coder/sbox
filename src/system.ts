/**
 * 系统集成副作用：关闭到托盘 + 老板键（全局快捷键）。
 * 在 App.vue 的 onMounted 中初始化一次；快捷键设置变更时 watchBossKey 会自动重注册。
 */
import { ref, watch } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { register, unregisterAll } from '@tauri-apps/plugin-global-shortcut'
import { settings } from './settings'

const appWindow = getCurrentWindow()

/** 老板键当前注册状态，供设置页展示反馈。 */
export const bossKeyStatus = ref<{ state: 'ok' | 'error' | 'idle'; message: string }>({
  state: 'idle',
  message: '',
})

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

/** 按当前设置（重新）注册老板键。先清空旧注册，避免改键后旧键残留。 */
export async function applyBossKey(): Promise<void> {
  try {
    await unregisterAll()
    if (!settings.bossKeyEnabled) {
      bossKeyStatus.value = { state: 'idle', message: '老板键已停用' }
      return
    }
    if (!settings.bossKey) {
      bossKeyStatus.value = { state: 'error', message: '未设置快捷键' }
      return
    }
    await register(settings.bossKey, (event) => {
      // 仅在按下时触发，避免松开再触发一次
      if (event.state === 'Pressed') void toggleWindow()
    })
    bossKeyStatus.value = { state: 'ok', message: `已生效：${settings.bossKey}` }
  } catch (e) {
    bossKeyStatus.value = { state: 'error', message: '注册失败，可能与系统或其他程序冲突，请换一组' }
    console.error('注册老板键失败：', e)
  }
}

/** 仅在快捷键相关字段变化时重注册，切换其它设置不受影响。 */
export function watchBossKey(): void {
  watch(
    () => [settings.bossKeyEnabled, settings.bossKey] as const,
    () => { void applyBossKey() },
  )
}
