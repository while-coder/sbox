import type { ToolDef } from '../../../registry'

/** JSON 格式化 / 查看器。 */
export const jsonTool: ToolDef = {
  key: 'json',
  label: 'JSON 格式化 / 查看器',
  description: '美化、压缩、按键排序，树状查看并复制单条属性的值或路径',
  category: 'encode',
  keywords: ['json', 'format', 'beautify', 'minify', 'tree', '美化', '压缩', '格式化', '排序', '路径'],
  component: () => import('./JsonView.vue'),
}
