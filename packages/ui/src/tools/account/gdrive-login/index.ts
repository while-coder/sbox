import type { ToolDef } from '../../../shell/registry'

/** Google Drive 登录。 */
export const gdriveLoginTool: ToolDef = {
  key: 'gdrive-login',
  label: 'Google Drive 登录',
  description: '浏览器 OAuth 登录，导出 Google auth 格式的授权文件（clientId / clientSecret / refreshToken）',
  category: 'account',
  keywords: ['google', 'drive', 'gdrive', 'oauth', 'refresh token', '登录', '谷歌', '云盘', '授权'],
  component: () => import('./GdriveLoginView.vue'),
}
