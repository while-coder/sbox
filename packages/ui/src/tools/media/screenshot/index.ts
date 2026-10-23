import type { ToolDef } from '../../../shell/registry'

/** 截图。 */
export const screenshotTool: ToolDef = {
  key: 'screenshot',
  label: '截图',
  description: '全屏框选截图，保存 / 复制到剪贴板 / 识别二维码，支持全局快捷键',
  category: 'media',
  keywords: ['screenshot', 'capture', 'snip', '截图', '截屏', '框选', '快捷键'],
  component: () => import('./ScreenshotView.vue'),
}
