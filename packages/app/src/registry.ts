/**
 * 桌面端工具注册表 = 共享工具（@sbox/tools-core 的 WEB_TOOLS）+ 仅桌面可用的 native 工具。
 * native 工具依赖本机能力（全屏截图 / 本机 Java / 绕 CORS 的 HTTP / OAuth 本地回调），无法上 Web。
 */
import { WEB_TOOLS, type ToolDef } from '@sbox/tools-core'

/** 仅桌面（Tauri）可用的工具。 */
export const NATIVE_TOOLS: ToolDef[] = [
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
    key: 'xiaoai-login',
    label: '小爱登录',
    description: '登录小米账号，导出 sbot channel.xiaoai 所需的 userId / passToken / loginDeviceId / deviceName',
    category: 'account',
    keywords: ['xiaoai', 'xiaomi', '小爱', '小米', 'login', 'cookie', '登录', 'passtoken'],
    component: () => import('./tools/xiaoai-login/XiaoaiLoginView.vue'),
  },
  {
    key: 'gdrive-login',
    label: 'Google Drive 登录',
    description: '浏览器 OAuth 登录，导出 sbot wiki.gdrive 所需的 clientId / clientSecret / refreshToken',
    category: 'account',
    keywords: ['google', 'drive', 'gdrive', 'oauth', 'refresh token', '登录', '谷歌', '云盘', '授权'],
    component: () => import('./tools/gdrive-login/GdriveLoginView.vue'),
  },
]

/** 桌面端完整工具列表。 */
export const ALL_TOOLS: ToolDef[] = [...WEB_TOOLS, ...NATIVE_TOOLS]
