/**
 * 随机生成器（纯前端，基于 crypto 强随机）
 * - UUID v4
 * - 随机字符串（自定义字符集 / 长度 / 数量）
 * - 密码（按勾选的字符类别组合）
 */

const CHARSET = {
  lower: 'abcdefghijklmnopqrstuvwxyz',
  upper: 'ABCDEFGHIJKLMNOPQRSTUVWXYZ',
  digits: '0123456789',
  symbols: '!@#$%^&*()-_=+[]{};:,.<>?',
}
// 易混淆字符：0/O、1/l/I
const AMBIGUOUS = /[0O1lI]/g

/** 用 crypto 在给定字符集里取 length 个字符，避免取模偏置。 */
function randomFromSet(set: string, length: number): string {
  if (!set) throw new Error('字符集为空')
  const setLen = set.length
  const max = Math.floor(0xffffffff / setLen) * setLen // 拒绝采样上界
  const out: string[] = []
  const buf = new Uint32Array(64)
  while (out.length < length) {
    crypto.getRandomValues(buf)
    for (let i = 0; i < buf.length && out.length < length; i++) {
      if (buf[i] < max) out.push(set[buf[i] % setLen])
    }
  }
  return out.join('')
}

export function uuidV4(): string {
  if (typeof crypto.randomUUID === 'function') return crypto.randomUUID()
  const b = new Uint8Array(16)
  crypto.getRandomValues(b)
  b[6] = (b[6] & 0x0f) | 0x40
  b[8] = (b[8] & 0x3f) | 0x80
  const h = [...b].map(x => x.toString(16).padStart(2, '0'))
  return `${h.slice(0, 4).join('')}-${h.slice(4, 6).join('')}-${h.slice(6, 8).join('')}-${h.slice(8, 10).join('')}-${h.slice(10).join('')}`
}

export function randomUuids(count: number): string[] {
  return Array.from({ length: count }, () => uuidV4())
}

export interface PasswordOptions {
  length: number
  lower: boolean
  upper: boolean
  digits: boolean
  symbols: boolean
  excludeAmbiguous: boolean
}

export function buildCharset(opts: Omit<PasswordOptions, 'length'>): string {
  let set = ''
  if (opts.lower) set += CHARSET.lower
  if (opts.upper) set += CHARSET.upper
  if (opts.digits) set += CHARSET.digits
  if (opts.symbols) set += CHARSET.symbols
  if (opts.excludeAmbiguous) set = set.replace(AMBIGUOUS, '')
  return set
}

export function generatePassword(opts: PasswordOptions): string {
  const set = buildCharset(opts)
  if (!set) throw new Error('请至少选择一种字符类别')
  if (opts.length < 1) throw new Error('长度至少为 1')
  return randomFromSet(set, opts.length)
}

export function generatePasswords(opts: PasswordOptions, count: number): string[] {
  return Array.from({ length: count }, () => generatePassword(opts))
}

/** 任意字符集随机串 */
export function randomString(set: string, length: number, count: number): string[] {
  return Array.from({ length: count }, () => randomFromSet(set, length))
}

/** 粗略的密码强度评估（熵，单位 bit） */
export function entropyBits(charsetSize: number, length: number): number {
  if (charsetSize <= 1 || length <= 0) return 0
  return Math.round(length * Math.log2(charsetSize))
}
