<script setup lang="ts">
import { ref } from 'vue'
import { settings, saveSettings } from '../settings'
import { autostartStatus, bossKeyStatus, screenshotKeyStatus, setAutostart } from '../system'

type KeyTarget = 'bossKey' | 'screenshotKey'
const recordingTarget = ref<KeyTarget | null>(null)

/** 几个常用预设组合键。 */
const PRESETS = [
  'CommandOrControl+Shift+H',
  'CommandOrControl+Shift+X',
  'CommandOrControl+Alt+B',
  'Alt+Q',
]
const SHOT_PRESETS = [
  'CommandOrControl+Shift+A',
  'CommandOrControl+Shift+S',
  'CommandOrControl+Alt+A',
  'PrintScreen',
]

/** 把键名转为 Tauri 加速键 token；非法/纯修饰键返回空串。 */
function keyToken(e: KeyboardEvent): string {
  const code = e.code
  if (code.startsWith('Key')) return code.slice(3) // KeyH -> H
  if (code.startsWith('Digit')) return code.slice(5) // Digit1 -> 1
  if (/^F\d{1,2}$/.test(code)) return code // F1..F12
  const map: Record<string, string> = {
    Space: 'Space',
    Enter: 'Enter',
    Tab: 'Tab',
    Backquote: '`',
    Minus: '-',
    Equal: '=',
    ArrowUp: 'Up',
    ArrowDown: 'Down',
    ArrowLeft: 'Left',
    ArrowRight: 'Right',
  }
  return map[code] ?? ''
}

function onRecordKeydown(e: KeyboardEvent) {
  const target = recordingTarget.value
  if (!target) return
  e.preventDefault()
  const token = keyToken(e)
  if (!token) return // 还在按修饰键，等主键

  const mods: string[] = []
  if (e.ctrlKey || e.metaKey) mods.push('CommandOrControl')
  if (e.shiftKey) mods.push('Shift')
  if (e.altKey) mods.push('Alt')

  settings[target] = [...mods, token].join('+')
  recordingTarget.value = null
  saveSettings()
}

function applyPreset(target: KeyTarget, combo: string) {
  settings[target] = combo
  saveSettings()
}

function onToggle() {
  saveSettings()
}

async function onAutostartToggle(event: Event) {
  const enabled = (event.target as HTMLInputElement).checked
  await setAutostart(enabled, !enabled)
  saveSettings()
}
</script>

<template>
  <div class="settings">
    <h2 class="page-title">设置</h2>

    <section class="card">
      <div class="row">
        <div class="row-text">
          <div class="row-label">开机启动</div>
          <div class="row-desc">登录系统后自动启动 sbox。</div>
        </div>
        <label class="switch">
          <input type="checkbox" v-model="settings.autostart" @change="onAutostartToggle" />
          <span class="slider"></span>
        </label>
      </div>

      <div
        v-if="autostartStatus.message"
        class="status"
        :class="autostartStatus.state"
      >
        <span class="status-dot"></span>{{ autostartStatus.message }}
      </div>

      <div class="row">
        <div class="row-text">
          <div class="row-label">关闭按钮最小化到托盘</div>
          <div class="row-desc">点击窗口关闭按钮（×）时隐藏到托盘，程序后台常驻；通过托盘菜单退出。</div>
        </div>
        <label class="switch">
          <input type="checkbox" v-model="settings.closeToTray" @change="onToggle" />
          <span class="slider"></span>
        </label>
      </div>
    </section>

    <section class="card">
      <div class="row">
        <div class="row-text">
          <div class="row-label">启用老板键</div>
          <div class="row-desc">全局快捷键，一键把窗口隐藏到托盘，再按一次恢复。</div>
        </div>
        <label class="switch">
          <input type="checkbox" v-model="settings.bossKeyEnabled" @change="onToggle" />
          <span class="slider"></span>
        </label>
      </div>

      <div class="row" :class="{ disabled: !settings.bossKeyEnabled }">
        <div class="row-text">
          <div class="row-label">快捷键</div>
          <div class="row-desc">点击下方按钮后按下组合键录制。</div>
        </div>
        <button
          class="recorder"
          :class="{ recording: recordingTarget === 'bossKey' }"
          :disabled="!settings.bossKeyEnabled"
          @click="recordingTarget = 'bossKey'"
          @blur="recordingTarget = null"
          @keydown="onRecordKeydown"
        >
          {{ recordingTarget === 'bossKey' ? '按下组合键…' : settings.bossKey }}
        </button>
      </div>

      <div class="presets" :class="{ disabled: !settings.bossKeyEnabled }">
        <span class="presets-label">预设：</span>
        <button
          v-for="p in PRESETS"
          :key="p"
          class="preset-chip"
          :class="{ active: settings.bossKey === p }"
          :disabled="!settings.bossKeyEnabled"
          @click="applyPreset('bossKey', p)"
        >{{ p }}</button>
      </div>

      <div
        v-if="settings.bossKeyEnabled && bossKeyStatus.message"
        class="status"
        :class="bossKeyStatus.state"
      >
        <span class="status-dot"></span>{{ bossKeyStatus.message }}
      </div>
    </section>

    <section class="card">
      <div class="row">
        <div class="row-text">
          <div class="row-label">启用截图快捷键</div>
          <div class="row-desc">全局快捷键，一键发起全屏框选截图。</div>
        </div>
        <label class="switch">
          <input type="checkbox" v-model="settings.screenshotEnabled" @change="onToggle" />
          <span class="slider"></span>
        </label>
      </div>

      <div class="row">
        <div class="row-text">
          <div class="row-label">截图时隐藏 sbox 窗口</div>
          <div class="row-desc">发起截图前先隐藏主窗口，避免把 sbox 自己截进去；关闭后将连同窗口一起截取。</div>
        </div>
        <label class="switch">
          <input type="checkbox" v-model="settings.screenshotHideSelf" @change="onToggle" />
          <span class="slider"></span>
        </label>
      </div>

      <div class="row" :class="{ disabled: !settings.screenshotEnabled }">
        <div class="row-text">
          <div class="row-label">快捷键</div>
          <div class="row-desc">点击下方按钮后按下组合键录制。</div>
        </div>
        <button
          class="recorder"
          :class="{ recording: recordingTarget === 'screenshotKey' }"
          :disabled="!settings.screenshotEnabled"
          @click="recordingTarget = 'screenshotKey'"
          @blur="recordingTarget = null"
          @keydown="onRecordKeydown"
        >
          {{ recordingTarget === 'screenshotKey' ? '按下组合键…' : settings.screenshotKey }}
        </button>
      </div>

      <div class="presets" :class="{ disabled: !settings.screenshotEnabled }">
        <span class="presets-label">预设：</span>
        <button
          v-for="p in SHOT_PRESETS"
          :key="p"
          class="preset-chip"
          :class="{ active: settings.screenshotKey === p }"
          :disabled="!settings.screenshotEnabled"
          @click="applyPreset('screenshotKey', p)"
        >{{ p }}</button>
      </div>

      <div
        v-if="settings.screenshotEnabled && screenshotKeyStatus.message"
        class="status"
        :class="screenshotKeyStatus.state"
      >
        <span class="status-dot"></span>{{ screenshotKeyStatus.message }}
      </div>
    </section>

    <p class="hint">CommandOrControl 在 Windows/Linux 上为 Ctrl，在 macOS 上为 ⌘。</p>
  </div>
