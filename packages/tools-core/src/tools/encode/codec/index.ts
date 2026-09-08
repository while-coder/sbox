import type { ToolDef } from '../../../registry'

/** 编解码工具：Base64 / URL / Hex / HTML / Unicode / JSON 转换与哈希。 */
export const codecTool: ToolDef = {
  key: 'codec',
  label: '编解码工具',
  description: 'Base64 / URL / Hex / HTML / Unicode / JSON 转换，文件与文本的 MD5 / SHA 哈希',
  category: 'encode',
  keywords: ['base64', 'url', 'hex', 'html', 'unicode', 'json', 'md5', 'sha', 'hash', 'encode', 'decode', '编码', '解码', '哈希', '文件'],
  component: () => import('./CodecView.vue'),
}
