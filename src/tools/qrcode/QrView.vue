<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue'
import {
  generateQrDataUrl, decodeQrFromBlob, readClipboardImage,
  type ErrorLevel,
} from './qr'
import { saveBase64File } from '../../save'

type Tab = 'gen' | 'scan'
const activeTab = ref<Tab>('gen')
const error = ref('')
const copied = ref(false)

// ── 生成 ──────────────────────────────────────────────────
const genText = ref('')
const level = ref<ErrorLevel>('M')
const size = ref(320)
const qrDataUrl = ref('')

async function regen() {
  error.value = ''
  if (!genText.value) { qrDataUrl.value = ''; return }
  try {
    qrDataUrl.value = await generateQrDataUrl(genText.value, {
      errorCorrectionLevel: level.value,
      width: size.value,
    })
  } catch (e: any) {
    qrDataUrl.value = ''
    error.value = String(e?.message || e)
  }
}
watch([genText, level, size], regen)

async function savePng() {
  if (!qrDataUrl.value) return
  try {
    const b64 = qrDataUrl.value.split(',', 2)[1]
    await saveBase64File(b64, 'qrcode.png')
  } catch (e: any) { error.value = String(e?.message || e) }
}

// ── 识别 ──────────────────────────────────────────────────
const scanPreview = ref('')
const scanResult = ref('')
const scanned = ref(false)   // 是否已尝试过识别
const scanBusy = ref(false)

async function handleBlob(blob: Blob) {
  error.value = ''
  scanBusy.value = true
  try {
    const { text, dataUrl } = await decodeQrFromBlob(blob)
    scanPreview.value = dataUrl
    scanResult.value = text ?? ''
    scanned.value = true
  } catch (e: any) {
    error.value = String(e?.message || e)
  } finally {
    scanBusy.value = false
  }
}

function onFilePicked(e: Event) {
  const f = (e.target as HTMLInputElement).files?.[0]
  if (f) void handleBlob(f)
}

function onDrop(e: DragEvent) {
  const f = e.dataTransfer?.files?.[0]
  if (f && f.type.startsWith('image/')) void handleBlob(f)
}

async function fromClipboardButton() {
  error.value = ''
  try {
    const blob = await readClipboardImage()
    if (!blob) { error.value = '剪贴板里没有图片（或浏览器不支持读取），可改用 Ctrl+V 粘贴'; return }
    await handleBlob(blob)
  } catch (e: any) {
    error.value = `读取剪贴板失败：${String(e?.message || e)}，可改用 Ctrl+V 粘贴`
  }
}

// 全局粘贴：识别 tab 激活时，Ctrl+V 直接识别剪贴板图片
function onPaste(e: ClipboardEvent) {
  if (activeTab.value !== 'scan') return
  const item = [...(e.clipboardData?.items ?? [])].find(i => i.type.startsWith('image/'))
  if (item) {
    const f = item.getAsFile()
    if (f) { e.preventDefault(); void handleBlob(f) }
  }
}
onMounted(() => window.addEventListener('paste', onPaste))
onUnmounted(() => window.removeEventListener('paste', onPaste))

// ── 复制 ──────────────────────────────────────────────────
async function copyResult() {
  if (!scanResult.value) return
  try {
    await navigator.clipboard.writeText(scanResult.value)
    copied.value = true
    setTimeout(() => (copied.value = false), 2000)
  } catch (e: any) { error.value = `复制失败: ${String(e?.message || e)}` }
}

const tabs: { key: Tab; label: string }[] = [
  { key: 'gen', label: '生成' },
  { key: 'scan', label: '识别' },
]
</script>

<template>
  <div class="tool">
    <h2>二维码生成 / 识别</h2>
    <p class="lead">本机生成二维码，或识别图片中的二维码（支持 Ctrl+V 粘贴、拖拽、选择文件、读取剪贴板）。</p>

    <div class="tab-bar">
      <button v-for="t in tabs" :key="t.key" class="tab"
        :class="{ active: activeTab === t.key }" @click="activeTab = t.key">
        {{ t.label }}
      </button>
    </div>

    <!-- 生成 -->
    <div v-if="activeTab === 'gen'" class="panel gen-grid">
      <section class="card">
        <div class="label-row"><span class="label">内容</span><button class="link-btn" @click="genText = ''">清空</button></div>
        <textarea v-model="genText" class="textarea" rows="6" placeholder="输入文本 / 网址 / 任意字符串…" />
        <div class="row opts">
          <label class="field">
            <span class="label">纠错等级</span>
            <select v-model="level" class="input">
              <option value="L">L（7%）</option>
              <option value="M">M（15%）</option>
              <option value="Q">Q（25%）</option>
              <option value="H">H（30%）</option>
            </select>
          </label>
          <label class="field">
            <span class="label">尺寸 {{ size }}px</span>
            <input v-model.number="size" type="range" min="128" max="640" step="32" />
          </label>
        </div>
      </section>

      <section class="card preview-card">
        <div v-if="qrDataUrl" class="qr-wrap">
          <img :src="qrDataUrl" class="qr-img" alt="二维码" />
          <button class="btn" @click="savePng">保存为 PNG…</button>
        </div>
        <p v-else class="placeholder">输入内容后这里会显示二维码</p>
      </section>
    </div>

    <!-- 识别 -->
    <div v-else class="panel scan-grid">
      <section
        class="card dropzone"
        @drop.prevent="onDrop"
        @dragover.prevent
      >
        <p class="dz-title">拖拽图片到此 · 或 Ctrl+V 粘贴</p>
        <div class="row dz-actions">
          <label class="btn btn-outline file-label">
            选择图片<input type="file" accept="image/*" class="hidden-file" @change="onFilePicked" />
          </label>
          <button class="btn btn-outline" @click="fromClipboardButton">读取剪贴板</button>
        </div>
        <div v-if="scanPreview" class="preview">
          <img :src="scanPreview" class="scan-img" alt="待识别图片" />
        </div>
      </section>

      <section class="card">
        <div class="label-row">
          <span class="label">识别结果</span>
          <button class="link-btn" :disabled="!scanResult" @click="copyResult">{{ copied ? '已复制 ✓' : '复制' }}</button>
        </div>
        <textarea v-model="scanResult" class="textarea" rows="8" readonly
          :placeholder="scanBusy ? '识别中…' : '识别到的内容会显示在此处'" />
        <p v-if="scanned && !scanResult && !scanBusy" class="hint bad">未在图片中识别到二维码</p>
      </section>
    </div>

    <p v-if="error" class="error">{{ error }}</p>
  </div>