</template>

<style scoped>
.settings { max-width: 720px; margin: 0 auto; }
.page-title { font-size: 18px; font-weight: 600; margin: 0 0 20px; }

.card {
  background: var(--card); border: 1px solid var(--border); border-radius: var(--radius);
  padding: 4px 16px; margin-bottom: 16px;
}
.row {
  display: flex; align-items: center; justify-content: space-between; gap: 16px;
  padding: 16px 0;
}
.row + .row { border-top: 1px solid var(--border); }
.row.disabled { opacity: 0.45; pointer-events: none; }
.row-text { flex: 1 1 auto; }
.row-label { font-size: 14px; font-weight: 500; }
.row-desc { font-size: 12px; color: var(--fg-muted); margin-top: 4px; }

/* 开关 */
.switch { position: relative; display: inline-block; width: 42px; height: 24px; flex: 0 0 auto; }
.switch input { opacity: 0; width: 0; height: 0; }
.slider {
  position: absolute; inset: 0; cursor: pointer;
  background: var(--border); border-radius: 24px; transition: 0.2s;
}
.slider::before {
  content: ''; position: absolute; height: 18px; width: 18px; left: 3px; bottom: 3px;
  background: #fff; border-radius: 50%; transition: 0.2s;
}
.switch input:checked + .slider { background: var(--primary); }
.switch input:checked + .slider::before { transform: translateX(18px); }

/* 录制按钮 */
.recorder {
  flex: 0 0 auto; min-width: 200px; text-align: center;
  background: var(--bg); color: var(--fg);
  border: 1px solid var(--border); border-radius: var(--radius);
  padding: 8px 14px; font-size: 13px; font-family: ui-monospace, monospace; cursor: pointer;
}
.recorder:hover:not(:disabled) { border-color: var(--primary); }
.recorder.recording { border-color: var(--primary); color: var(--primary); }
.recorder:disabled { cursor: not-allowed; }

/* 预设 */
.presets { display: flex; align-items: center; flex-wrap: wrap; gap: 8px; padding: 0 0 16px; }
.presets.disabled { opacity: 0.45; pointer-events: none; }
.presets-label { font-size: 12px; color: var(--fg-muted); }
.preset-chip {
  background: var(--bg); color: var(--fg-muted);
  border: 1px solid var(--border); border-radius: 6px;
  padding: 3px 8px; font-size: 12px; font-family: ui-monospace, monospace; cursor: pointer;
}
.preset-chip:hover:not(:disabled) { border-color: var(--primary); color: var(--primary); }
.preset-chip.active { border-color: var(--primary); color: var(--primary); background: var(--card); }

/* 注册状态反馈 */
.status {
  display: flex; align-items: center; gap: 6px;
  font-size: 12px; padding: 0 0 16px;
}
.status-dot { width: 7px; height: 7px; border-radius: 50%; flex: 0 0 auto; }
.status.ok { color: var(--success); }
.status.ok .status-dot { background: var(--success); }
.status.error { color: var(--danger); }
.status.error .status-dot { background: var(--danger); }
.status.idle { color: var(--fg-muted); }
.status.idle .status-dot { background: var(--fg-muted); }

.hint { font-size: 12px; color: var(--fg-muted); margin: 0; }
</style>
