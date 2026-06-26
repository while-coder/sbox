<script setup lang="ts">
import { ref, computed } from 'vue'
import {
  stringToBase64, base64ToString, base64ToBase64Url, base64UrlToBase64,
  base64ToBytes,
  stringToHex, hexToString,
  urlEncode, urlDecode,
  htmlEncode, htmlDecode,
  unicodeEscape, unicodeUnescape,
  formatBytes,
  hashString, hashFile,
  fileToBase64,
  type HashAlgorithm,
} from './codec'
import { getPlatform } from '../../platform'

type Tab = 'text' | 'base64' | 'hash'

const activeTab = ref<Tab>('text')
const error = ref('')
const copiedKey = ref('')

// ── 共享输入 / 输出（文本 & Base64 tab 共用）──────────────
const textInput = ref('')
const textOutput = ref('')
const showIO = computed(() => activeTab.value === 'text' || activeTab.value === 'base64')

function safeRun(fn: () => string) {
  error.value = ''
  try { textOutput.value = fn() }
  catch (e: any) { error.value = String(e?.message || e) }
}

function swapTextIO() {
  const v = textOutput.value
  textOutput.value = textInput.value
  textInput.value = v
}

// ── 哈希（文本 / 文件 → 摘要，单向）──────────────────────
const hashSource = ref<'text' | 'file'>('text')
const hashAlgo = ref<HashAlgorithm>('sha256')
const hashText = ref('')
const hashFilePicked = ref<File | null>(null)
const hashOutput = ref('')
const hashBusy = ref(false)

const hashFileInfo = computed(() => {
  const f = hashFilePicked.value
  return f ? `${f.name} · ${formatBytes(f.size)}` : ''
})

function onHashFilePicked(e: Event) {
  const f = (e.target as HTMLInputElement).files?.[0]
  if (!f) return
  hashFilePicked.value = f
  hashOutput.value = ''
}

async function doHash() {
  error.value = ''
  hashBusy.value = true
  try {
    if (hashSource.value === 'text') {
      hashOutput.value = await hashString(hashText.value, hashAlgo.value)
    } else {
      if (!hashFilePicked.value) { error.value = '请先选择文件'; return }
      hashOutput.value = await hashFile(hashFilePicked.value, hashAlgo.value)
    }
  } catch (e: any) {
    hashOutput.value = ''
    error.value = String(e?.message || e)
  } finally {
    hashBusy.value = false
  }
}

// ── 文件 ⇄ Base64 ────────────────────────────────────────
const encFile = ref<File | null>(null)
const encB64 = ref('')
const encBusy = ref(false)
const encFileInfo = computed(() => {
  const f = encFile.value
  return f ? `${f.name} · ${formatBytes(f.size)}` : ''
})

function onEncFilePicked(e: Event) {
  const f = (e.target as HTMLInputElement).files?.[0]
  if (!f) return
  encFile.value = f
  encB64.value = ''
}

async function doEncode() {
  if (!encFile.value) return
  error.value = ''
  encBusy.value = true
  try { encB64.value = await fileToBase64(encFile.value) }
  catch (e: any) { error.value = String(e?.message || e) }
  finally { encBusy.value = false }
}

async function saveEncTxt() {
  try { await getPlatform().saveText(encB64.value, (encFile.value?.name || 'file') + '.b64.txt') }
  catch (e: any) { error.value = String(e?.message || e) }
}

const decB64Input = ref('')
const decFilename = ref('decoded.bin')

async function doDecodeSave() {
  if (!decB64Input.value) return
  error.value = ''
  try {
    const b64 = decB64Input.value.includes(',')
      ? decB64Input.value.split(',', 2)[1]
      : decB64Input.value
    await getPlatform().saveBinary(base64ToBytes(b64), decFilename.value || 'decoded.bin', 'application/octet-stream')
  } catch (e: any) { error.value = String(e?.message || e) }
}

