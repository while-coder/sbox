<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch, type WatchStopHandle } from 'vue'
import { RouterLink, useRoute, useRouter } from 'vue-router'
import {
  AppWindow, Braces, Clock3, FileKey2, FileSearch2, FolderKey, Hash,
  Home, Image, KeyRound, Menu, MonitorUp, QrCode, ScanText, Settings, ShieldCheck,
  Shuffle, X, type LucideIcon,
} from 'lucide-vue-next'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { UpdaterDialog } from '@while-coder/tauri-updater-vue'
import { toolsByCategory } from '@sbox/tools-core'
import { ALL_TOOLS } from './registry'
import { loadSettings } from './settings'
import { setupCloseToTray, applyBossKey, watchBossKey, refreshAutostart } from './system'
import { ensureOverlay } from './tools/screenshot/screenshot'

const route = useRoute()
const router = useRouter()
const sidebarOpen = ref(false)
const isOverlay = computed(() => route.path === '/screenshot-overlay')
const toolGroups = computed(() => toolsByCategory(ALL_TOOLS))
let stopShortcutWatch: WatchStopHandle | undefined
let unlistenOpenSettings: (() => void) | undefined

const toolIcons: Record<string, LucideIcon> = {
  'file-locks': FileSearch2, screenshot: MonitorUp, 'keystore-gen': FolderKey,
  'ssh-keygen': KeyRound, translator: ScanText, 'xiaoai-login': AppWindow,
  'gdrive-login': AppWindow, codec: Braces, json: Braces, 'json-convert': Shuffle,
  jwt: ShieldCheck, timestamp: Clock3, 'random-gen': Shuffle, checksum: Hash,
  qrcode: QrCode, 'image-convert': Image,
}

function iconFor(toolKey: string): LucideIcon {
  return toolIcons[toolKey] ?? FileKey2
}

function closeSidebar() {
  sidebarOpen.value = false
}

watch(() => route.fullPath, closeSidebar)

onMounted(async () => {
  if (getCurrentWindow().label !== 'main') return
  // 尽早创建并定尺寸隐藏截图覆盖层，避免第一次截图临时创建默认大小的窗口。
  void ensureOverlay().catch((e) => console.error('初始化截图覆盖层失败：', e))
  loadSettings()
  void refreshAutostart()
  await setupCloseToTray()
  await applyBossKey()
  stopShortcutWatch = watchBossKey()
  unlistenOpenSettings = await listen('open-settings', () => router.push('/settings'))
})

onUnmounted(() => {
  stopShortcutWatch?.()
  unlistenOpenSettings?.()
})
</script>

<template>
  <!-- 截图覆盖层：全屏无外壳 -->
  <router-view v-if="isOverlay" />

  <div v-else class="app-shell">
    <a class="skip-link" href="#main-content">跳到内容</a>
    <aside class="sidebar" :class="{ open: sidebarOpen }" aria-label="主菜单">
      <div class="brand">
        <div class="brand-mark" aria-hidden="true"><span>S</span></div>
        <div>
          <div class="brand-name">sbox</div>
          <div class="brand-subtitle">实用工具箱</div>
        </div>
        <button class="sidebar-close" type="button" aria-label="关闭菜单" @click="closeSidebar">
          <X :size="18" />
        </button>
      </div>

      <nav class="sidebar-nav">
        <RouterLink class="nav-item overview" to="/">
          <Home :size="17" stroke-width="1.8" />
          <span>工具概览</span>
        </RouterLink>

        <section v-for="group in toolGroups" :key="group.key" class="nav-group">
          <h2>{{ group.label }}</h2>
          <RouterLink
            v-for="tool in group.tools"
            :key="tool.key"
            class="nav-item"
            :to="`/${tool.key}`"
            :title="tool.description"
          >
            <component :is="iconFor(tool.key)" :size="17" stroke-width="1.8" />
            <span>{{ tool.label }}</span>
          </RouterLink>
        </section>
      </nav>

      <div class="sidebar-footer">
        <RouterLink class="nav-item" to="/settings">
          <Settings :size="17" stroke-width="1.8" />
          <span>设置</span>
        </RouterLink>
      </div>
    </aside>

    <div v-if="sidebarOpen" class="sidebar-scrim" aria-hidden="true" @click="closeSidebar" />

    <section class="content-shell">
      <header class="mobile-topbar">
        <button class="menu-button" type="button" aria-label="打开菜单" @click="sidebarOpen = true">
          <Menu :size="20" />
        </button>
        <span>sbox</span>
      </header>
      <main id="main-content" class="app-main" tabindex="-1">
        <router-view />
      </main>
    </section>
    <UpdaterDialog />
  </div>