</template>

<style scoped>
.tool { max-width: 1000px; margin: 0 auto; }
.lead { color: var(--fg-muted); margin-bottom: 16px; }

.tab-bar { display: flex; gap: 4px; border-bottom: 1px solid var(--border); margin-bottom: 16px; }
.tab {
  padding: 8px 16px; background: none; border: none; border-bottom: 2px solid transparent;
  margin-bottom: -1px; cursor: pointer; font-size: 13px; font-weight: 500;
  color: var(--fg-muted); font-family: inherit;
}
.tab:hover { color: var(--fg); }
.tab.active { color: var(--fg); border-bottom-color: var(--primary); }

.card {
  background: var(--card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 16px;
}
.gen-grid, .scan-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; align-items: start; }

.label-row { display: flex; align-items: center; justify-content: space-between; margin-bottom: 6px; }
.label { font-size: 12px; font-weight: 600; color: var(--fg-muted); }

.opts { margin-top: 12px; gap: 24px; align-items: flex-end; }
.field { display: flex; flex-direction: column; gap: 6px; }

.preview-card { display: flex; align-items: center; justify-content: center; min-height: 240px; }
.qr-wrap { display: flex; flex-direction: column; align-items: center; gap: 12px; }
.qr-img { background: #fff; border-radius: 6px; max-width: 100%; }
.placeholder { color: var(--fg-muted); font-size: 13px; }

.dropzone { display: flex; flex-direction: column; gap: 12px; }
.dz-title { margin: 0; color: var(--fg-muted); font-size: 13px; text-align: center; padding: 12px; border: 1px dashed var(--border); border-radius: 6px; }
.dz-actions { gap: 8px; }
.file-label { display: inline-flex; align-items: center; cursor: pointer; }
.hidden-file { display: none; }
.preview { margin-top: 4px; }
.scan-img { max-width: 100%; max-height: 280px; border-radius: 6px; border: 1px solid var(--border); }

.row { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
.hint { font-size: 12px; margin: 8px 0 0; }
.hint.bad { color: var(--danger); }

.input {
  padding: 6px 10px; border: 1px solid var(--border); border-radius: 4px;
  background: var(--bg); color: var(--fg);
  font: 13px ui-monospace, SFMono-Regular, Consolas, monospace;
}
.input:focus { outline: none; border-color: var(--primary); }

.textarea {
  width: 100%; padding: 8px 10px; border: 1px solid var(--border); border-radius: 4px;
  background: var(--bg); color: var(--fg);
  font: 13px/1.5 ui-monospace, SFMono-Regular, Consolas, monospace;
  resize: vertical; box-sizing: border-box; word-break: break-all;
}
.textarea:focus { outline: none; border-color: var(--primary); }

.btn {
  background: var(--primary); color: #fff; border: none;
  padding: 6px 14px; border-radius: 4px; cursor: pointer; font-size: 13px; font-weight: 500;
}
.btn:hover:not(:disabled) { background: var(--primary-hover); }
.btn-outline { background: transparent; color: var(--fg); border: 1px solid var(--border); }
.btn-outline:hover:not(:disabled) { border-color: var(--primary); color: var(--primary); }

.link-btn { background: none; border: none; padding: 4px 8px; font-size: 12px; color: var(--fg-muted); cursor: pointer; }
.link-btn:hover:not(:disabled) { color: var(--primary); }
.link-btn:disabled { opacity: 0.5; cursor: not-allowed; }

.error { color: var(--danger); margin: 12px 0 0; font-size: 13px; }

@media (max-width: 700px) {
  .gen-grid, .scan-grid { grid-template-columns: 1fr; }
}
</style>
