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

/** 单条规则应用到文件名的 base/ext 上，不可变，返回新的 parts。insert/sequence 为位置操作，不受 applyToExtension 影响。 */
export function applyRule(rule: RenameRule, parts: NameParts, fileIndex: number): NameParts {
  if (!rule.enabled) return parts
  switch (rule.type) {
    case 'replace': {
      const transform = replaceTransform(rule)
      if (!transform) return parts
      const next: NameParts = { base: transform(parts.base), ext: parts.ext }
      if (rule.applyToExtension) next.ext = transform(parts.ext)
      return next
    }
    case 'insert': {
      const text = rule.text
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
 * files 为文件名（不含路径）列表；冲突比较一律不区分大小写（Windows 语义）。
 */
export function previewRename(rules: RenameRule[], files: string[]): PreviewItem[] {
  const enabledRules = rules.filter(r => r.enabled)

  const items: PreviewItem[] = files.map((originalName, fileIndex) => {
    let parts = splitFileName(originalName)
    const steps: PreviewStep[] = []
    for (const rule of enabledRules) {
      parts = applyRule(rule, parts, fileIndex)
      steps.push({ ruleId: rule.id, ruleLabel: ruleLabel(rule), name: joinFileName(parts) })
    }
    const finalName = joinFileName(parts)

    let status: PreviewItem['status'] = 'ok'
    let statusDetail: string | undefined
    if (!parts.base.trim() || !finalName) {
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
  items.forEach((item, i) => {
    if (item.status !== 'ok') return
    const key = item.finalName.toLowerCase()
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
  files.forEach((f, i) => {
    const key = f.toLowerCase()
    const group = originalsByLower.get(key)
    if (group) group.push(i)
    else originalsByLower.set(key, [i])
  })
  items.forEach((item, i) => {
    if (item.status !== 'ok') return
    const group = originalsByLower.get(item.finalName.toLowerCase())
    const other = group?.find(j => j !== i)
    if (other !== undefined) {
      item.status = 'conflict-exists'
      item.statusDetail = `目标名与批内文件原始名相同: ${files[other]}`
    }
  })

  return items
}