// ── 字节大小 ─────────────────────────────────────────────
const bytesInput = ref('')
const bytesOutput = ref('')

function fmtBytes() {
  error.value = ''
  const n = Number(bytesInput.value)
  if (!Number.isFinite(n)) { error.value = '请输入有效数字'; return }
  bytesOutput.value = formatBytes(n)
}

// ── 复制 ────────────────────────────────────────────────
async function copyValue(key: string, value: string) {
  if (!value) return
  try {
    await navigator.clipboard.writeText(value)
    copiedKey.value = key
    setTimeout(() => { if (copiedKey.value === key) copiedKey.value = '' }, 2000)
  } catch (e: any) {
    error.value = `复制失败: ${String(e?.message || e)}`
  }
}

const tabs: { key: Tab; label: string }[] = [
  { key: 'text', label: '文本' },
  { key: 'base64', label: 'Base64' },
  { key: 'hash', label: '哈希' },
]
</script>

<template>
  <div class="codec">
    <h2>编解码工具</h2>
    <p class="lead">常见编解码与哈希。所有计算在本机完成，文件不会离开本机。</p>

    <div class="tab-bar">
      <button
        v-for="t in tabs"
        :key="t.key"
        class="tab"
        :class="{ active: activeTab === t.key }"
        @click="activeTab = t.key"
      >{{ t.label }}</button>
    </div>

    <!-- 共享输入 / 输出（文本 & Base64）-->
    <section v-if="showIO" class="card io-card">
      <div class="io-grid">
        <div class="io-col">
          <div class="label-row">
            <span class="label">输入</span>
            <button class="link-btn" @click="textInput = ''">清空</button>
          </div>
          <textarea v-model="textInput" class="textarea" rows="7" placeholder="在此粘贴或输入文本…" />
        </div>
        <div class="io-col">
          <div class="label-row">
            <span class="label">输出</span>
            <span class="actions-inline">
              <button class="link-btn" @click="swapTextIO" :disabled="!textOutput">⇄ 交换</button>
              <button class="link-btn" @click="copyValue('textOutput', textOutput)" :disabled="!textOutput">
                {{ copiedKey === 'textOutput' ? '已复制 ✓' : '复制' }}
              </button>
            </span>
          </div>
          <textarea v-model="textOutput" class="textarea" rows="7" readonly />
        </div>
      </div>
    </section>

    <!-- ===== 文本 Tab ===== -->
    <div v-if="activeTab === 'text'" class="panel">
      <div class="op-grid">
        <section class="card op-card">
          <h3 class="group-title">URL</h3>
          <div class="btn-col">
            <button class="btn" @click="safeRun(() => urlEncode(textInput))">URL Encode</button>
            <button class="btn" @click="safeRun(() => urlDecode(textInput))">URL Decode</button>
          </div>
        </section>

        <section class="card op-card">
          <h3 class="group-title">Hex</h3>
          <div class="btn-col">
            <button class="btn" @click="safeRun(() => stringToHex(textInput))">字符串 → Hex</button>
            <button class="btn" @click="safeRun(() => hexToString(textInput))">Hex → 字符串</button>
          </div>
        </section>

        <section class="card op-card">
          <h3 class="group-title">HTML</h3>
          <div class="btn-col">
            <button class="btn" @click="safeRun(() => htmlEncode(textInput))">HTML Escape</button>
            <button class="btn" @click="safeRun(() => htmlDecode(textInput))">HTML Unescape</button>
          </div>
        </section>

        <section class="card op-card">
          <h3 class="group-title">Unicode</h3>
          <div class="btn-col">
            <button class="btn" @click="safeRun(() => unicodeEscape(textInput))">Unicode Escape</button>
            <button class="btn" @click="safeRun(() => unicodeUnescape(textInput))">Unicode Unescape</button>
          </div>
        </section>
      </div>
    </div>

    <!-- ===== Base64 Tab ===== -->
    <div v-if="activeTab === 'base64'" class="panel">
      <section class="card">
        <h3 class="group-title">字符串 ⇄ Base64（作用于上方输入/输出）</h3>
        <div class="row">
          <button class="btn" @click="safeRun(() => stringToBase64(textInput))">字符串 → Base64</button>
          <button class="btn" @click="safeRun(() => base64ToString(textInput))">Base64 → 字符串</button>
          <button class="btn btn-outline" @click="safeRun(() => base64ToBase64Url(textInput))">Base64 → URL-safe</button>
          <button class="btn btn-outline" @click="safeRun(() => base64UrlToBase64(textInput))">URL-safe → Base64</button>
        </div>
      </section>

      <div class="file-grid">
        <section class="card">
          <h3 class="group-title">文件 → Base64</h3>
          <div class="row">
            <input type="file" @change="onEncFilePicked" />
            <button class="btn" :disabled="!encFile || encBusy" @click="doEncode">{{ encBusy ? '编码中…' : '编码' }}</button>
          </div>
          <div v-if="encFileInfo" class="meta meta-block">{{ encFileInfo }}</div>
          <textarea v-model="encB64" class="textarea" rows="6" readonly placeholder="编码结果将显示在此处" />
          <div v-if="encB64" class="row">
            <button class="link-btn" @click="copyValue('encB64', encB64)">
              {{ copiedKey === 'encB64' ? '已复制 ✓' : '复制' }}
            </button>
            <button class="link-btn" @click="saveEncTxt">另存为 .b64.txt</button>
          </div>
        </section>

        <section class="card">
          <h3 class="group-title">Base64 → 文件</h3>
          <textarea v-model="decB64Input" class="textarea" rows="6" placeholder="在此粘贴 Base64 文本（可包含 data URL 头）" />
          <div class="row row-spaced">
            <input v-model="decFilename" class="input filename-input" placeholder="filename.bin" />
            <button class="btn" :disabled="!decB64Input" @click="doDecodeSave">保存文件…</button>
          </div>
        </section>
      </div>
    </div>

    <!-- ===== 哈希 Tab ===== -->
    <div v-if="activeTab === 'hash'" class="panel">
      <section class="card">
        <div class="row row-between">
          <div class="seg">
            <h3 class="group-title group-title-inline">哈希</h3>
            <label class="radio"><input type="radio" value="text" v-model="hashSource" /> 文本</label>
            <label class="radio"><input type="radio" value="file" v-model="hashSource" /> 文件</label>
          </div>
          <select v-model="hashAlgo" class="input algo-select">
            <option value="md5">MD5</option>
            <option value="sha1">SHA-1</option>
            <option value="sha256">SHA-256</option>
            <option value="sha512">SHA-512</option>
          </select>
        </div>

        <textarea v-if="hashSource === 'text'" v-model="hashText" class="textarea" rows="4" placeholder="在此输入要哈希的文本…" />
        <div v-else class="row src-row"><input type="file" @change="onHashFilePicked" /></div>
        <div v-if="hashSource === 'file' && hashFileInfo" class="meta meta-block">{{ hashFileInfo }}</div>

        <div class="row row-spaced">
          <button class="btn" :disabled="hashBusy" @click="doHash">{{ hashBusy ? '计算中…' : '计算' }}</button>
          <input v-model="hashOutput" readonly class="input mono input-grow" placeholder="结果将显示在此处" />
          <button v-if="hashOutput" class="link-btn" @click="copyValue('hashOut', hashOutput)">
            {{ copiedKey === 'hashOut' ? '已复制 ✓' : '复制' }}
          </button>
        </div>
      </section>

      <section class="card">
        <h3 class="group-title">字节大小</h3>
        <div class="row">
          <input v-model="bytesInput" class="input input-grow" placeholder="1048576" />
          <button class="btn" @click="fmtBytes">格式化</button>
          <input v-model="bytesOutput" readonly class="input input-grow" />
        </div>
      </section>
    </div>

    <p v-if="error" class="error">{{ error }}</p>
  </div>
