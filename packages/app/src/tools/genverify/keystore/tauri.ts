import { invoke } from '@tauri-apps/api/core'

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
  return await invoke('keystore_check_java')
}

export async function generateKeystore(input: GenerateInput): Promise<GenerateResult> {
  return await invoke('keystore_generate', input)
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
}

export interface KeystoreListResult {
  path: string
  storeType: string | null
  entries: KeystoreEntry[]
}

export interface ListInput {
  path: string
  alias?: string
  storePassword?: string
}

/** 对应 keytool -list -v，解析所有别名的证书指纹。 */
export async function listKeystore(input: ListInput): Promise<KeystoreListResult> {
  return await invoke('keystore_info_list', { input })
}

export interface KeyHashInput {
  path: string
  alias: string
  storePassword?: string
}

export interface KeyHashResult {
  alias: string
  /** base64(SHA1(证书 DER))，即发布密钥散列 */
  sha1Base64: string
  /** 冒号分隔的 SHA-1 十六进制指纹，便于核对 */
  sha1Hex: string
}

/** 对应 keytool -exportcert | openssl sha1 -binary | openssl base64。 */
export async function getKeyHash(input: KeyHashInput): Promise<KeyHashResult> {
  return await invoke('keystore_key_hash', { input })
}
