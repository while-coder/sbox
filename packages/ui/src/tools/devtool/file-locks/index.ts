import type { ToolDef } from '../../../shell/registry'

/** 文件占用检查。 */
export const fileLocksTool: ToolDef = {
  key: 'file-locks',
  label: '文件占用检查',
  description: '检查文件或文件夹被哪些 Windows 进程占用，便于处理删除、重命名失败',
  category: 'devtool',
  keywords: ['file lock', 'locked file', 'handle', 'process', 'occupy', '占用', '文件占用', '文件夹占用', '进程', '删除失败', '无法删除', '重命名'],
  component: () => import('./FileLocksView.vue'),
}
