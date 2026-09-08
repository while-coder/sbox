import type { ToolDef } from '../../../shell/registry'

/** 端口占用检查。 */
export const portCheckTool: ToolDef = {
  key: 'port-check',
  label: '端口占用检查',
  description: '查看本地 TCP / UDP 端口被哪些进程占用，可强制结束占用进程，排查端口冲突',
  category: 'devtool',
  keywords: ['port', 'portage', 'netstat', 'listen', 'occupied', 'kill', 'process', '端口', '端口占用', '占用', '监听', '进程', '杀进程', '端口冲突', '端口号'],
  component: () => import('./PortCheckView.vue'),
}
