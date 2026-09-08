import type { ToolDef } from '../../../registry'

/** 随机生成器。 */
export const randomGenTool: ToolDef = {
  key: 'random-gen',
  label: '随机生成器',
  description: '基于强随机生成密码、UUID 与随机字符串',
  category: 'genverify',
  keywords: ['random', 'password', 'uuid', 'guid', 'secret', '密码', '随机', '生成'],
  component: () => import('./RandomGenView.vue'),
}
