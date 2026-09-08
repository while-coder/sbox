import type { ToolDef } from '../../../registry'

/** 图片格式转换。 */
export const imageConvertTool: ToolDef = {
  key: 'image-convert',
  label: '图片格式转换',
  description: 'png/jpeg/webp/avif/gif/bmp/ico/tiff 等互转，支持 HEIC 与 SVG 矢量输入；批量转换或单图编辑（旋转 / 裁剪 / 改尺寸）',
  category: 'media',
  keywords: ['image', 'convert', 'format', 'heic', 'heif', 'avif', 'svg', 'ico', 'webp', 'png', 'jpeg', 'tiff', 'bmp', 'gif', '图片', '转换', '格式', '缩放', '批量', '旋转', '裁剪', '单图', 'rotate', 'crop', 'resize'],
  component: () => import('./ImageConvertView.vue'),
}
