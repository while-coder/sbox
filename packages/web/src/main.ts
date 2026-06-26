import { createApp } from 'vue'
import { setPlatform, webPlatform } from '@sbox/tools-core'
import '@sbox/tools-core/theme.css'
import App from './App.vue'
import router from './router'

// 注入浏览器平台实现（落盘 = 下载）
setPlatform(webPlatform)

createApp(App).use(router).mount('#app')
