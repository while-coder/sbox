import type { ToolDef } from '../../../registry'

/** 文件校验 (Checksum)。 */
export const checksumTool: ToolDef = {
  key: 'checksum',
  label: '文件校验 (Checksum)',
  description: '计算文件 / 文本哈希并与期望值比对，确认完整性',
  category: 'genverify',
  keywords: ['checksum', 'hash', 'md5', 'sha', 'verify', '校验', '哈希', '完整性'],
  component: () => import('./ChecksumView.vue'),
}
