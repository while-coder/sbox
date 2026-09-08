import type { ToolDef } from '../../../registry'

/** JWT 解码。 */
export const jwtTool: ToolDef = {
  key: 'jwt',
  label: 'JWT 解码',
  description: '解析 JWT 的 header 与 payload，时间声明转可读时间（不校验签名）',
  category: 'devtool',
  keywords: ['jwt', 'token', 'jws', 'bearer', '解码', 'decode', 'claims'],
  component: () => import('./JwtView.vue'),
}
