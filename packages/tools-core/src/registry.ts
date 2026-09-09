/**
 * 共享工具注册表 —— 仅含可跨平台（Web / 桌面）运行的工具。
 * 各工具的定义（ToolDef）放在 tools/<key>/index.ts 中，此处只做汇总；
 * 宿主（web / app）各自组合自己的完整列表：
 *   - web 直接用 WEB_TOOLS
 *   - app 用 [...WEB_TOOLS, ...NATIVE_TOOLS]
 * 路由与首页（HomeView）按传入的列表自动接入。
 */
import type { Component } from 'vue'
import { codecTool } from './tools/encode/codec'
import { jsonTool } from './tools/encode/json'
import { jsonConvertTool } from './tools/encode/json-convert'
import { jwtTool } from './tools/devtool/jwt'
import { timestampTool } from './tools/devtool/timestamp'
import { randomGenTool } from './tools/genverify/random-gen'
import { checksumTool } from './tools/genverify/checksum'
import { qrcodeTool } from './tools/media/qrcode'

export interface ToolDef {
  /** 唯一 key，同时作为路由路径 /<key> */
  key: string
  label: string
  description: string
  /** 所属分类，决定首页分组 */
  category: CategoryKey
  /** 置顶显示：首页在所有分类之前单独成组 */
  pinned?: boolean
  /** 搜索关键词（label/description 之外的别名、英文名等） */
  keywords: string[]
  /** 懒加载组件 */
  component: () => Promise<{ default: Component }>
}

export type CategoryKey = 'encode' | 'devtool' | 'genverify' | 'media' | 'account'

export const CATEGORIES: { key: CategoryKey; label: string }[] = [
  { key: 'encode', label: '编解码 / 数据' },
  { key: 'devtool', label: '开发辅助' },
  { key: 'genverify', label: '生成 / 校验' },
  { key: 'media', label: '图像 / 媒体' },
  { key: 'account', label: '账号 / 设备' },
]

/** 可上线（纯前端 / 平台适配）的工具，Web 与桌面共用。 */
export const WEB_TOOLS: ToolDef[] = [
  codecTool,
  jsonTool,
  jsonConvertTool,
  jwtTool,
  timestampTool,
  randomGenTool,
  checksumTool,
  qrcodeTool,
]

/** 按分类分组，保持 CATEGORIES 的顺序，空分类省略。 */
export function toolsByCategory(tools: ToolDef[]): { key: CategoryKey; label: string; tools: ToolDef[] }[] {
  return CATEGORIES
    .map(c => ({ ...c, tools: tools.filter(t => t.category === c.key) }))
    .filter(g => g.tools.length > 0)
}

/** 导航分组：pinned 工具（如本机信息）单独成组置顶，其余按分类分组。首页与菜单共用。 */
export function toolNavGroups(tools: ToolDef[]): { key: string; label: string; tools: ToolDef[] }[] {
  const pinned = tools.filter(t => t.pinned)
  const grouped = toolsByCategory(tools.filter(t => !t.pinned))
  if (pinned.length) grouped.unshift({ key: 'pinned', label: '常用', tools: pinned })
  return grouped
}

/** 模糊搜索：匹配 label / description / keywords（大小写不敏感）。 */
export function searchTools(tools: ToolDef[], query: string): ToolDef[] {
  const q = query.trim().toLowerCase()
  if (!q) return tools
  return tools.filter(t => {
    const haystack = [t.label, t.description, ...t.keywords].join(' ').toLowerCase()
    return q.split(/\s+/).every(term => haystack.includes(term))
  })
}
