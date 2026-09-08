/**
 * JSON / YAML / TOML 互转与校验（纯前端实现）
 * 思路：任意格式 → 解析为 JS 值 → 序列化为目标格式。
 */
import { load as yamlLoad, dump as yamlDump } from 'js-yaml'
import { parse as tomlParse, stringify as tomlStringify } from 'smol-toml'

export type DataFormat = 'json' | 'yaml' | 'toml'

export const FORMAT_LABELS: Record<DataFormat, string> = {
  json: 'JSON',
  yaml: 'YAML',
  toml: 'TOML',
}

/** 把任意支持的格式文本解析为 JS 值，失败时抛出可读错误。 */
export function parseAny(text: string, format: DataFormat): unknown {
  const trimmed = text.trim()
  if (!trimmed) throw new Error('输入为空')
  switch (format) {
    case 'json':
      return JSON.parse(trimmed)
    case 'yaml':
      return yamlLoad(trimmed)
    case 'toml':
      return tomlParse(trimmed)
  }
}

/** 把 JS 值序列化为目标格式文本。 */
export function stringifyAny(value: unknown, format: DataFormat, indent = 2): string {
  switch (format) {
    case 'json':
      return JSON.stringify(value, null, indent)
    case 'yaml':
      return yamlDump(value, { indent, lineWidth: -1, noRefs: true })
    case 'toml':
      if (value === null || typeof value !== 'object' || Array.isArray(value)) {
        throw new Error('TOML 顶层必须是对象（表），不能是数组或标量')
      }
      return tomlStringify(value as Record<string, unknown>)
  }
}

/** 一步到位：from → to 转换。 */
export function convert(text: string, from: DataFormat, to: DataFormat, indent = 2): string {
  return stringifyAny(parseAny(text, from), to, indent)
}

/**
 * 自动识别输入格式。识别顺序按「专属特征强弱」排列：
 *   JSON（{}/[] 开头）→ TOML（含 key=/[table]）→ JSON（标量）→ YAML（兜底）。
 * 注意 JSON 是 YAML 的子集、纯文本也是合法 YAML 标量，故 YAML 放最后并要求结果为对象/数组。
 */
export function detectFormat(text: string): DataFormat | null {
  const t = text.trim()
  if (!t) return null

  // 1) JSON：以 { 或 [ 开头且能解析
  if (t[0] === '{' || t[0] === '[') {
    try { JSON.parse(t); return 'json' } catch { /* 继续 */ }
  }

  // 2) TOML：存在 key = ... 或 [table] 行，且能解析为非空表
  const looksToml = /^[ \t]*(\[\[?[^\]]+\]\]?|[A-Za-z0-9_."'-]+[ \t]*=)/m.test(t)
  if (looksToml) {
    try {
      const v = tomlParse(t)
      if (v && typeof v === 'object' && Object.keys(v as object).length) return 'toml'
    } catch { /* 继续 */ }
  }

  // 3) JSON：数字 / 带引号字符串等非花括号开头
  try { JSON.parse(t); return 'json' } catch { /* 继续 */ }

  // 4) YAML 兜底：能解析且结果为对象/数组（排除把纯文本误判为标量）
  try {
    const v = yamlLoad(t)
    if (v && typeof v === 'object') return 'yaml'
  } catch { /* 继续 */ }

  return null
}

/** 仅校验输入是否为合法的指定格式，返回值/错误信息。 */
export function validate(text: string, format: DataFormat): { ok: boolean; message: string } {
  try {
    parseAny(text, format)
    return { ok: true, message: `合法的 ${FORMAT_LABELS[format]}` }
  } catch (e: any) {
    return { ok: false, message: String(e?.message || e) }
  }
}
