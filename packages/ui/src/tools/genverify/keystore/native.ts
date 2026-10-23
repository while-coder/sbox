import { native } from '../../../platform/native'

export interface JavaCheck {
  available: boolean
  path: string | null
}

export interface DName {
  commonName: string
  organizationalUnit?: string
  organization?: string
  locality?: string
  state?: string
  country?: string
}

export interface GenerateInput {
  path: string
  alias: string
  storePassword: string
  keyPassword: string
  validityDays: number
  dname: DName
}

export interface GenerateResult {
  path: string
  base64: string
  fingerprint_sha256: string
  fingerprint_sha1: string
}

export async function checkJava(): Promise<JavaCheck> {
  return native.tools.keystore.checkJava()
}

export async function generateKeystore(input: GenerateInput): Promise<GenerateResult> {
  return native.tools.keystore.generateKeystore(input)
}

export interface KeystoreEntry {
  alias: string
  entryType: string | null
  creationDate: string | null
  owner: string | null
  issuer: string | null
  serialNumber: string | null
  validFrom: string | null
  validUntil: string | null
  signatureAlgorithm: string | null
  keyAlgorithm: string | null
  fingerprintMd5: string
  fingerprintSha1: string
  fingerprintSha256: string
  /** 密钥散列：base64(SHA1(证书 DER))，即 Facebook「Android Key Hashes」/微信「应用签名」 */
  sha1Base64: string | null
}

export interface FileInfoInput {
  path: string
  /** 仅 keystore 有效：留空列出所有别名 */
  alias?: string
  storePassword?: string
}

export interface FileInfoResult {
  /** 文件类型：keystore / cert / apk */
  kind: 'keystore' | 'cert' | 'apk'
  path: string
  storeType: string | null
  entries: KeystoreEntry[]
}

/**
 * 读取签名文件详情（统一入口）：
 * - keystore 走 keytool -list -v；
 * - 裸证书（.der/.pem/.crt/.cer/.p7b）与 APK/AAB 走纯 Rust 解析，无需 JDK。
 */
export async function fileInfo(input: FileInfoInput): Promise<FileInfoResult> {
  return native.tools.keystore.fileInfo(input)
}
