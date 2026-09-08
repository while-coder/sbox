import type { ToolDef } from '../../../registry'

/** 二维码生成 / 识别。 */
export const qrcodeTool: ToolDef = {
  key: 'qrcode',
  label: '二维码生成 / 识别',
  description: '生成二维码，或识别图片中的二维码（支持粘贴、拖拽、读取剪贴板）',
  category: 'media',
  keywords: ['qrcode', 'qr', '二维码', 'scan', 'decode', 'generate', '识别', '生成', '扫码', '剪贴板'],
  component: () => import('./QrView.vue'),
}
