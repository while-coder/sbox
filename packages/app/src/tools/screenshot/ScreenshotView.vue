<script setup lang="ts">
import { ref } from 'vue'
import { startScreenshot } from './screenshot'
import { settings, saveSettings } from '../../settings'
import { screenshotKeyStatus, shortcutsRefreshing } from '../../system'

const error = ref('')
const recordingKey = ref(false)

const PRESETS = [
  'CommandOrControl+Shift+A',
  'CommandOrControl+Shift+S',
  'CommandOrControl+Alt+A',
  'PrintScreen',
]

function keyToken(e: KeyboardEvent): string {
  const code = e.code
  if (code.startsWith('Key')) return code.slice(3)
  if (code.startsWith('Digit')) return code.slice(5)
  if (/^F\d{1,2}$/.test(code)) return code
  const map: Record<string, string> = {
    Space: 'Space', Enter: 'Enter', Tab: 'Tab', Backquote: '`', Minus: '-', Equal: '=',
    ArrowUp: 'Up', ArrowDown: 'Down', ArrowLeft: 'Left', ArrowRight: 'Right',
  }
  return map[code] ?? ''
}

function onRecordKeydown(e: KeyboardEvent) {
  if (!recordingKey.value) return
  e.preventDefault()
  const token = keyToken(e)
  if (!token) return

  const modifiers: string[] = []
  if (e.ctrlKey || e.metaKey) modifiers.push('CommandOrControl')
  if (e.shiftKey) modifiers.push('Shift')
  if (e.altKey) modifiers.push('Alt')
  settings.screenshotKey = [...modifiers, token].join('+')
  recordingKey.value = false
  saveSettings()
}

function selectPreset(key: string) {
  settings.screenshotKey = key
  saveSettings()
}

async function shoot() {
  error.value = ''
  try {
    await startScreenshot()
  } catch (e: any) {
    error.value = `截图失败：${String(e?.message || e)}`
  }
}
</script>

<template>
  <div class="tool">
    <h2>截图</h2>
    <p class="lead">捕获主显示器后在全屏覆盖层上框选区域，可保存、复制到剪贴板或识别其中的二维码。</p>

    <section class="card">
      <button class="btn big" @click="shoot">开始截图</button>
      <p class="tip">点击后窗口会自动隐藏并冻结屏幕，拖拽鼠标框选区域；Esc 取消，Enter 保存。</p>
      <p class="tip" v-if="settings.screenshotEnabled && settings.screenshotKey">全局快捷键：<kbd>{{ settings.screenshotKey }}</kbd></p>
      <p class="tip muted">注：当前版本捕获主显示器；多显示器框选将在后续完善。</p>
    </section>

    <section class="settings-card" aria-labelledby="screenshot-shortcut-heading">
      <h3 id="screenshot-shortcut-heading">快捷键与行为</h3>
      <div class="row">
        <div class="row-text">
          <div class="row-label">启用截图快捷键</div>
          <div class="row-desc">全局快捷键，一键发起全屏框选截图。</div>
        </div>
        <label class="switch">
          <input v-model="settings.screenshotEnabled" type="checkbox" :disabled="shortcutsRefreshing" @change="saveSettings" />
          <span class="slider"></span>
        </label>
      </div>

      <div class="row" :class="{ disabled: !settings.screenshotEnabled }">
        <div class="row-text">
          <div class="row-label">快捷键</div>
          <div class="row-desc">点击右侧按钮后按下组合键录制。</div>
        </div>
        <button
          type="button"
          class="recorder"
          :class="{ recording: recordingKey }"
          :disabled="!settings.screenshotEnabled || shortcutsRefreshing"
          @click="recordingKey = true"
          @blur="recordingKey = false"
          @keydown="onRecordKeydown"
        >{{ recordingKey ? '按下组合键…' : settings.screenshotKey }}</button>
      </div>

      <div class="presets" :class="{ disabled: !settings.screenshotEnabled }">
        <span class="presets-label">预设：</span>
        <button
          v-for="preset in PRESETS"
          :key="preset"
          type="button"
          class="preset-chip"
          :class="{ active: settings.screenshotKey === preset }"
          :disabled="!settings.screenshotEnabled || shortcutsRefreshing"
          @click="selectPreset(preset)"
        >{{ preset }}</button>
      </div>

      <div v-if="screenshotKeyStatus.message" class="status" :class="screenshotKeyStatus.state" aria-live="polite">
        <span class="status-dot"></span>{{ screenshotKeyStatus.message }}
      </div>

      <div class="row">
        <div class="row-text">
          <div class="row-label">截图时隐藏 sbox 窗口</div>
          <div class="row-desc">避免把当前窗口截进去；关闭后会连同 sbox 一起截取。</div>
        </div>
        <label class="switch">
          <input v-model="settings.screenshotHideSelf" type="checkbox" @change="saveSettings" />
          <span class="slider"></span>
        </label>
      </div>
    </section>

    <p v-if="error" class="error">{{ error }}</p>
  </div>