</template>

<style scoped>
.codec { max-width: 1100px; margin: 0 auto; }
.lead { color: var(--fg-muted); margin-bottom: 16px; }

.tab-bar {
  display: flex;
  gap: 4px;
  border-bottom: 1px solid var(--border);
  margin-bottom: 16px;
}
.tab {
  padding: 8px 16px;
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  margin-bottom: -1px;
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
  color: var(--fg-muted);
  font-family: inherit;
}
.tab:hover { color: var(--fg); }
.tab.active { color: var(--fg); border-bottom-color: var(--primary); }

.panel { display: flex; flex-direction: column; gap: 12px; }
.card {
  background: var(--card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 16px;
}
.io-card { margin-bottom: 12px; }
.group-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--fg-muted);
  margin: 0 0 12px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
.group-title-inline { margin: 0; }

/* IO 区 */
.io-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}
.io-col { display: flex; flex-direction: column; }
.label-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}
.label {
  font-size: 12px;
  font-weight: 600;
  color: var(--fg-muted);
}
.actions-inline { display: inline-flex; gap: 8px; }

/* 文本操作网格 */
.op-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
}
.op-card { padding: 12px 14px; }
.btn-col { display: flex; flex-direction: column; gap: 6px; }

/* 哈希 / 文件 */
.seg { display: flex; align-items: center; gap: 14px; }
.radio { font-size: 13px; color: var(--fg); display: inline-flex; align-items: center; gap: 5px; cursor: pointer; }
.algo-select { flex: 0 0 140px; }
.src-row { margin-top: 4px; }
.file-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }

