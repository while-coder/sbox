import type { ToolDef } from '../../../shell/registry'

/** Keystore 工具：生成 + 指纹查看，双 tab。 */
export const keystoreTool: ToolDef = {
  key: 'keystore-gen',
  label: 'Keystore 工具',
  description: '生成 Android 签名 keystore（PKCS12）与 GitHub Actions secrets；或读取现有 keystore 的 SHA-1 / SHA-256 指纹（微信开放平台、Google Play 等需要）',
  category: 'genverify',
  keywords: ['keystore', 'android', 'apk', 'sign', 'pkcs12', 'jks', '签名', '密钥库', 'sha1', 'sha-1', 'md5', 'sha256', 'fingerprint', '指纹', 'keytool'],
  component: () => import('./KeystoreView.vue'),
}
