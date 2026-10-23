import type { ToolDef } from '../../../shell/registry'

/** 本机信息。 */
export const systemInfoTool: ToolDef = {
  key: 'system-info',
  label: '本机信息',
  description: '查看操作系统版本、主板 BIOS、CPU / 内存 / 显卡规格、磁盘与 IP 地址等本机软硬件信息',
  category: 'account',
  pinned: true,
  keywords: ['system', 'info', 'hardware', 'cpu', 'gpu', 'memory', 'disk', 'ip', 'bios', 'os', 'version', '设备', '系统', '版本', '主板', '显卡', '内存', '硬盘', '存储', '地址', '网络', '配置', '电脑信息'],
  component: () => import('./SystemInfoView.vue'),
}
