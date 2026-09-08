import type { ToolDef } from '../../../shell/registry'

/** Keystore 生成。 */
export const keystoreGenTool: ToolDef = {
  key: 'keystore-gen',
  label: 'Keystore 生成',
  description: '生成 Android APK 签名 keystore（PKCS12），自动给出 GitHub Actions 所需的 4 个 secret',
  category: 'genverify',
  keywords: ['keystore', 'android', 'apk', 'sign', 'pkcs12', 'jks', '签名', '密钥库'],
  component: () => import('./KeystoreGenView.vue'),
}
