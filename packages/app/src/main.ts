import { createApp } from 'vue'
import '@qingfeng346/ui-kit/style.css'
import './theme.css'
import App from './App.vue'
import router from './shell/router'
import { setupLogger } from './shell/logger'

setupLogger()

// ui-kit 暗色令牌跟随系统（theme-dark.css 以 html[data-theme="dark"] 生效）
const applyColorScheme = () => {
  document.documentElement.dataset.theme
    = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}
applyColorScheme()
window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', applyColorScheme)

createApp(App).use(router).mount('#app')
