/**
 * 批量重命名规则引擎 —— 纯 TS 字符串逻辑，不依赖 Tauri。
 * 文件名拆为 base + ext（ext 不含点），规则列表按顺序 fold 出最终名，
 * 每步中间名记入 steps 供"逐步预览"，末尾统一做冲突检测（Windows 风格不区分大小写）。
 */

export type RuleType = 'replace' | 'insert' | 'sequence' | 'case' | 'remove'

export interface RuleBase {
  /** 唯一 id（crypto.randomUUID()），列表 key 与拖拽标识 */
  id: string
  enabled: boolean
  /** 替换/大小写/删除类变换是否额外作用于扩展名（默认只处理 base name） */
  applyToExtension: boolean
}

export type RenameRule = RuleBase & (
  | { type: 'replace'; find: string; useRegex: boolean; replacement: string; caseSensitive: boolean }
  | { type: 'insert'; text: string; position: 'start' | 'end' | 'index'; index: number; beforeExtension: boolean }
  | { type: 'sequence'; start: number; step: number; pad: number; position: 'prefix' | 'suffix' | 'index'; index: number; template?: string }
  | { type: 'case'; mode: 'upper' | 'lower' | 'title' }
  | { type: 'remove'; mode: 'range' | 'chars' | 'extension'; rangeStart?: number; rangeEnd?: number; chars?: string }
)

/** 文件名/扩展名分离的中间表示，ext 不含点；隐藏文件（.gitignore）与无扩展名文件 ext 为 '' */
export interface NameParts { base: string; ext: string }

export interface RenameFile {
  name: string
  path?: string
  size?: number
  createdMs?: number | null
  modifiedMs?: number | null
}

export const METADATA_TAGS = [
  { key: 'Date_Now', label: '当前日期时间', hint: '本地时间，格式为 YYYY-MM-DD_HH-mm-ss' },
  { key: 'Clipboard', label: '剪贴板文本', hint: '使用读取时的文本；可在预览工具栏重新读取' },
  { key: 'File_Name', label: '原始文件名', hint: '包含扩展名，不受前面的规则影响' },
  { key: 'File_BaseName', label: '原始主文件名', hint: '不含扩展名' },
  { key: 'File_Extension', label: '扩展名', hint: '不含前导点；无扩展名时为空' },
  { key: 'File_FolderName', label: '所在文件夹名称', hint: '只取最后一级文件夹名称' },
  { key: 'File_FilePath', label: '完整文件路径', hint: '含路径分隔符，直接用于文件名会提示无效；可用后续替换规则处理' },
  { key: 'File_Size', label: '文件大小（自动单位）', hint: '例如 1.5 KB，以 1024 为单位换算' },
  { key: 'File_SizeBytes', label: '文件大小（字节）', hint: '整数，不含单位' },
  { key: 'File_SizeKB', label: '文件大小（KB）', hint: '以 1024 换算，最多两位小数，不含单位' },
  { key: 'File_SizeMB', label: '文件大小（MB）', hint: '以 1024 换算，最多两位小数，不含单位' },
  { key: 'File_SizeGB', label: '文件大小（GB）', hint: '以 1024 换算，最多两位小数，不含单位' },
  { key: 'File_DateCreated', label: '文件创建时间', hint: '本地时间，格式为 YYYY-MM-DD_HH-mm-ss' },
  { key: 'File_DateModified', label: '文件修改时间', hint: '本地时间，格式为 YYYY-MM-DD_HH-mm-ss' },
] as const

export interface MetadataOptions { now?: number; clipboard?: string }

const TAG_PATTERN = /\{([A-Za-z][A-Za-z0-9_]*)\}/g

