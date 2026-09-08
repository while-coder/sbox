<script setup lang="ts">
import { ref, computed, shallowRef } from 'vue'
import JsonNode from './JsonNode.vue'

const input = ref('')
const error = ref('')
const copied = ref(false)
const indent = ref(2)

// 解析后的值（用 shallowRef 避免大对象被深度代理）
const parsed = shallowRef<{ ok: boolean; value: unknown }>({ ok: false, value: null })
const hasTree = computed(() => parsed.value.ok)

function parse() {
  error.value = ''
  const t = input.value.trim()
  if (!t) { parsed.value = { ok: false, value: null }; return }
  try {
    parsed.value = { ok: true, value: JSON.parse(t) }
  } catch (e: any) {
    parsed.value = { ok: false, value: null }
    error.value = String(e?.message || e)
  }
}

function beautify() {
  error.value = ''
  try {
    input.value = JSON.stringify(JSON.parse(input.value), null, indent.value)
    parse()
  } catch (e: any) { error.value = String(e?.message || e) }
}

function minify() {
  error.value = ''
  try {
    input.value = JSON.stringify(JSON.parse(input.value))
    parse()
  } catch (e: any) { error.value = String(e?.message || e) }
}

function sortKeys() {
  error.value = ''
  try {
    const sorted = sortDeep(JSON.parse(input.value))
    input.value = JSON.stringify(sorted, null, indent.value)
    parse()
  } catch (e: any) { error.value = String(e?.message || e) }
}

function sortDeep(v: unknown): unknown {
  if (Array.isArray(v)) return v.map(sortDeep)
  if (v && typeof v === 'object') {
    return Object.fromEntries(
      Object.keys(v as object).sort().map(k => [k, sortDeep((v as Record<string, unknown>)[k])]),
    )
  }
  return v
}

async function copyInput() {
  if (!input.value) return
  try {
    await navigator.clipboard.writeText(input.value)
    copied.value = true
    setTimeout(() => (copied.value = false), 2000)
  } catch (e: any) { error.value = `复制失败: ${String(e?.message || e)}` }
}

const stats = computed(() => {
  if (!parsed.value.ok) return ''
  const s = input.value.length
  return `${s.toLocaleString()} 字符`
})
</script>

<template>
  <div class="tool">
    <h2>JSON 格式化 / 查看器</h2>
    <p class="lead">美化、压缩、按键排序，并以树状查看；悬停任意节点可复制其值或访问路径。</p>

    <section class="card toolbar">
      <button class="btn" @click="beautify">美化</button>
      <button class="btn" @click="minify">压缩</button>
      <button class="btn btn-outline" @click="sortKeys">键排序</button>
      <button class="btn btn-outline" @click="parse">解析查看</button>
      <div class="seg">
        <span class="label">缩进</span>
        <select v-model.number="indent" class="input indent-select">
          <option :value="2">2</option>
          <option :value="4">4</option>
        </select>
      </div>
      <div class="spacer" />
      <span v-if="stats" class="stats">{{ stats }}</span>
      <button class="link-btn" :disabled="!input" @click="copyInput">{{ copied ? '已复制 ✓' : '复制' }}</button>
      <button class="link-btn" :disabled="!input" @click="input = ''">清空</button>
    </section>

    <div class="io-grid">
      <div class="io-col">
        <div class="label-row"><span class="label">输入</span></div>
        <textarea v-model="input" class="textarea" rows="22" placeholder="在此粘贴 JSON…" @blur="parse" />
      </div>
      <div class="io-col">
        <div class="label-row"><span class="label">树状视图</span></div>
        <div class="tree card">
          <JsonNode v-if="hasTree" :k="null" :value="parsed.value" path="$" :depth="0" />
          <p v-else class="placeholder">{{ error ? '解析失败' : '点击「解析查看」或在输入框失焦后展示树状结构' }}</p>
        </div>
      </div>
    </div>

    <p v-if="error" class="error">{{ error }}</p>
  </div>
</template>

<style scoped>
.tool { max-width: 1100px; margin: 0 auto; }
.lead { color: var(--fg-muted); margin-bottom: 16px; }

.card {
  background: var(--card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 12px 16px;
}
.toolbar { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; margin-bottom: 12px; }
.seg { display: flex; align-items: center; gap: 6px; }
.spacer { flex: 1 1 auto; }
.label { font-size: 12px; font-weight: 600; color: var(--fg-muted); }
.indent-select { flex: 0 0 64px; }
.stats { font-size: 12px; color: var(--fg-muted); }

.io-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
.io-col { display: flex; flex-direction: column; }
.label-row { margin-bottom: 6px; }

.tree { padding: 12px; overflow: auto; max-height: 560px; min-height: 120px; }
.placeholder { color: var(--fg-muted); font-size: 13px; margin: 0; }

.input {
  padding: 6px 10px;
  border: 1px solid var(--border);
  border-radius: 4px;
  background: var(--bg);
  color: var(--fg);
  font: 13px ui-monospace, SFMono-Regular, Consolas, monospace;
}
.input:focus { outline: none; border-color: var(--primary); }

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
  background: var(--primary); color: #fff; border: none;
  padding: 6px 14px; border-radius: 4px; cursor: pointer;
  font-size: 13px; font-weight: 500;
}
.btn:hover:not(:disabled) { background: var(--primary-hover); }
.btn-outline { background: transparent; color: var(--fg); border: 1px solid var(--border); }
.btn-outline:hover:not(:disabled) { border-color: var(--primary); color: var(--primary); }

.link-btn { background: none; border: none; padding: 4px 8px; font-size: 12px; color: var(--fg-muted); cursor: pointer; }
.link-btn:hover:not(:disabled) { color: var(--primary); }
.link-btn:disabled { opacity: 0.5; cursor: not-allowed; }

.error { color: var(--danger); margin: 12px 0 0; font-size: 13px; }

@media (max-width: 700px) {
  .io-grid { grid-template-columns: 1fr; }
}
</style>
