<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import {
  closeTranslator,
  navigateTranslator,
  openTranslator,
  reloadTranslator,
  setTranslatorBounds,
  type TranslationProvider,
  type TranslationWebviewBounds,
} from './tauri'

const provider = ref<TranslationProvider>('google')
const viewport = ref<HTMLElement | null>(null)
const loading = ref(true)
const error = ref('')
const providers: { value: TranslationProvider; label: string }[] = [
  { value: 'google', label: 'Google 翻译' },
  { value: 'baidu', label: '百度翻译' },
  { value: 'bing', label: 'Bing 翻译' },
  { value: 'youdao', label: '有道翻译' },
  { value: 'deepl', label: 'DeepL' },
  { value: 'tencent', label: '混元翻译' },
]

let resizeObserver: ResizeObserver | null = null
let frameId = 0
let opened = false

function currentBounds(): TranslationWebviewBounds | null {
  const element = viewport.value
  if (!element) return null
  const rect = element.getBoundingClientRect()
  if (rect.width < 1 || rect.height < 1) return null
  return {
    x: Math.round(rect.left),
    y: Math.round(rect.top),
    width: Math.round(rect.width),
    height: Math.round(rect.height),
  }
}

function scheduleBoundsSync() {
  window.cancelAnimationFrame(frameId)
  frameId = window.requestAnimationFrame(() => {
    const bounds = currentBounds()
    if (opened && bounds) void setTranslatorBounds(bounds)
  })
}

async function mountWebview() {
  const bounds = currentBounds()
  if (!bounds) return
  loading.value = true
  error.value = ''
  try {
    await openTranslator(provider.value, bounds)
    opened = true
  } catch (e: any) {
    error.value = String(e?.message || e)
  } finally {
    loading.value = false
  }
}

async function changeProvider() {
  loading.value = true
  error.value = ''
  try {
    if (opened) await navigateTranslator(provider.value)
    else await mountWebview()
  } catch (e: any) {
    error.value = String(e?.message || e)
  } finally {
    loading.value = false
  }
}

async function selectProvider(nextProvider: TranslationProvider) {
  if (loading.value || provider.value === nextProvider) return
  provider.value = nextProvider
  await changeProvider()
}

function focusProvider(index: number) {
  const item = providers[index]
  if (!item) return
  void selectProvider(item.value)
  void nextTick(() => document.getElementById(`translation-provider-${item.value}`)?.focus())
}

function moveProvider(event: KeyboardEvent, index: number, offset: number) {
  event.preventDefault()
  focusProvider((index + offset + providers.length) % providers.length)
}

async function reload() {
  error.value = ''
  try {
    await reloadTranslator()
  } catch (e: any) {
    error.value = String(e?.message || e)
  }
}

onMounted(async () => {
  await nextTick()
  resizeObserver = new ResizeObserver(scheduleBoundsSync)
  if (viewport.value) resizeObserver.observe(viewport.value)
  window.addEventListener('resize', scheduleBoundsSync)
  document.addEventListener('scroll', scheduleBoundsSync, true)
  await mountWebview()
})

onBeforeUnmount(() => {
  window.cancelAnimationFrame(frameId)
  resizeObserver?.disconnect()
  window.removeEventListener('resize', scheduleBoundsSync)
  document.removeEventListener('scroll', scheduleBoundsSync, true)
  opened = false
  void closeTranslator()
})
</script>

