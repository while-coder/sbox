/**
 * 桌面端工具注册表 = 共享工具（@sbox/tools-core 的 WEB_TOOLS）+ 仅桌面可用的 native 工具。
 * native 工具依赖本机能力（全屏截图 / 本机 Java / 绕 CORS 的 HTTP / OAuth 本地回调），无法上 Web。
 */
import { WEB_TOOLS, type ToolDef } from '@sbox/tools-core'

/** 仅桌面（Tauri）可用的工具。 */
export const NATIVE_TOOLS: ToolDef[] = [
  {
    key: 'system-info',
    label: '本机信息',
    description: '查看操作系统版本、主板 BIOS、CPU / 内存 / 显卡规格、磁盘与 IP 地址等本机软硬件信息',
    category: 'account',
    pinned: true,
    keywords: ['system', 'info', 'hardware', 'cpu', 'gpu', 'memory', 'disk', 'ip', 'bios', 'os', 'version', '设备', '系统', '版本', '主板', '显卡', '内存', '硬盘', '存储', '地址', '网络', '配置', '电脑信息'],
    component: () => import('./tools/system-info/SystemInfoView.vue'),
  },
  {
    key: 'file-locks',
    label: '文件占用检查',
    description: '检查文件或文件夹被哪些 Windows 进程占用，便于处理删除、重命名失败',
    category: 'devtool',
    keywords: ['file lock', 'locked file', 'handle', 'process', 'occupy', '占用', '文件占用', '文件夹占用', '进程', '删除失败', '无法删除', '重命名'],
    component: () => import('./tools/file-locks/FileLocksView.vue'),
  },
  {
    key: 'env-vars',
    label: '环境变量 / Hosts',
    description: '查看和编辑 Windows 用户/系统环境变量（系统变量需管理员权限）与 hosts 文件，修改后自动广播生效',
    category: 'devtool',
    keywords: ['env', 'environment', 'variable', 'path', 'registry', 'hosts', '域名解析', '环境变量', '系统变量', '用户变量', '变量', '注册表'],
    component: () => import('./tools/env-vars/EnvVarsView.vue'),
  },
  {
    key: 'screenshot',
    label: '截图',
    description: '全屏框选截图，保存 / 复制到剪贴板 / 识别二维码，支持全局快捷键',
    category: 'media',
    keywords: ['screenshot', 'capture', 'snip', '截图', '截屏', '框选', '快捷键'],
    component: () => import('./tools/screenshot/ScreenshotView.vue'),
  },
  {
    key: 'keystore-gen',
    label: 'Keystore 生成',
    description: '生成 Android APK 签名 keystore（PKCS12），自动给出 GitHub Actions 所需的 4 个 secret',
    category: 'genverify',
    keywords: ['keystore', 'android', 'apk', 'sign', 'pkcs12', 'jks', '签名', '密钥库'],
    component: () => import('./tools/keystore-gen/KeystoreGenView.vue'),
  },
  {
    key: 'ssh-keygen',
    label: 'SSH 密钥生成',
    description: '生成 Ed25519 或 RSA 4096 私钥和公钥，可选密码保护，并给出 OpenSSH SHA-256 指纹',
    category: 'genverify',
    keywords: ['ssh', 'key', 'keygen', 'ed25519', 'rsa', 'rsa4096', 'private key', 'public key', 'fingerprint', '密钥', '私钥', '公钥', '证书', '指纹'],
    component: () => import('./tools/ssh-keygen/SshKeygenView.vue'),
  },
  {
    key: 'translator',
    label: '网页翻译',
    description: '在应用内切换使用 Google、百度、Bing、有道、DeepL 和混元翻译',
    category: 'devtool',
    keywords: ['translate', 'translator', 'google translate', 'baidu fanyi', 'bing translator', 'youdao', 'deepl', 'tencent', 'hunyuan', '翻译', '谷歌翻译', '百度翻译', '必应翻译', '有道翻译', '混元翻译'],
    component: () => import('./tools/translator/TranslatorView.vue'),
  },
  {
    key: 'xiaoai-login',
    label: '小爱登录',
    description: '登录小米账号，导出 userId / passToken / loginDeviceId / deviceName',
    category: 'account',
    keywords: ['xiaoai', 'xiaomi', '小爱', '小米', 'login', 'cookie', '登录', 'passtoken'],
    component: () => import('./tools/xiaoai-login/XiaoaiLoginView.vue'),
  },
  {
    key: 'gdrive-login',
    label: 'Google Drive 登录',
    description: '浏览器 OAuth 登录，导出 Google auth 格式的授权文件（clientId / clientSecret / refreshToken）',
    category: 'account',
    keywords: ['google', 'drive', 'gdrive', 'oauth', 'refresh token', '登录', '谷歌', '云盘', '授权'],
    component: () => import('./tools/gdrive-login/GdriveLoginView.vue'),
  },
]

/** 桌面端完整工具列表。 */
export const ALL_TOOLS: ToolDef[] = [...WEB_TOOLS, ...NATIVE_TOOLS]
