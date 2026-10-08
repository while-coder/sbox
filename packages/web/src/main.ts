import { createApp } from 'vue'
import { setPlatform, webPlatform } from '@sbox/tools-core'
import '@qingfeng346/ui-kit/style.css'
import '@sbox/tools-core/theme.css'
import App from './App.vue'
import router from './router'

// ui-kit 暗色令牌跟随系统（theme-dark.css 以 html[data-theme="dark"] 生效）
const applyColorScheme = () => {
  document.documentElement.dataset.theme
    = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}
applyColorScheme()
window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', applyColorScheme)

// 注入浏览器平台实现（落盘 = 下载）
setPlatform(webPlatform)

createApp(App).use(router).mount('#app')
