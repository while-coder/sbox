/**
 * 工具注册表 —— 单一数据源。
 * 新增工具：在此追加一条，路由（router.ts）与首页（HomeView）会自动接入。
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

export type CategoryKey = 'encode' | 'devtool' | 'genverify' | 'account'

export const CATEGORIES: { key: CategoryKey; label: string }[] = [
  { key: 'encode', label: '编解码 / 数据' },
  { key: 'devtool', label: '开发辅助' },
  { key: 'genverify', label: '生成 / 校验' },
  { key: 'account', label: '账号 / 设备' },
]

export const TOOLS: ToolDef[] = [
  {
    key: 'codec',
    label: '编解码工具',
    description: 'Base64 / URL / Hex / HTML / Unicode / JSON 转换，文件与文本的 MD5 / SHA 哈希',
    category: 'encode',
    keywords: ['base64', 'url', 'hex', 'html', 'unicode', 'json', 'md5', 'sha', 'hash', 'encode', 'decode', '编码', '解码', '哈希', '文件'],
    component: () => import('./codec/CodecView.vue'),
  },
  {
    key: 'json',
    label: 'JSON 格式化 / 查看器',
    description: '美化、压缩、按键排序，树状查看并复制单条属性的值或路径',
    category: 'encode',
    keywords: ['json', 'format', 'beautify', 'minify', 'tree', '美化', '压缩', '格式化', '排序', '路径'],
    component: () => import('./json/JsonView.vue'),
  },
  {
    key: 'json-convert',
    label: 'JSON / YAML / TOML 互转',
    description: '三种配置格式互转与格式化校验，源格式可自动检测',
    category: 'encode',
    keywords: ['json', 'yaml', 'yml', 'toml', 'convert', '转换', '格式化', 'format', 'validate', '校验', '自动检测'],
    component: () => import('./json-convert/JsonConvertView.vue'),
  },
  {
    key: 'jwt',
    label: 'JWT 解码',
    description: '解析 JWT 的 header 与 payload，时间声明转可读时间（不校验签名）',
    category: 'devtool',
    keywords: ['jwt', 'token', 'jws', 'bearer', '解码', 'decode', 'claims'],
    component: () => import('./jwt/JwtView.vue'),
  },
  {
    key: 'timestamp',
    label: '时间戳 / 时区转换',
    description: 'Unix 时间戳与日期互转、相对时间与时区换算',
    category: 'devtool',
    keywords: ['timestamp', 'unix', 'epoch', 'date', 'timezone', '时间戳', '时区', '日期'],
    component: () => import('./timestamp/TimestampView.vue'),
  },
  {
    key: 'random-gen',
    label: '随机生成器',
    description: '基于强随机生成密码、UUID 与随机字符串',
    category: 'genverify',
    keywords: ['random', 'password', 'uuid', 'guid', 'secret', '密码', '随机', '生成'],
    component: () => import('./random-gen/RandomGenView.vue'),
  },
  {
    key: 'checksum',
    label: '文件校验 (Checksum)',
    description: '计算文件 / 文本哈希并与期望值比对，确认完整性',
    category: 'genverify',
    keywords: ['checksum', 'hash', 'md5', 'sha', 'verify', '校验', '哈希', '完整性'],
    component: () => import('./checksum/ChecksumView.vue'),
  },
  {
    key: 'keystore-gen',
    label: 'Keystore 生成',
    description: '生成 Android APK 签名 keystore（PKCS12），自动给出 GitHub Actions 所需的 4 个 secret',
    category: 'genverify',
    keywords: ['keystore', 'android', 'apk', 'sign', 'pkcs12', 'jks', '签名', '密钥库'],
    component: () => import('./keystore-gen/KeystoreGenView.vue'),
  },
  {
    key: 'xiaoai-login',
    label: '小爱登录',
    description: '登录小米账号，导出 sbot channel.xiaoai 所需的 userId / passToken / loginDeviceId / deviceName',
    category: 'account',
    keywords: ['xiaoai', 'xiaomi', '小爱', '小米', 'login', 'cookie', '登录', 'passtoken'],
    component: () => import('./xiaoai-login/XiaoaiLoginView.vue'),
  },
]

/** 按分类分组，保持 CATEGORIES 的顺序，空分类省略。 */
export function toolsByCategory(): { key: CategoryKey; label: string; tools: ToolDef[] }[] {
  return CATEGORIES
    .map(c => ({ ...c, tools: TOOLS.filter(t => t.category === c.key) }))
    .filter(g => g.tools.length > 0)
}

/** 模糊搜索：匹配 label / description / keywords（大小写不敏感）。 */
export function searchTools(query: string): ToolDef[] {
  const q = query.trim().toLowerCase()
  if (!q) return TOOLS
  return TOOLS.filter(t => {
    const haystack = [t.label, t.description, ...t.keywords].join(' ').toLowerCase()
    return q.split(/\s+/).every(term => haystack.includes(term))
  })
}
