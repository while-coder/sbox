/**
 * 截图触发：隐藏主窗口 → 捕获主显示器 → 创建全屏覆盖层窗口框选。
 * 覆盖层是同一前端的 #/screenshot-overlay 路由（独立 WebviewWindow）。
 */
import { invoke } from '@tauri-apps/api/core'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { getCurrentWindow } from '@tauri-apps/api/window'

export interface Capture {
  base64: string
  width: number
  height: number
  x: number
  y: number
  scale: number
}

const OVERLAY_LABEL = 'screenshot-overlay'

/** 读取最近一次捕获（覆盖层窗口调用）。 */
export function getLatestCapture(): Promise<Capture | null> {
  return invoke<Capture | null>('screenshot_latest')
}

/** 发起截图。从主窗口上下文调用（按钮或全局快捷键回调）。 */
export async function startScreenshot(): Promise<void> {
  // 关闭可能残留的旧覆盖层
  const old = await WebviewWindow.getByLabel(OVERLAY_LABEL)
  if (old) { try { await old.close() } catch { /* ignore */ } }

  const main = getCurrentWindow()
  const wasVisible = await main.isVisible()
  try {
    if (wasVisible) {
      await main.hide()
      // 等窗口真正从屏幕消失再截，避免把自己截进去
      await new Promise((r) => setTimeout(r, 200))
    }
    await invoke('screenshot_capture')
  } catch (e) {
    if (wasVisible) { await main.show(); await main.setFocus() }
    throw e
  }

  const overlay = new WebviewWindow(OVERLAY_LABEL, {
    url: 'index.html#/screenshot-overlay',
    fullscreen: true,
    transparent: true,
    decorations: false,
    alwaysOnTop: true,
    skipTaskbar: true,
    resizable: false,
    shadow: false,
    title: 'sbox 截图',
  })
  overlay.once('tauri://error', (e) => {
    console.error('创建截图覆盖层失败：', e)
    if (wasVisible) { void main.show() }
  })
}

/** 覆盖层结束（保存/复制/取消）后调用：关闭覆盖层并恢复主窗口。 */
export async function finishScreenshot(): Promise<void> {
  const main = await WebviewWindow.getByLabel('main')
  if (main) { await main.show(); await main.setFocus() }
  const overlay = await WebviewWindow.getByLabel(OVERLAY_LABEL)
  if (overlay) { try { await overlay.close() } catch { /* ignore */ } }
}
