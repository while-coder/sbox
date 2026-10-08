import { createApp } from 'vue'
import { setPlatform } from '@sbox/tools-core'
import '@qingfeng346/ui-kit/style.css'
import '@sbox/tools-core/theme.css'
import App from './App.vue'
import router from './shell/router'
import { setupLogger } from './shell/logger'
import { tauriPlatform } from './platform/platform-tauri'

setupLogger()

// ui-kit 暗色令牌跟随系统（theme-dark.css 以 html[data-theme="dark"] 生效）
const applyColorScheme = () => {
  document.documentElement.dataset.theme
    = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}
applyColorScheme()
window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', applyColorScheme)

// 注入 Tauri 平台实现（落盘 = 保存对话框 + Rust 写文件）
setPlatform(tauriPlatform)

createApp(App).use(router).mount('#app')
