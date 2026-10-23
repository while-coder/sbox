<script setup lang="ts">
import { onMounted, onUnmounted, type WatchStopHandle } from 'vue'
import { useRouter } from 'vue-router'
import SboxApp from '@sbox/ui/app'
import { UpdaterDialog } from '@while-coder/tauri-updater-vue'
import { APP_TOOLS } from './shell/registry'
import { native } from './platform/native'
import { loadSettings } from './shell/settings'
import { setupCloseToTray, applyBossKey, watchBossKey, refreshAutostart } from './shell/system'
import { ensureOverlay } from './tools/media/screenshot/screenshot'

const router = useRouter()
let stopShortcutWatch: WatchStopHandle | undefined
let unlistenClose: (() => void) | undefined
let unlistenOpenSettings: (() => void) | undefined
onMounted(async () => {
  loadSettings()
  if (native.windowLabel !== 'main') return
  void ensureOverlay().catch(error => console.error('初始化截图覆盖层失败：', error))
  void refreshAutostart()
  unlistenClose = await setupCloseToTray()
  await applyBossKey()
  stopShortcutWatch = watchBossKey()
  unlistenOpenSettings = await native.listen('open-settings', () => router.push('/settings'))
})
onUnmounted(() => {
  stopShortcutWatch?.()
  unlistenOpenSettings?.()
  unlistenClose?.()
})
</script>

<template>
  <SboxApp :tools="APP_TOOLS"><template #updater><UpdaterDialog /></template></SboxApp>
</template>