function formatDate(timestamp: number): string {
  const date = new Date(timestamp)
  if (!Number.isFinite(date.getTime())) throw new Error('日期无效')
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}_${pad(date.getHours())}-${pad(date.getMinutes())}-${pad(date.getSeconds())}`
}

/** 按原始文件信息展开一次标签，标签值中的大括号不会再次解析。 */
export function resolveMetadata(text: string, file: RenameFile, options: MetadataOptions = {}): string {
  return text.replace(TAG_PATTERN, (_, key: string) => {
    const tag = METADATA_TAGS.find(tag => tag.key === key)
    if (!tag) throw new Error(`未知元数据标签：{${key}}`)
    const required = <T>(value: T | null | undefined): T => {
      if (value == null) throw new Error(`无法读取${tag.label}`)
      return value
    }
    const parts = splitFileName(file.name)
    const size = (power: number) => String(Number((required(file.size) / 1024 ** power).toFixed(2)))
    switch (key) {
      case 'Date_Now': return formatDate(options.now ?? Date.now())
      case 'Clipboard': return required(options.clipboard)
      case 'File_Name': return file.name
      case 'File_BaseName': return parts.base
      case 'File_Extension': return parts.ext
      case 'File_FilePath': return required(file.path)
      case 'File_FolderName': {
        const path = required(file.path)
        const dir = path.slice(0, Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\')))
        return dir.slice(Math.max(dir.lastIndexOf('/'), dir.lastIndexOf('\\')) + 1)
      }
      case 'File_SizeBytes': return String(required(file.size))
      case 'File_SizeKB': return size(1)
      case 'File_SizeMB': return size(2)
      case 'File_SizeGB': return size(3)
      case 'File_Size': {
        const bytes = required(file.size)
        const power = bytes > 0 ? Math.min(3, Math.floor(Math.log(bytes) / Math.log(1024))) : 0
        return `${size(power)} ${['B', 'KB', 'MB', 'GB'][power]}`
      }
      case 'File_DateCreated': return formatDate(required(file.createdMs))
      case 'File_DateModified': return formatDate(required(file.modifiedMs))
      default: throw new Error(`未知元数据标签：{${key}}`)
    }
  })
}

export interface PreviewStep { ruleId: string; ruleLabel: string; name: string }

export interface PreviewItem {
  originalName: string
  finalName: string
  /** 每条启用规则之后的完整文件名，供逐步预览 */
  steps: PreviewStep[]
  status: 'unchanged' | 'ok' | 'conflict-exists' | 'conflict-duplicate' | 'invalid'
  statusDetail?: string
}

/** Windows 文件名不允许出现的字符 */
const INVALID_CHARS = /[\\/:*?"<>|]/

export function splitFileName(fullName: string): NameParts {
  const dot = fullName.lastIndexOf('.')
  // 首字符的 '.' 是隐藏文件标记（.gitignore），不是扩展名分隔
  if (dot <= 0) return { base: fullName, ext: '' }
  return { base: fullName.slice(0, dot), ext: fullName.slice(dot + 1) }
}

export function joinFileName(parts: NameParts): string {
  return parts.ext ? `${parts.base}.${parts.ext}` : parts.base
}

/** 规则参数校验：返回错误文案，null 表示合法。供预览标记与 RuleCard 红框提示。 */
export function ruleError(rule: RenameRule): string | null {
  switch (rule.type) {
    case 'replace':
      if (!rule.find) return '请填写查找内容'
      if (rule.useRegex) {
        try {
          new RegExp(rule.find, flagsOf(rule.caseSensitive))
        } catch {
          return '正则表达式无效'
        }
      }
      return null
    case 'insert':
      return rule.text ? null : '请填写插入内容'
    case 'remove':
      if (rule.mode === 'range' && (rule.rangeEnd ?? 0) <= (rule.rangeStart ?? 0)) return '结束位置需大于起始位置'
      if (rule.mode === 'chars' && !rule.chars) return '请填写要删除的字符'
      return null
    default:
      return null
  }
}

export function ruleTypeLabel(type: RuleType): string {
  const labels: Record<RuleType, string> = {
    replace: '搜索替换',
    insert: '插入文本',
    sequence: '序号编号',
    case: '大小写',
    remove: '删除',
  }
  return labels[type]
}

export function ruleLabel(rule: RenameRule): string {
  switch (rule.type) {
    case 'replace': return rule.useRegex ? `正则 ${rule.find} → ${rule.replacement}` : `${rule.find} → ${rule.replacement}`
    case 'insert': return `插入 "${rule.text}"`
    case 'case': return rule.mode === 'upper' ? '转大写' : rule.mode === 'lower' ? '转小写' : '首字母大写'
    case 'remove':
      if (rule.mode === 'extension') return '删除扩展名'
      if (rule.mode === 'range') return `删除第 ${rule.rangeStart ?? 0}~${(rule.rangeEnd ?? 0) - 1} 位`
      return `删除字符 "${rule.chars ?? ''}"`
    case 'sequence': {
      const n = String(rule.start)
      return rule.template ? `序号 ${rule.template.replace('{n}', n)}` : `序号 ${n}`
    }
  }
}

function flagsOf(caseSensitive: boolean): string {
  return caseSensitive ? '' : 'i'
}

function titleCase(s: string): string {
  return s.replace(/\S+/g, word => word.charAt(0).toUpperCase() + word.slice(1))
}

function caseTransform(mode: 'upper' | 'lower' | 'title'): (s: string) => string {
  if (mode === 'upper') return s => s.toUpperCase()
  if (mode === 'lower') return s => s.toLowerCase()
  return titleCase
}

function replaceTransform(rule: Extract<RenameRule, { type: 'replace' }>): ((s: string) => string) | null {
  if (ruleError(rule)) return null
  if (rule.useRegex) {
    const re = new RegExp(rule.find, `${flagsOf(rule.caseSensitive)}g`)
    return s => s.replace(re, rule.replacement)
  }
  if (rule.caseSensitive) return s => s.split(rule.find).join(rule.replacement)
  // 不区分大小写：逐段匹配原文本长度后替换，保留未匹配部分原样
  return s => {
    const lower = s.toLowerCase()
    const needle = rule.find.toLowerCase()
    let out = ''
    let i = 0
    while (i < s.length) {
      const at = lower.indexOf(needle, i)
      if (at < 0) {
        out += s.slice(i)
        break
      }
      out += s.slice(i, at) + rule.replacement
      i = at + needle.length
    }
    return out
  }
}

/**
 * 对扩展名应用文本变换。ext 不含点，但用户输入的查找内容可能带前导点（如 ".js"），
 * 先按裸 ext 匹配，未命中再补上点匹配一次，并去掉结果的前导点。
 */
function transformExtension(transform: (s: string) => string, ext: string): string {
  if (!ext) return ext
  const bare = transform(ext)
  if (bare !== ext) return bare
  const dotted = `.${ext}`
  const replaced = transform(dotted)
  return replaced === dotted ? bare : replaced.replace(/^\.+/, '')
}

/** 单条规则应用到文件名的 base/ext 上，不可变，返回新的 parts。insert/sequence 为位置操作，不受 applyToExtension 影响。 */
export function applyRule(rule: RenameRule, parts: NameParts, fileIndex: number, file: RenameFile = { name: joinFileName(parts) }, options: MetadataOptions = {}): NameParts {
  if (!rule.enabled) return parts
  switch (rule.type) {
    case 'replace': {
      const transform = replaceTransform(rule)
      if (!transform) return parts
      const next: NameParts = { base: transform(parts.base), ext: parts.ext }
      if (rule.applyToExtension) next.ext = transformExtension(transform, parts.ext)
      return next
    }
    case 'insert': {
      const text = resolveMetadata(rule.text, file, options)
      if (!text) return parts
      const base = parts.base
      if (rule.position === 'start') return { ...parts, base: text + base }
      if (rule.position === 'index') {
        const at = Math.max(0, Math.min(rule.index, base.length))
        return { ...parts, base: base.slice(0, at) + text + base.slice(at) }
      }
      // end：beforeExtension=true 时插在 base 末尾（点之前），否则插在整个文件名末尾（扩展名之后）
      if (rule.beforeExtension) return { ...parts, base: base + text }
      return { base: joinFileName(parts) + text, ext: '' }
    }
    case 'sequence': {
      const n = rule.start + fileIndex * rule.step
      const num = String(Math.max(0, n)).padStart(Math.max(0, rule.pad), '0')
      const text = rule.template ? rule.template.replace('{n}', num) : num
      if (!text) return parts
      const base = parts.base
      if (rule.position === 'prefix') return { ...parts, base: text + base }
      if (rule.position === 'suffix') return { ...parts, base: base + text }
      const at = Math.max(0, Math.min(rule.index, base.length))
      return { ...parts, base: base.slice(0, at) + text + base.slice(at) }
    }
    case 'case': {
      const transform = caseTransform(rule.mode)
      const next: NameParts = { base: transform(parts.base), ext: parts.ext }
      if (rule.applyToExtension) next.ext = transform(parts.ext)
      return next
    }
    case 'remove': {
      if (rule.mode === 'extension') return { base: parts.base, ext: '' }
      const removeRange = (s: string): string => {
        const start = Math.max(0, Math.min(rule.rangeStart ?? 0, s.length))
        const end = Math.max(start, Math.min(rule.rangeEnd ?? 0, s.length))
        return s.slice(0, start) + s.slice(end)
      }
      const removeChars = (s: string): string => {
        const set = new Set([...(rule.chars ?? '')])
        return [...s].filter(ch => !set.has(ch)).join('')
      }
      const transform = rule.mode === 'range' ? removeRange : removeChars
      const next: NameParts = { base: transform(parts.base), ext: parts.ext }
      if (rule.applyToExtension) next.ext = transform(parts.ext)
      return next
    }
  }
}

/**
 * 规则列表 fold 出全部文件的最终名与冲突状态。
 * 冲突比较按所在文件夹与文件名，不区分大小写（Windows 语义）。
 */
export function previewRename(rules: RenameRule[], files: (string | RenameFile)[], options: MetadataOptions = {}): PreviewItem[] {
  const enabledRules = rules.filter(r => r.enabled)
  const metadata = files.map(file => typeof file === 'string' ? { name: file } : file)
  const context = { ...options, now: options.now ?? Date.now() }

  const items: PreviewItem[] = metadata.map((file, fileIndex) => {
    const originalName = file.name
    let parts = splitFileName(originalName)
    const steps: PreviewStep[] = []
    let metadataError: string | undefined
    for (const rule of enabledRules) {
      try {
        parts = applyRule(rule, parts, fileIndex, file, context)
      } catch (error) {
        metadataError = error instanceof Error ? error.message : String(error)
        break
      }
      steps.push({ ruleId: rule.id, ruleLabel: ruleLabel(rule), name: joinFileName(parts) })
    }
    const finalName = joinFileName(parts)

    let status: PreviewItem['status'] = 'ok'
    let statusDetail: string | undefined
    if (metadataError) {
      status = 'invalid'
      statusDetail = metadataError
    } else if (!parts.base.trim() || !finalName) {
      status = 'invalid'
      statusDetail = '文件名为空'
    } else if (INVALID_CHARS.test(finalName)) {
      status = 'invalid'
      statusDetail = '包含 Windows 不允许的字符 \\ / : * ? " < > |'
    } else if (finalName === originalName) {
      status = 'unchanged'
    }
    // 注意：仅大小写不同时不算 unchanged——Windows 允许就地改大小写，按 ok 处理（后端支持）
    return { originalName, finalName, steps, status, statusDetail }
  })

  // 批内目标名互撞 → 双双标红。unchanged 项执行时会被跳过，不参与撞名
  const byFinal = new Map<string, number[]>()
  const nameKey = (name: string, index: number) => {
    const path = metadata[index].path ?? ''
    const dir = path.slice(0, Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\')) + 1)
    return `${dir}${name}`.toLowerCase()
  }
  items.forEach((item, i) => {
    if (item.status !== 'ok') return
    const key = nameKey(item.finalName, i)
    const group = byFinal.get(key)
    if (group) group.push(i)
    else byFinal.set(key, [i])
  })
  for (const group of byFinal.values()) {
    if (group.length > 1) {
      for (const i of group) {
        const others = group.filter(j => j !== i).map(j => items[j].originalName)
        items[i].status = 'conflict-duplicate'
        items[i].statusDetail = `与批内文件重名: ${others.join(', ')}`
      }
    }
  }

  // 目标名撞上批内其他文件（非自身）的原始名 → 链式冲突，标红阻止
  const originalsByLower = new Map<string, number[]>()
  metadata.forEach((f, i) => {
    const key = nameKey(f.name, i)
    const group = originalsByLower.get(key)
    if (group) group.push(i)
    else originalsByLower.set(key, [i])
  })
  items.forEach((item, i) => {
    if (item.status !== 'ok') return
    const group = originalsByLower.get(nameKey(item.finalName, i))
    const other = group?.find(j => j !== i)
    if (other !== undefined) {
      item.status = 'conflict-exists'
      item.statusDetail = `目标名与批内文件原始名相同: ${metadata[other].name}`
    }
  })

  return items
}
