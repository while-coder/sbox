import type { ToolDef } from '../../../registry'

/** JSON / YAML / TOML 互转。 */
export const jsonConvertTool: ToolDef = {
  key: 'json-convert',
  label: 'JSON / YAML / TOML 互转',
  description: '三种配置格式互转与格式化校验，源格式可自动检测',
  category: 'encode',
  keywords: ['json', 'yaml', 'yml', 'toml', 'convert', '转换', '格式化', 'format', 'validate', '校验', '自动检测'],
  component: () => import('./JsonConvertView.vue'),
}
