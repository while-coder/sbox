<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Box, Github, RefreshCw } from 'lucide-vue-next'
import { settings, saveSettings } from '../settings'
import { autostartStatus, bossKeyStatus, setAutostart } from '../system'
import { useUpdater } from '../useUpdater'

const SETTING_TABS = [
  { key: 'general', label: '通用' },
  { key: 'shortcuts', label: '快捷键' },
  { key: 'about', label: '关于' },
] as const
type SettingsTab = (typeof SETTING_TABS)[number]['key']

const activeTab = ref<SettingsTab>('general')
const {
  appVersion,
  updateStatus,
  updateStatusText,
  checkForUpdate,
  initUpdaterVersion,
} = useUpdater()

type KeyTarget = 'bossKey'
const recordingTarget = ref<KeyTarget | null>(null)
const aboutError = ref('')

/** 几个常用预设组合键。 */
const PRESETS = [
  'CommandOrControl+Shift+H',
  'CommandOrControl+Shift+X',
  'CommandOrControl+Alt+B',
  'Alt+Q',
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

async function onCheckUpdate() {
  await checkForUpdate({ silent: false })
}

async function openGitHub() {
  aboutError.value = ''
  try {
    await invoke('open_external_url', { url: 'https://github.com/while-coder/sbox' })
  } catch (e: any) {
    aboutError.value = `无法打开 GitHub：${String(e?.message || e)}`
  }
}

onMounted(() => {
  void initUpdaterVersion()
})
</script>

<template>
  <div class="settings">
    <h2 class="page-title">设置</h2>

    <div class="settings-tabs" role="tablist" aria-label="设置分类">
      <button
        v-for="tab in SETTING_TABS"
        :key="tab.key"
        type="button"
        role="tab"
        :class="{ active: activeTab === tab.key }"
        :aria-selected="activeTab === tab.key"
        @click="activeTab = tab.key"
      >
        {{ tab.label }}
      </button>
    </div>

    <section v-show="activeTab === 'general'" class="card">
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

    <section v-show="activeTab === 'shortcuts'" class="card">
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

    <p v-show="activeTab === 'shortcuts'" class="hint">CommandOrControl 在 Windows/Linux 上为 Ctrl，在 macOS 上为 ⌘。</p>

    <section v-show="activeTab === 'about'" class="about">
      <header class="about-hero">
        <div class="about-mark" aria-hidden="true"><Box :size="30" stroke-width="1.8" /></div>
        <div class="about-copy">
          <h2>sbox</h2>
          <p>本地桌面工具箱</p>
        </div>
        <span class="version-badge">v{{ appVersion || '…' }}</span>
        <p class="about-description">集中提供编码、校验、图像处理、截图和系统辅助工具，让日常操作少一些重复步骤。</p>
      </header>

      <div class="about-grid">
        <section class="about-panel" aria-labelledby="update-heading">
          <div class="about-panel-icon update-icon" aria-hidden="true"><RefreshCw :size="19" stroke-width="1.8" /></div>
          <div class="about-panel-copy">
            <h3 id="update-heading">应用更新</h3>
            <p :class="{ 'update-error': updateStatus === 'error' }">{{ updateStatusText }}</p>
          </div>
          <button
            type="button"
            class="btn btn-outline about-action"
            :disabled="updateStatus === 'checking' || updateStatus === 'downloading'"
            @click="onCheckUpdate"
          >{{ updateStatus === 'checking' ? '检查中…' : '检查更新' }}</button>
        </section>

        <section class="about-panel" aria-labelledby="github-heading">
          <div class="about-panel-icon github-icon" aria-hidden="true"><Github :size="19" stroke-width="1.8" /></div>
          <div class="about-panel-copy">
            <h3 id="github-heading">GitHub 与使用说明</h3>
            <p>查看源码、使用说明、更新记录或提交问题反馈。</p>
          </div>
          <button type="button" class="btn btn-outline about-action" @click="openGitHub">查看详情</button>
        </section>
      </div>
      <p v-if="aboutError" class="about-error" role="alert">{{ aboutError }}</p>
    </section>
  </div>
</template>

<style scoped>
.settings { max-width: 720px; margin: 0 auto; }
.page-title { font-size: 18px; font-weight: 600; margin: 0 0 20px; }

.settings-tabs {
  display: flex; gap: 4px;
  padding: 4px; margin-bottom: 16px;
  border: 1px solid var(--border); border-radius: var(--radius);
  background: var(--card);
}
.settings-tabs button {
  flex: 1 1 0; min-height: 36px;
  border: 0; border-radius: 6px;
  background: transparent; color: var(--fg-muted);
  font-size: 13px; font-weight: 600; cursor: pointer;
}
.settings-tabs button:hover { background: var(--bg); color: var(--fg); }
.settings-tabs button.active { background: var(--primary); color: #fff; }

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

.about { margin-top: 4px; }
.about-hero {
  display: grid; grid-template-columns: auto minmax(0, 1fr) auto; gap: 12px; align-items: center;
  padding: 24px; border: 1px solid var(--border); border-radius: calc(var(--radius) + 2px);
  background: linear-gradient(135deg, color-mix(in srgb, var(--primary) 12%, var(--card)), var(--card) 56%);
}
.about-mark {
  display: grid; place-items: center;
  width: 54px; height: 54px; border-radius: 14px; background: var(--primary); color: #fff;
  box-shadow: 0 8px 20px color-mix(in srgb, var(--primary) 28%, transparent);
}
.about-copy { min-width: 0; }
.about-copy h2 { margin: 0; font-size: 22px; line-height: 1.2; }
.about-copy p { margin: 4px 0 0; color: var(--fg-muted); font-size: 13px; }
.version-badge { padding: 4px 8px; border: 1px solid var(--border); border-radius: 99px; background: var(--card); color: var(--fg-muted); font: 12px ui-monospace, monospace; }
.about-description { grid-column: 1 / -1; margin: 5px 0 0; color: var(--fg-muted); font-size: 13px; }
.about-grid { display: grid; gap: 12px; margin-top: 14px; }
.about-panel { display: grid; grid-template-columns: auto minmax(0, 1fr) auto; gap: 12px; align-items: center; padding: 16px; border: 1px solid var(--border); border-radius: var(--radius); background: var(--card); }
.about-panel-icon { display: grid; width: 38px; height: 38px; place-items: center; border-radius: 9px; color: var(--primary); background: color-mix(in srgb, var(--primary) 12%, var(--card)); }
.github-icon { color: var(--fg); background: var(--bg); }
.about-panel-copy { min-width: 0; }
.about-panel-copy h3 { margin: 0; font-size: 14px; font-weight: 600; }
.about-panel-copy p { margin: 4px 0 0; color: var(--fg-muted); font-size: 12px; }
.about-action { flex: 0 0 auto; min-width: 96px; }
.row-desc.update-error { color: var(--danger); }
.about-panel-copy .update-error { color: var(--danger); }
.about-error { margin: 0 0 16px; color: var(--danger); font-size: 12px; }

@media (max-width: 560px) {
  .row { align-items: flex-start; }
  .update-row { flex-direction: column; }
  .update-button { width: 100%; }
  .about-hero { grid-template-columns: auto minmax(0, 1fr); padding: 20px; }
  .version-badge { justify-self: start; }
  .about-panel { grid-template-columns: auto minmax(0, 1fr); }
  .about-action { grid-column: 1 / -1; width: 100%; }
}
</style>
