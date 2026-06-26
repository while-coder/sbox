import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// 部署到 GitHub Pages 项目页子路径 /sbox/；本地 dev 用 '/'。
export default defineConfig(({ command }) => ({
  base: command === 'build' ? '/sbox/' : '/',
  plugins: [vue()],
  build: {
    target: 'esnext',
  },
}))
