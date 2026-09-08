import type { ToolDef } from '../../../shell/registry'

/** 小爱登录。 */
export const xiaoaiLoginTool: ToolDef = {
  key: 'xiaoai-login',
  label: '小爱登录',
  description: '登录小米账号，导出 userId / passToken / loginDeviceId / deviceName',
  category: 'account',
  keywords: ['xiaoai', 'xiaomi', '小爱', '小米', 'login', 'cookie', '登录', 'passtoken'],
  component: () => import('./XiaoaiLoginView.vue'),
}
