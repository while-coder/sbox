import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// 部署到 GitHub Pages 子路径 /sbox/app/（站点根由文档站占用）；本地 dev 用 '/'。
export default defineConfig(({ command }) => ({
  base: command === 'build' ? '/sbox/app/' : '/',
  plugins: [vue()],
  build: {
    target: 'esnext',
  },
}))
