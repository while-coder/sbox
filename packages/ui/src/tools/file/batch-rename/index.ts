import type { ToolDef } from '../../../shell/registry'

/** 批量重命名。 */
export const batchRenameTool: ToolDef = {
  key: 'batch-rename',
  label: '批量重命名',
  description: '按规则链批量重命名文件：搜索替换、插入文本、序号编号、大小写转换，实时预览结果',
  category: 'file',
  keywords: ['rename', 'batch rename', 'bulk rename', 'renamer', '重命名', '批量改名', '文件名', '规则'],
  component: () => import('./BatchRenameView.vue'),
}
