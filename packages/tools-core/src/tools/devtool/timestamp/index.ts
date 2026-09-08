import type { ToolDef } from '../../../registry'

/** 时间戳 / 时区转换。 */
export const timestampTool: ToolDef = {
  key: 'timestamp',
  label: '时间戳 / 时区转换',
  description: 'Unix 时间戳与日期互转、相对时间与时区换算',
  category: 'devtool',
  keywords: ['timestamp', 'unix', 'epoch', 'date', 'timezone', '时间戳', '时区', '日期'],
  component: () => import('./TimestampView.vue'),
}
