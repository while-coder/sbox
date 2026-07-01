/**
 * 截图触发：预创建一个隐藏的全屏覆盖层窗口，截图时只 show()，避免每次冷启动 webview。
 * 流程：（必要时）隐藏主窗口 → 捕获主显示器 → 通知覆盖层刷新并显示。
 */
import { invoke } from '@tauri-apps/api/core'
import { emit } from '@tauri-apps/api/event'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { settings } from '../../settings'

export interface Capture {
  width: number
  height: number
  x: number
  y: number
  scale: number
}

export type CapturePixels = ArrayBuffer | Uint8Array | number[]

const OVERLAY_LABEL = 'screenshot-overlay'
/** 截图就绪事件，覆盖层据此刷新画面并显示自身。 */
export const CAPTURE_READY = 'sbox://capture-ready'

/** 读取最近一次捕获（覆盖层窗口调用）。 */
export function getLatestCapture(): Promise<Capture | null> {
  return invoke<Capture | null>('screenshot_latest')
}

/** 读取最近一次捕获的 RGBA 原始像素（覆盖层窗口调用）。 */
export function getLatestCapturePixels(): Promise<CapturePixels> {
  return invoke<CapturePixels>('screenshot_latest_pixels')
}

/** 预创建隐藏的覆盖层窗口（应用启动时调用一次）。幂等。 */
export async function ensureOverlay(): Promise<WebviewWindow> {
  const existing = await WebviewWindow.getByLabel(OVERLAY_LABEL)
  if (existing) return existing
  return new WebviewWindow(OVERLAY_LABEL, {
    url: 'index.html#/screenshot-overlay',
    transparent: true,
    decorations: false,
    alwaysOnTop: true,
    skipTaskbar: true,
    resizable: false,
    shadow: false,
    focus: false,
    visible: false,
    title: 'sbox 截图',
  })
}

/** 发起截图。从主窗口上下文调用（按钮或全局快捷键回调）。 */
export async function startScreenshot(): Promise<void> {
  const main = getCurrentWindow()
  // 仅当设置开启「隐藏自身」且主窗口当前可见时，才需要在截图前隐藏并在结束后恢复。
  const shouldHide =
    settings.screenshotHideSelf && main.label === 'main' && (await main.isVisible())
  try {
    if (shouldHide) {
      await main.hide()
      // 等窗口真正从屏幕消失再截，避免把自己截进去。
      // hide() 返回不代表 DWM 已刷掉这一帧，留足时间给合成器，否则会截到 sbox 残影。
      await new Promise((r) => setTimeout(r, 250))
    }
    await invoke('screenshot_capture')
    await ensureOverlay()
    // 覆盖层收到后自行刷新画面、重置选区并 show
    await emit(CAPTURE_READY, { restoreMain: shouldHide })
  } catch (e) {
    if (shouldHide) {
      await main.show()
      await main.setFocus()
    }
    console.error('截图失败：', e)
    throw e
  }
}

/** 覆盖层结束（保存/复制/取消）后调用：隐藏覆盖层，按需恢复主窗口。 */
export async function finishScreenshot(restoreMain: boolean): Promise<void> {
  await getCurrentWindow().hide()
  if (restoreMain) {
    const main = await WebviewWindow.getByLabel('main')
    if (main) { await main.show(); await main.setFocus() }
  }
}
