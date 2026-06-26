/**
 * 共享工具注册表 —— 仅含可跨平台（Web / 桌面）运行的工具。
 * 宿主（web / app）各自组合自己的完整列表：
 *   - web 直接用 WEB_TOOLS
 *   - app 用 [...WEB_TOOLS, ...NATIVE_TOOLS]
 * 路由与首页（HomeView）按传入的列表自动接入。
 */
import type { Component } from 'vue'

export interface ToolDef {
  /** 唯一 key，同时作为路由路径 /<key> */
  key: string
  label: string
  description: string
  /** 所属分类，决定首页分组 */
  category: CategoryKey
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
  {
    key: 'codec',
    label: '编解码工具',
    description: 'Base64 / URL / Hex / HTML / Unicode / JSON 转换，文件与文本的 MD5 / SHA 哈希',
    category: 'encode',
    keywords: ['base64', 'url', 'hex', 'html', 'unicode', 'json', 'md5', 'sha', 'hash', 'encode', 'decode', '编码', '解码', '哈希', '文件'],
    component: () => import('./tools/codec/CodecView.vue'),
  },
  {
    key: 'json',
    label: 'JSON 格式化 / 查看器',
    description: '美化、压缩、按键排序，树状查看并复制单条属性的值或路径',
    category: 'encode',
    keywords: ['json', 'format', 'beautify', 'minify', 'tree', '美化', '压缩', '格式化', '排序', '路径'],
    component: () => import('./tools/json/JsonView.vue'),
  },
  {
    key: 'json-convert',
    label: 'JSON / YAML / TOML 互转',
    description: '三种配置格式互转与格式化校验，源格式可自动检测',
    category: 'encode',
    keywords: ['json', 'yaml', 'yml', 'toml', 'convert', '转换', '格式化', 'format', 'validate', '校验', '自动检测'],
    component: () => import('./tools/json-convert/JsonConvertView.vue'),
  },
  {
    key: 'jwt',
    label: 'JWT 解码',
    description: '解析 JWT 的 header 与 payload，时间声明转可读时间（不校验签名）',
    category: 'devtool',
    keywords: ['jwt', 'token', 'jws', 'bearer', '解码', 'decode', 'claims'],
    component: () => import('./tools/jwt/JwtView.vue'),
  },
  {
    key: 'timestamp',
    label: '时间戳 / 时区转换',
    description: 'Unix 时间戳与日期互转、相对时间与时区换算',
    category: 'devtool',
    keywords: ['timestamp', 'unix', 'epoch', 'date', 'timezone', '时间戳', '时区', '日期'],
    component: () => import('./tools/timestamp/TimestampView.vue'),
  },
  {
    key: 'random-gen',
    label: '随机生成器',
    description: '基于强随机生成密码、UUID 与随机字符串',
    category: 'genverify',
    keywords: ['random', 'password', 'uuid', 'guid', 'secret', '密码', '随机', '生成'],
    component: () => import('./tools/random-gen/RandomGenView.vue'),
  },
  {
    key: 'checksum',
    label: '文件校验 (Checksum)',
    description: '计算文件 / 文本哈希并与期望值比对，确认完整性',
    category: 'genverify',
    keywords: ['checksum', 'hash', 'md5', 'sha', 'verify', '校验', '哈希', '完整性'],
    component: () => import('./tools/checksum/ChecksumView.vue'),
  },
  {
    key: 'qrcode',
    label: '二维码生成 / 识别',
    description: '生成二维码，或识别图片中的二维码（支持粘贴、拖拽、读取剪贴板）',
    category: 'media',
    keywords: ['qrcode', 'qr', '二维码', 'scan', 'decode', 'generate', '识别', '生成', '扫码', '剪贴板'],
    component: () => import('./tools/qrcode/QrView.vue'),
  },
  {
    key: 'image-convert',
    label: '图片格式转换',
    description: 'png/jpeg/webp/avif/gif/bmp/ico/tiff 等互转，支持 HEIC 与 SVG 矢量输入；批量转换或单图编辑（旋转 / 裁剪 / 改尺寸）',
    category: 'media',
    keywords: ['image', 'convert', 'format', 'heic', 'heif', 'avif', 'svg', 'ico', 'webp', 'png', 'jpeg', 'tiff', 'bmp', 'gif', '图片', '转换', '格式', '缩放', '批量', '旋转', '裁剪', '单图', 'rotate', 'crop', 'resize'],
    component: () => import('./tools/image-convert/ImageConvertView.vue'),
  },
]

/** 按分类分组，保持 CATEGORIES 的顺序，空分类省略。 */
export function toolsByCategory(tools: ToolDef[]): { key: CategoryKey; label: string; tools: ToolDef[] }[] {
  return CATEGORIES
    .map(c => ({ ...c, tools: tools.filter(t => t.category === c.key) }))
    .filter(g => g.tools.length > 0)
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