</template>

<style scoped>
.tool { max-width: 720px; margin: 0 auto; }
.lead { color: var(--fg-muted); margin-bottom: 16px; }
.card {
  background: var(--card); border: 1px solid var(--border);
  border-radius: var(--radius); padding: 24px; text-align: center;
}
.settings-card {
  margin-top: 16px; padding: 4px 20px;
  border: 1px solid var(--border); border-radius: var(--radius); background: var(--card);
}
.settings-card h3 { margin: 16px 0 4px; font-size: 15px; }
.btn.big {
  background: var(--primary); color: #fff; border: none;
  padding: 12px 32px; border-radius: var(--radius); cursor: pointer;
  font-size: 15px; font-weight: 600;
}
.btn.big:hover { background: var(--primary-hover); }
.tip { color: var(--fg-muted); font-size: 13px; margin: 12px 0 0; }
.tip.muted { opacity: 0.75; font-size: 12px; }
.row { display: flex; align-items: center; justify-content: space-between; gap: 16px; padding: 16px 0; }
.row + .row { border-top: 1px solid var(--border); }
.row.disabled, .presets.disabled { opacity: .45; pointer-events: none; }
.row-text { flex: 1 1 auto; min-width: 0; }
.row-label { font-size: 14px; font-weight: 500; }
.row-desc { margin-top: 4px; color: var(--fg-muted); font-size: 12px; }
.switch { position: relative; display: inline-block; flex: 0 0 auto; width: 42px; height: 24px; }
.switch input { width: 0; height: 0; opacity: 0; }
.slider { position: absolute; inset: 0; border-radius: 24px; background: var(--border); cursor: pointer; transition: .2s; }
.slider::before { position: absolute; bottom: 3px; left: 3px; width: 18px; height: 18px; border-radius: 50%; background: #fff; content: ''; transition: .2s; }
.switch input:checked + .slider { background: var(--primary); }
.switch input:checked + .slider::before { transform: translateX(18px); }
.recorder { flex: 0 0 auto; min-width: 200px; border: 1px solid var(--border); border-radius: var(--radius); background: var(--bg); color: var(--fg); padding: 8px 14px; font: 13px ui-monospace, monospace; cursor: pointer; }
.recorder:hover:not(:disabled), .recorder.recording { border-color: var(--primary); color: var(--primary); }
.recorder:disabled { cursor: not-allowed; }
.presets { display: flex; flex-wrap: wrap; align-items: center; gap: 8px; padding: 0 0 16px; }
.presets-label { color: var(--fg-muted); font-size: 12px; }
.preset-chip { border: 1px solid var(--border); border-radius: 6px; background: var(--bg); color: var(--fg-muted); padding: 3px 8px; font: 12px ui-monospace, monospace; cursor: pointer; }
.preset-chip:hover:not(:disabled), .preset-chip.active { border-color: var(--primary); color: var(--primary); background: var(--card); }
.status { display: flex; align-items: center; gap: 6px; padding: 0 0 16px; font-size: 12px; }
.status-dot { width: 7px; height: 7px; flex: 0 0 auto; border-radius: 50%; }
.status.ok { color: var(--success); }.status.ok .status-dot { background: var(--success); }
.status.error { color: var(--danger); }.status.error .status-dot { background: var(--danger); }
.status.idle { color: var(--fg-muted); }.status.idle .status-dot { background: var(--fg-muted); }
kbd {
  background: var(--bg); border: 1px solid var(--border); border-radius: 4px;
  padding: 1px 6px; font: 12px ui-monospace, Consolas, monospace;
}
.error { color: var(--danger); margin: 12px 0 0; font-size: 13px; }
@media (max-width: 560px) {
  .settings-card { padding: 4px 16px; }
  .row { align-items: flex-start; }
  .recorder { min-width: 144px; }
}
</style>
