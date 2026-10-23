import type { ToolDef } from '../../../shell/registry'

/** 环境变量 / Hosts。 */
export const envVarsTool: ToolDef = {
  key: 'env-vars',
  label: '环境变量 / Hosts',
  description: '查看和编辑 Windows 用户/系统环境变量（系统变量需管理员权限）与 hosts 文件，修改后自动广播生效',
  category: 'devtool',
  keywords: ['env', 'environment', 'variable', 'path', 'registry', 'hosts', '域名解析', '环境变量', '系统变量', '用户变量', '变量', '注册表'],
  component: () => import('./EnvVarsView.vue'),
}
