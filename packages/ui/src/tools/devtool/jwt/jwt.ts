/**
 * JWT 解码（纯前端，仅解析、不校验签名）
 * 结构：header.payload.signature，前两段为 base64url 编码的 JSON。
 */

function base64UrlToString(seg: string): string {
  const pad = seg.length % 4 === 0 ? 0 : 4 - (seg.length % 4)
  const b64 = seg.replace(/-/g, '+').replace(/_/g, '/') + '='.repeat(pad)
  const binary = atob(b64)
  const bytes = new Uint8Array(binary.length)
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i)
  return new TextDecoder().decode(bytes)
}

export interface DecodedJwt {
  header: Record<string, unknown>
  payload: Record<string, unknown>
  signature: string
  /** payload 中的时间类声明，转成可读时间 */
  claims: { key: string; label: string; date: string; expired?: boolean }[]
}

const TIME_CLAIMS: Record<string, string> = {
  exp: '过期时间 (exp)',
  iat: '签发时间 (iat)',
  nbf: '生效时间 (nbf)',
  auth_time: '认证时间 (auth_time)',
}

export function decodeJwt(token: string): DecodedJwt {
  const t = token.trim()
  const parts = t.split('.')
  if (parts.length < 2 || parts.length > 3) {
    throw new Error('不是合法的 JWT（应为 header.payload.signature 三段）')
  }
  let header: Record<string, unknown>
  let payload: Record<string, unknown>
  try {
    header = JSON.parse(base64UrlToString(parts[0]))
  } catch {
    throw new Error('header 解析失败')
  }
  try {
    payload = JSON.parse(base64UrlToString(parts[1]))
  } catch {
    throw new Error('payload 解析失败')
  }

  const nowSec = Math.floor(Date.now() / 1000)
  const claims: DecodedJwt['claims'] = []
  for (const [key, label] of Object.entries(TIME_CLAIMS)) {
    const v = payload[key]
    if (typeof v === 'number') {
      claims.push({
        key,
        label,
        date: new Date(v * 1000).toLocaleString(),
        expired: key === 'exp' ? v < nowSec : undefined,
      })
    }
  }

  return {
    header,
    payload,
    signature: parts[2] ?? '',
    claims,
  }
}
