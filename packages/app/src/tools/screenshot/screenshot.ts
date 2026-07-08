/**
 * 截图触发：预创建一个透明全屏覆盖层窗口，空闲时鼠标穿透，截图时接管交互。
 * 流程：（必要时）隐藏主窗口 → 捕获主显示器 → 通知覆盖层刷新并接管交互。
 */
import { invoke } from '@tauri-apps/api/core'
import { emit } from '@tauri-apps/api/event'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { getCurrentWindow } from '@tauri-apps/api/window'
import type { Window } from '@tauri-apps/api/window'
import { settings } from '../../settings'

export interface Capture {
  width: number
  height: number
  previewWidth: number
  previewHeight: number
  x: number
  y: number
  scale: number
}

export type CapturePixels = ArrayBuffer | Uint8Array | number[]

export interface SelectionRect {
  x: number
  y: number
  width: number
  height: number
}

const OVERLAY_LABEL = 'screenshot-overlay'
const HIDE_CAPTURE_SETTLE_MS = 280
/** 截图就绪事件，覆盖层据此刷新画面并接管交互。 */
export const CAPTURE_READY = 'sbox://capture-ready'

/** 读取最近一次捕获（覆盖层窗口调用）。 */
export function getLatestCapture(): Promise<Capture | null> {
  return invoke<Capture | null>('screenshot_latest')
}

/** 读取最近一次捕获的 RGBA 原始像素（覆盖层窗口调用）。 */
export function getLatestCapturePixels(): Promise<CapturePixels> {
  return invoke<CapturePixels>('screenshot_latest_pixels')
}

/** 按原图裁剪选区，返回 RGBA 像素。 */
export function cropSelectionPixels(selection: SelectionRect): Promise<CapturePixels> {
  return invoke<CapturePixels>('screenshot_crop_pixels', { selection })
}

/** 按原图裁剪选区并保存为 PNG。 */
export function saveSelection(path: string, selection: SelectionRect): Promise<void> {
  return invoke('screenshot_save_selection', { path, selection })
}

/** 清理最近一次截图，释放 Rust 端原图缓存。 */
export function clearCapture(): Promise<void> {
  return invoke('screenshot_clear')
}

async function makeOverlayIdle(win: Window): Promise<void> {
  await win.setIgnoreCursorEvents(true)
  // 空闲时隐藏窗口而非仅靠透明穿透：macOS 上透明合成有首帧/时序空窗，
  // 常驻可见会露出一块白色矩形。截图时再由 CAPTURE_READY 回调 show。
  await win.hide()
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

/** 预创建透明的覆盖层窗口（应用启动时调用一次），保持隐藏，截图时再显示。幂等。 */
export async function ensureOverlay(): Promise<WebviewWindow> {
  const existing = await WebviewWindow.getByLabel(OVERLAY_LABEL)
  if (existing) {
    // 保持隐藏；真正的 show 由 CAPTURE_READY 回调在定位并绘制画面后执行。
    await makeOverlayIdle(existing)
    return existing
  }
  const overlay = new WebviewWindow(OVERLAY_LABEL, {
    url: 'index.html#/screenshot-overlay',
    transparent: true,
    backgroundColor: '#00000000',
    decorations: false,
    alwaysOnTop: true,
    skipTaskbar: true,
    resizable: false,
    shadow: false,
    focus: false,
    // 预创建但不显示，避免启动/空闲时露出白块（详见 makeOverlayIdle）。
    visible: false,
    title: 'sbox 截图',
  })
  await makeOverlayIdle(overlay)
  return overlay
}

async function waitUntilHidden(win: Window): Promise<void> {
  if (!(await win.isVisible())) return

  await new Promise<void>((resolve) => {
    let done = false
    let unlisten: UnlistenFn | null = null
    let pollTimer: ReturnType<typeof setTimeout> | null = null
    let fallbackTimer: ReturnType<typeof setTimeout> | null = null

    const cleanup = () => {
      if (unlisten) { unlisten(); unlisten = null }
      if (pollTimer) { clearTimeout(pollTimer); pollTimer = null }
      if (fallbackTimer) { clearTimeout(fallbackTimer); fallbackTimer = null }
    }
    const finish = () => {
      if (done) return
      done = true
      cleanup()
      resolve()
    }
    const checkVisible = async () => {
      if (done) return
      try {
        if (!(await win.isVisible())) {
          finish()
          return
        }
      } catch {
        finish()
        return
      }
      pollTimer = setTimeout(checkVisible, 16)
    }

    void win.onFocusChanged((focused) => {
      if (!focused) void checkVisible()
    }).then((fn) => {
      unlisten = fn
    }).catch(() => {})
    void checkVisible()
    fallbackTimer = setTimeout(finish, 180)
  })
}

/** 发起截图。从主窗口上下文调用（按钮或全局快捷键回调）。 */
export async function startScreenshot(): Promise<void> {
  const main = getCurrentWindow()
  const t0 = performance.now()
  // 仅当设置开启「隐藏自身」且主窗口当前可见时，才需要在截图前隐藏并在结束后恢复。
  const shouldHide =
    settings.screenshotHideSelf && main.label === 'main' && (await main.isVisible())
  let hiddenAt = t0
  let capturedAt = t0
  try {
    if (shouldHide) {
      const hidden = waitUntilHidden(main)
      await main.hide()
      await hidden
      // isVisible=false 只代表 Tauri 状态已变更，Windows DWM 还可能保留一帧隐藏动画残影。
      await sleep(HIDE_CAPTURE_SETTLE_MS)
      hiddenAt = performance.now()
    }
    await invoke('screenshot_capture')
    capturedAt = performance.now()
    await ensureOverlay()
    // 覆盖层收到后自行刷新画面、重置选区并接管鼠标事件。
    await emit(CAPTURE_READY, { restoreMain: shouldHide })
    console.debug('[screenshot] start', {
      hideMs: Math.round(hiddenAt - t0),
      captureMs: Math.round(capturedAt - hiddenAt),
      totalMs: Math.round(performance.now() - t0),
    })
  } catch (e) {
    if (shouldHide) {
      await main.show()
      await main.setFocus()
    }
    console.error('截图失败：', e)
    throw e
  }
}

/** 覆盖层结束（保存/复制/取消）后调用：让覆盖层保持透明穿透，按需恢复主窗口。 */
export async function finishScreenshot(restoreMain: boolean): Promise<void> {
  await makeOverlayIdle(getCurrentWindow()).catch((e) => console.warn('截图覆盖层穿透失败：', e))
  await clearCapture().catch((e) => console.warn('清理截图缓存失败：', e))
  if (restoreMain) {
    const main = await WebviewWindow.getByLabel('main')
    if (main) { await main.show(); await main.setFocus() }
  }
}
