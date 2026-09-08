import type { ToolDef } from '../../../shell/registry'

/** SSH 密钥生成。 */
export const sshKeygenTool: ToolDef = {
  key: 'ssh-keygen',
  label: 'SSH 密钥生成',
  description: '生成 Ed25519 或 RSA 4096 私钥和公钥，可选密码保护，并给出 OpenSSH SHA-256 指纹',
  category: 'genverify',
  keywords: ['ssh', 'key', 'keygen', 'ed25519', 'rsa', 'rsa4096', 'private key', 'public key', 'fingerprint', '密钥', '私钥', '公钥', '证书', '指纹'],
  component: () => import('./SshKeygenView.vue'),
}
