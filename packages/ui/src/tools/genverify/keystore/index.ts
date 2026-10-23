import type { ToolDef } from '../../../shell/registry'

/** Keystore 工具：生成 + 指纹查看，双 tab。 */
export const keystoreTool: ToolDef = {
  key: 'keystore-gen',
  label: 'Keystore 工具',
  description: '生成 Android 签名 keystore（PKCS12）；或查看 keystore / 证书（der/pem/p7b）/ APK、AAB 的证书详情：SHA-1 / SHA-256 指纹与密钥散列（Facebook Android Key Hashes、微信应用签名等需要），无需 JDK',
  category: 'genverify',
  keywords: ['keystore', 'android', 'apk', 'aab', 'sign', 'pkcs12', 'jks', '签名', '密钥库', 'sha1', 'sha-1', 'md5', 'sha256', 'fingerprint', '指纹', 'keytool', 'key hash', 'hash', '散列', '应用签名', '发布密钥', 'exportcert', 'base64', 'facebook', 'der', 'pem', 'crt', 'cer', 'p7b', '证书', 'apksigner', 'deployment_cert', 'play'],
  component: () => import('./KeystoreView.vue'),
}