/* 通用行布局 */
.row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.row-spaced { margin-top: 12px; }
.row-between { justify-content: space-between; margin-bottom: 12px; }
.input-grow { flex: 1 1 0; min-width: 0; }
.filename-input { flex: 0 0 240px; }

.input {
  padding: 6px 10px;
  border: 1px solid var(--border);
  border-radius: 4px;
  background: var(--bg);
  color: var(--fg);
  font: 13px ui-monospace, SFMono-Regular, Consolas, monospace;
}
.input:focus { outline: none; border-color: var(--primary); }
.input.mono { font-family: ui-monospace, SFMono-Regular, Consolas, monospace; }

.textarea {
  width: 100%;
  padding: 8px 10px;
  border: 1px solid var(--border);
  border-radius: 4px;
  background: var(--bg);
  color: var(--fg);
  font: 13px/1.5 ui-monospace, SFMono-Regular, Consolas, monospace;
  resize: vertical;
  box-sizing: border-box;
}
.textarea:focus { outline: none; border-color: var(--primary); }

.btn {
  background: var(--primary);
  color: #fff;
  border: none;
  padding: 6px 14px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
}
.btn:hover:not(:disabled) { background: var(--primary-hover); }
.btn:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-outline {
  background: transparent;
  color: var(--fg);
  border: 1px solid var(--border);
}
.btn-outline:hover:not(:disabled) { border-color: var(--primary); color: var(--primary); }

.link-btn {
  background: none;
  border: none;
  padding: 4px 8px;
  font-size: 12px;
  color: var(--fg-muted);
  cursor: pointer;
}
.link-btn:hover:not(:disabled) { color: var(--primary); }
.link-btn:disabled { opacity: 0.5; cursor: not-allowed; }

.meta {
  font-size: 12px;
  color: var(--fg-muted);
  font-family: ui-monospace, SFMono-Regular, Consolas, monospace;
}
.meta-block { margin: 8px 0; }

.error { color: var(--danger); margin: 12px 0 0; font-size: 13px; }

@media (max-width: 960px) {
  .op-grid { grid-template-columns: repeat(2, 1fr); }
}
@media (max-width: 700px) {
  .io-grid,
  .file-grid,
  .op-grid { grid-template-columns: 1fr; }
}
</style>
