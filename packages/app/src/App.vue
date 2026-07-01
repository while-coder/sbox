<script setup lang="ts">
import { useRoute, useRouter } from 'vue-router'
import { computed, onMounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import UpdaterDialog from './UpdaterDialog.vue'
import { loadSettings } from './settings'
import { setupCloseToTray, applyBossKey, watchBossKey } from './system'
import { ensureOverlay } from './tools/screenshot/screenshot'

const route = useRoute()
const router = useRouter()
const isHome = computed(() => route.path === '/')
const isSettings = computed(() => route.path === '/settings')
// 截图覆盖层是独立全屏窗口，不渲染应用外壳、也不重复初始化系统副作用
const isOverlay = computed(() => route.path === '/screenshot-overlay')

onMounted(async () => {
  if (getCurrentWindow().label !== 'main') return
  loadSettings()
  await setupCloseToTray()
  await applyBossKey()
  // 改键 / 开关老板键时自动重新注册
  watchBossKey()
  // 托盘「设置」菜单 → 跳转设置页
  await listen('open-settings', () => router.push('/settings'))
  // 预创建透明常驻的截图覆盖层，后续截图只需切换鼠标穿透，避免窗口动画和冷启动。
  void ensureOverlay()
})
</script>

<template>
  <!-- 截图覆盖层：全屏无外壳 -->
  <router-view v-if="isOverlay" />

  <div v-else class="app">
    <UpdaterDialog />
    <header class="app-bar">
      <button v-if="!isHome" class="back" @click="router.push('/')">← 返回</button>
      <h1 class="title">sbox</h1>
      <button
        v-if="!isSettings"
        class="settings-btn"
        title="设置"
        @click="router.push('/settings')"
      >⚙</button>
    </header>
    <main class="app-main">
      <router-view />
    </main>
  </div>
</template>

<style>
:root {
  --fg: #1a1a1a;
  --fg-muted: #666;
  --bg: #fafafa;
  --card: #fff;
  --border: #e5e5e5;
  --primary: #2d6cdf;
  --primary-hover: #1f56b8;
  --success: #2da44e;
  --danger: #cf222e;
  --radius: 8px;
}
@media (prefers-color-scheme: dark) {
  :root {
    --fg: #e5e5e5;
    --fg-muted: #999;
    --bg: #1a1a1a;
    --card: #242424;
    --border: #333;
  }
}
* { box-sizing: border-box; }
html, body, #app { height: 100%; margin: 0; }
body { font: 14px/1.5 -apple-system, BlinkMacSystemFont, "Segoe UI", "PingFang SC", sans-serif; color: var(--fg); background: var(--bg); }

.app { display: flex; flex-direction: column; height: 100%; }
.app-bar {
  display: flex; align-items: center; gap: 12px;
  padding: 12px 20px;
  border-bottom: 1px solid var(--border);
  background: var(--card);
}
.title { font-size: 16px; font-weight: 600; margin: 0; }
.back {
  background: none; border: none; color: var(--primary); cursor: pointer;
  font-size: 14px; padding: 4px 8px;
}
.back:hover { color: var(--primary-hover); }
.settings-btn {
  margin-left: auto; background: none; border: none; cursor: pointer;
  font-size: 18px; line-height: 1; padding: 4px 6px; color: var(--fg-muted);
}
.settings-btn:hover { color: var(--primary); }
.app-main { flex: 1; overflow: auto; padding: 20px; }

button.btn {
  background: var(--primary); color: #fff; border: none;
  padding: 8px 16px; border-radius: var(--radius); cursor: pointer;
  font-size: 14px; font-weight: 500;
}
button.btn:hover:not(:disabled) { background: var(--primary-hover); }
button.btn:disabled { opacity: 0.5; cursor: not-allowed; }
button.btn-outline {
  background: transparent; color: var(--fg); border: 1px solid var(--border);
}
button.btn-outline:hover:not(:disabled) { border-color: var(--primary); color: var(--primary); }
</style>
