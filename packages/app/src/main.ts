import { createApp } from 'vue'
import { setPlatform } from '@sbox/tools-core'
import '@sbox/tools-core/theme.css'
import App from './App.vue'
import router from './router'
import { setupLogger } from './logger'
import { tauriPlatform } from './platform-tauri'

setupLogger()

// 注入 Tauri 平台实现（落盘 = 保存对话框 + Rust 写文件）
setPlatform(tauriPlatform)

createApp(App).use(router).mount('#app')
