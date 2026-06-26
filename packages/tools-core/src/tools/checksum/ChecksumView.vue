<script setup lang="ts">
import { ref, computed } from 'vue'
import { hashFile, hashString, formatBytes, type HashAlgorithm } from '../codec/codec'

type Source = 'file' | 'text'
const source = ref<Source>('file')
const algo = ref<HashAlgorithm>('sha256')
const error = ref('')
const busy = ref(false)
const copied = ref(false)

const picked = ref<File | null>(null)
const textInput = ref('')
const actual = ref('')
const expected = ref('')

const fileInfo = computed(() => {
  const f = picked.value
  return f ? `${f.name} · ${formatBytes(f.size)}` : ''
})

// 期望值与实际值比对（忽略大小写与空白）
const verdict = computed<null | boolean>(() => {
  if (!actual.value || !expected.value.trim()) return null
  return actual.value.toLowerCase() === expected.value.trim().toLowerCase()
})

function onPick(e: Event) {
  const f = (e.target as HTMLInputElement).files?.[0]
  if (!f) return
  picked.value = f
  actual.value = ''
}

async function compute() {
  error.value = ''
  busy.value = true
  try {
    if (source.value === 'file') {
      if (!picked.value) { error.value = '请先选择文件'; return }
      actual.value = await hashFile(picked.value, algo.value)
    } else {
      actual.value = await hashString(textInput.value, algo.value)
    }
  } catch (e: any) {
    actual.value = ''
    error.value = String(e?.message || e)
  } finally {
    busy.value = false
  }
}

async function copyActual() {
  if (!actual.value) return
  try {
    await navigator.clipboard.writeText(actual.value)
    copied.value = true
    setTimeout(() => (copied.value = false), 2000)
  } catch (e: any) {
    error.value = `复制失败: ${String(e?.message || e)}`
  }
}

const algos: { v: HashAlgorithm; label: string }[] = [
  { v: 'md5', label: 'MD5' },
  { v: 'sha1', label: 'SHA-1' },
  { v: 'sha256', label: 'SHA-256' },
  { v: 'sha512', label: 'SHA-512' },
]
</script>

<template>
  <div class="tool">
    <h2>文件校验 (Checksum)</h2>
    <p class="lead">计算文件或文本的哈希，并与官方给出的期望值比对，确认下载是否完整、未被篡改。本机计算。</p>

    <section class="card">
      <div class="row row-between">
        <div class="seg">
          <label class="radio"><input type="radio" value="file" v-model="source" /> 文件</label>
          <label class="radio"><input type="radio" value="text" v-model="source" /> 文本</label>
        </div>
        <select v-model="algo" class="input algo-select">
          <option v-for="a in algos" :key="a.v" :value="a.v">{{ a.label }}</option>
        </select>
      </div>

      <div v-if="source === 'file'" class="row src-row">
        <input type="file" @change="onPick" />
        <button class="btn" :disabled="!picked || busy" @click="compute">
          {{ busy ? '计算中…' : '计算' }}
        </button>
      </div>
      <div v-else class="src-block">
        <textarea v-model="textInput" class="textarea" rows="4" placeholder="在此输入要校验的文本…" />
        <button class="btn" :disabled="busy" @click="compute">{{ busy ? '计算中…' : '计算' }}</button>
      </div>
      <div v-if="source === 'file' && fileInfo" class="meta">{{ fileInfo }}</div>
    </section>

    <section class="card">
      <div class="label-row">
        <span class="label">计算结果</span>
        <button class="link-btn" :disabled="!actual" @click="copyActual">
          {{ copied ? '已复制 ✓' : '复制' }}
        </button>
      </div>
      <input v-model="actual" readonly class="input mono input-full" placeholder="结果将显示在此处" />

      <div class="label-row label-row-top">
        <span class="label">期望值（粘贴官方哈希进行比对，可选）</span>
      </div>
      <input v-model="expected" class="input mono input-full" placeholder="粘贴期望的哈希值…" />

      <div v-if="verdict !== null" class="verdict" :class="verdict ? 'ok' : 'bad'">
        {{ verdict ? '✓ 匹配：哈希一致' : '✗ 不匹配：哈希不一致' }}
      </div>
    </section>

    <p v-if="error" class="error">{{ error }}</p>
  </div>
</template>

<style scoped>
.tool { max-width: 880px; margin: 0 auto; }
.lead { color: var(--fg-muted); margin-bottom: 16px; }

.card {
  background: var(--card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 16px;
  margin-bottom: 12px;
}

.row { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
.row-between { justify-content: space-between; margin-bottom: 12px; }
.src-row { margin-top: 4px; }
.src-block { display: flex; flex-direction: column; gap: 10px; }
.seg { display: flex; gap: 16px; }
.radio { font-size: 13px; color: var(--fg); display: inline-flex; align-items: center; gap: 5px; cursor: pointer; }
.algo-select { flex: 0 0 140px; }

.label-row { display: flex; align-items: center; justify-content: space-between; margin-bottom: 6px; }
.label-row-top { margin-top: 16px; }
.label { font-size: 12px; font-weight: 600; color: var(--fg-muted); }

.input {
  padding: 6px 10px;
  border: 1px solid var(--border);
  border-radius: 4px;
  background: var(--bg);
  color: var(--fg);
  font: 13px ui-monospace, SFMono-Regular, Consolas, monospace;
}
.input:focus { outline: none; border-color: var(--primary); }
.input-full { width: 100%; box-sizing: border-box; }
.mono { font-family: ui-monospace, SFMono-Regular, Consolas, monospace; }

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

.meta { margin-top: 10px; font-size: 12px; color: var(--fg-muted); font-family: ui-monospace, SFMono-Regular, Consolas, monospace; }

.verdict { margin-top: 14px; font-size: 14px; font-weight: 600; padding: 8px 12px; border-radius: 6px; }
.verdict.ok { color: var(--success); background: rgba(45,164,78,0.12); }
.verdict.bad { color: var(--danger); background: rgba(207,34,46,0.12); }

.btn {
  background: var(--primary); color: #fff; border: none;
  padding: 6px 14px; border-radius: 4px; cursor: pointer;
  font-size: 13px; font-weight: 500; align-self: flex-start;
}
.btn:hover:not(:disabled) { background: var(--primary-hover); }
.btn:disabled { opacity: 0.5; cursor: not-allowed; }

.link-btn {
  background: none; border: none; padding: 4px 8px;
  font-size: 12px; color: var(--fg-muted); cursor: pointer;
}
.link-btn:hover:not(:disabled) { color: var(--primary); }
.link-btn:disabled { opacity: 0.5; cursor: not-allowed; }

.error { color: var(--danger); margin: 4px 0 0; font-size: 13px; }
</style>