</template>

<style>
:root {
  --fg: #1a1a1a;
  --fg-muted: #666;
  --bg: #f8f8f8;
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

.app-shell { display: flex; height: 100%; min-width: 0; overflow: hidden; }
.sidebar {
  display: flex; flex: 0 0 252px; flex-direction: column; min-width: 0;
  border-right: 1px solid var(--border); background: var(--card);
}
.brand { display: flex; align-items: center; gap: 10px; min-height: 68px; padding: 14px 18px; border-bottom: 1px solid var(--border); }
.brand-mark { display: grid; width: 32px; height: 32px; place-items: center; border-radius: 8px; background: var(--primary); color: #fff; font-weight: 750; box-shadow: 0 2px 8px color-mix(in srgb, var(--primary) 25%, transparent); }
.brand-name { font-size: 16px; font-weight: 650; line-height: 1.2; }
.brand-subtitle { margin-top: 2px; color: var(--fg-muted); font-size: 11px; }
.sidebar-close { display: none; }
.sidebar-nav { flex: 1 1 auto; min-height: 0; overflow-y: auto; padding: 12px 10px; }
.nav-group { margin: 18px 0 0; }
.nav-group h2 { margin: 0 0 5px; padding: 0 9px; color: var(--fg-muted); font-size: 11px; font-weight: 650; letter-spacing: .04em; }
.nav-item {
  display: flex; align-items: center; gap: 10px; min-height: 38px; margin: 2px 0; padding: 7px 9px;
  border-radius: 6px; color: var(--fg); text-decoration: none; line-height: 1.3;
}
.nav-item svg { flex: 0 0 auto; color: var(--fg-muted); }
.nav-item:hover { background: color-mix(in srgb, var(--primary) 7%, transparent); color: var(--fg); }
.nav-item.router-link-exact-active { background: color-mix(in srgb, var(--primary) 13%, var(--card)); color: var(--primary); font-weight: 600; }
.nav-item.router-link-exact-active svg { color: var(--primary); }
.sidebar-footer { padding: 10px; border-top: 1px solid var(--border); }
.content-shell { display: flex; flex: 1 1 auto; min-width: 0; flex-direction: column; }
.app-main { flex: 1 1 auto; min-width: 0; overflow: auto; padding: 34px clamp(24px, 5vw, 72px); }
.app-main:focus { outline: 0; }
.mobile-topbar, .menu-button, .sidebar-scrim { display: none; }
.skip-link { position: fixed; z-index: 10; top: -48px; left: 16px; padding: 8px 12px; border-radius: 6px; background: var(--primary); color: #fff; text-decoration: none; }
.skip-link:focus { top: 12px; }

button.btn {
  background: var(--primary); color: #fff; border: none;
  padding: 8px 16px; border-radius: var(--radius); cursor: pointer;
  font-size: 14px; font-weight: 500;
}
button.btn:hover:not(:disabled) { background: var(--primary-hover); }
button.btn:disabled { opacity: 0.5; cursor: not-allowed; }
button.btn-outline { background: transparent; color: var(--fg); border: 1px solid var(--border); }
button.btn-outline:hover:not(:disabled) { border-color: var(--primary); color: var(--primary); }

@media (max-width: 760px) {
  .sidebar { position: fixed; z-index: 4; top: 0; bottom: 0; left: 0; width: min(300px, calc(100vw - 48px)); transform: translateX(-100%); transition: transform 180ms ease; box-shadow: none; }
  .sidebar.open { transform: translateX(0); box-shadow: 10px 0 28px rgb(0 0 0 / 16%); }
  .sidebar-close { display: grid; width: 32px; height: 32px; margin-left: auto; place-items: center; border: 0; border-radius: 6px; background: transparent; color: var(--fg-muted); }
  .sidebar-scrim { display: block; position: fixed; z-index: 3; inset: 0; background: rgb(0 0 0 / 32%); }
  .mobile-topbar { display: flex; align-items: center; gap: 10px; flex: 0 0 52px; padding: 0 14px; border-bottom: 1px solid var(--border); background: var(--card); font-size: 15px; font-weight: 650; }
  .menu-button { display: grid; width: 36px; height: 36px; place-items: center; border: 0; border-radius: 6px; background: transparent; color: var(--fg); }
  .menu-button:hover, .sidebar-close:hover { background: var(--bg); }
  .app-main { padding: 22px 16px; }
}
@media (prefers-reduced-motion: reduce) { .sidebar { transition: none; } }
</style>