<template>
  <div class="translator">
    <div class="toolbar">
      <span class="toolbar-label">翻译服务</span>
      <div class="provider-tabs" role="tablist" aria-label="翻译服务">
        <button
          v-for="(item, index) in providers"
          :id="`translation-provider-${item.value}`"
          :key="item.value"
          type="button"
          role="tab"
          class="provider-tab"
          :class="{ active: provider === item.value }"
          :aria-selected="provider === item.value"
          aria-controls="translation-webview"
          :tabindex="provider === item.value ? 0 : -1"
          :disabled="loading"
          @click="selectProvider(item.value)"
          @keydown.left="moveProvider($event, index, -1)"
          @keydown.right="moveProvider($event, index, 1)"
          @keydown.home.prevent="focusProvider(0)"
          @keydown.end.prevent="focusProvider(providers.length - 1)"
        >
          {{ item.label }}
        </button>
      </div>
      <button type="button" class="toolbar-btn" :disabled="loading || !opened" @click="reload">
        刷新网页
      </button>
      <span v-if="loading" class="state" aria-live="polite">正在打开…</span>
      <span v-else-if="error" class="state error" role="alert">{{ error }}</span>
    </div>

    <div
      id="translation-webview"
      ref="viewport"
      class="webview-viewport"
      role="tabpanel"
      :aria-labelledby="`translation-provider-${provider}`"
    >
      <p v-if="loading" class="placeholder">正在加载翻译网页…</p>
      <div v-else-if="error" class="placeholder error-panel">
        <p>翻译网页打开失败：{{ error }}</p>
        <button type="button" class="btn btn-outline" @click="mountWebview">重试</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.translator {
  height: 100%; min-height: 520px;
  display: flex; flex-direction: column; gap: 12px;
}
.toolbar {
  flex: 0 0 auto; min-height: 42px;
  display: flex; align-items: center; gap: 10px;
  border-bottom: 1px solid var(--border);
}
.toolbar-label { flex: 0 0 auto; color: var(--fg-muted); font-size: 13px; }
.provider-tabs {
  align-self: stretch; flex: 1 1 auto; min-width: 0;
  display: flex; align-items: stretch;
  overflow-x: auto; scrollbar-width: thin;
}
.provider-tab {
  position: relative; min-width: 108px; padding: 8px 14px;
  border: none; border-bottom: 2px solid transparent;
  background: transparent; color: var(--fg-muted); font-size: 13px;
  white-space: nowrap; cursor: pointer;
  transition: color 0.15s, border-color 0.15s, background 0.15s;
}
.provider-tab:hover:not(:disabled) { color: var(--fg); background: var(--bg); }
.provider-tab.active { color: var(--primary); border-bottom-color: var(--primary); font-weight: 600; }
.toolbar-btn {
  min-height: 36px; border: 1px solid var(--border); border-radius: 5px;
  background: var(--card); color: var(--fg); font-size: 13px;
}
.toolbar-btn { padding: 6px 12px; cursor: pointer; }
.toolbar-btn:hover:not(:disabled) { border-color: var(--primary); color: var(--primary); }
.provider-tab:focus-visible, .toolbar-btn:focus-visible {
  outline: 2px solid var(--primary); outline-offset: 2px;
}
.provider-tab:disabled, .toolbar-btn:disabled { opacity: 0.5; cursor: not-allowed; }
.state { margin-left: auto; color: var(--fg-muted); font-size: 12px; }
.state.error { color: var(--danger); }
.webview-viewport {
  position: relative; flex: 1 1 auto; min-height: 460px;
  overflow: hidden; border: 1px solid var(--border); border-radius: var(--radius);
  background: var(--card);
}
.placeholder {
  position: absolute; inset: 0; margin: 0;
  display: flex; align-items: center; justify-content: center;
  color: var(--fg-muted); font-size: 13px;
}
.error-panel { flex-direction: column; gap: 12px; padding: 20px; text-align: center; color: var(--danger); }
.error-panel p { margin: 0; }
@media (max-width: 600px) {
  .translator { min-height: 480px; }
  .toolbar { flex-wrap: wrap; }
  .provider-tabs { order: 3; flex: 1 1 100%; }
  .provider-tab { flex: 1 0 auto; }
  .state { order: 4; width: 100%; margin-left: 0; }
  .webview-viewport { min-height: 400px; }
}
</style>
