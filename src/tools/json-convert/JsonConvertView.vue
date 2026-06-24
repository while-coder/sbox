<script setup lang="ts">
import { ref, computed } from 'vue'
import { convert, validate, detectFormat, FORMAT_LABELS, type DataFormat } from './convert'

const formats: DataFormat[] = ['json', 'yaml', 'toml']

const from = ref<DataFormat | 'auto'>('auto')
const to = ref<DataFormat>('yaml')
const indent = ref(2)
const input = ref('')
const output = ref('')
const error = ref('')
const validMsg = ref('')
const validOk = ref(false)
const copied = ref(false)

// 自动检测：from 为 auto 时实时识别输入格式
const detected = computed<DataFormat | null>(() =>
  from.value === 'auto' ? detectFormat(input.value) : null,
)
// 实际生效的源格式
const effectiveFrom = computed<DataFormat | null>(() =>
  from.value === 'auto' ? detected.value : from.value,
)

function doConvert() {
  error.value = ''
  validMsg.value = ''
  const src = effectiveFrom.value
  if (!src) { output.value = ''; error.value = '无法自动识别源格式，请手动选择'; return }
  try {
    output.value = convert(input.value, src, to.value, indent.value)
  } catch (e: any) {
    output.value = ''
    error.value = String(e?.message || e)
  }
}

function doValidate() {
  error.value = ''
  const src = effectiveFrom.value
  if (!src) { validOk.value = false; validMsg.value = '无法自动识别源格式'; return }
  const r = validate(input.value, src)
  validOk.value = r.ok
  validMsg.value = r.message
}

function swap() {
  // from 为 auto 时，用检测结果作为交换后的目标格式
  const resolvedFrom = effectiveFrom.value ?? 'json'
  from.value = to.value
  to.value = resolvedFrom
  if (output.value) {
    input.value = output.value
    output.value = ''
  }
}

async function copyOutput() {
  if (!output.value) return
  try {
    await navigator.clipboard.writeText(output.value)
    copied.value = true
    setTimeout(() => (copied.value = false), 2000)
  } catch (e: any) {
    error.value = `复制失败: ${String(e?.message || e)}`
  }
}

const sameFormat = computed(() => effectiveFrom.value === to.value)
const fromLabel = computed(() =>
  from.value === 'auto'
    ? (detected.value ? FORMAT_LABELS[detected.value] : '自动')
    : FORMAT_LABELS[from.value],
)
</script>

<template>
  <div class="tool">
    <h2>JSON / YAML / TOML 互转</h2>
    <p class="lead">三种配置格式互转与格式化校验。所有解析在本机完成。</p>

    <section class="card toolbar">
      <div class="seg">
        <span class="label">源</span>
        <select v-model="from" class="input">
          <option value="auto">自动检测</option>
          <option v-for="f in formats" :key="f" :value="f">{{ FORMAT_LABELS[f] }}</option>
        </select>
        <span v-if="from === 'auto'" class="detected" :class="{ none: !detected }">
          {{ detected ? `→ ${FORMAT_LABELS[detected]}` : (input.trim() ? '? 无法识别' : '') }}
        </span>
      </div>
      <button class="btn btn-outline" @click="swap" title="交换源/目标">⇄</button>
      <div class="seg">
        <span class="label">目标</span>
        <select v-model="to" class="input">
          <option v-for="f in formats" :key="f" :value="f">{{ FORMAT_LABELS[f] }}</option>
        </select>
      </div>
      <div class="seg">
        <span class="label">缩进</span>
        <select v-model.number="indent" class="input indent-select">
          <option :value="2">2</option>
          <option :value="4">4</option>
        </select>
      </div>
      <div class="spacer" />
      <button class="btn btn-outline" @click="doValidate">校验源</button>
      <button class="btn" @click="doConvert" :disabled="sameFormat">转换</button>
    </section>

    <p v-if="sameFormat" class="hint">源与目标格式相同，仅作格式化（美化）。</p>
    <p v-if="validMsg" class="valid" :class="{ ok: validOk, bad: !validOk }">
      {{ validOk ? '✓' : '✗' }} {{ validMsg }}
    </p>

    <div class="io-grid">
      <div class="io-col">
        <div class="label-row">
          <span class="label">{{ fromLabel }} 输入</span>
          <button class="link-btn" @click="input = ''">清空</button>
        </div>
        <textarea v-model="input" class="textarea" rows="16" placeholder="在此粘贴或输入…" />
      </div>
      <div class="io-col">
        <div class="label-row">
          <span class="label">{{ FORMAT_LABELS[to] }} 输出</span>
          <button class="link-btn" :disabled="!output" @click="copyOutput">
            {{ copied ? '已复制 ✓' : '复制' }}
          </button>
        </div>
        <textarea v-model="output" class="textarea" rows="16" readonly placeholder="转换结果将显示在此处" />
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
.toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
  margin-bottom: 12px;
}
.seg { display: flex; align-items: center; gap: 6px; }
.spacer { flex: 1 1 auto; }
.label { font-size: 12px; font-weight: 600; color: var(--fg-muted); }
.indent-select { flex: 0 0 64px; }
.detected { font-size: 12px; font-weight: 600; color: var(--success); }
.detected.none { color: var(--fg-muted); }

.io-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
.io-col { display: flex; flex-direction: column; }
.label-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}

.hint { font-size: 12px; color: var(--fg-muted); margin: 0 0 12px; }
.valid { font-size: 13px; margin: 0 0 12px; }
.valid.ok { color: var(--success); }
.valid.bad { color: var(--danger); }

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
.btn-outline { background: transparent; color: var(--fg); border: 1px solid var(--border); }
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

.error { color: var(--danger); margin: 12px 0 0; font-size: 13px; }

@media (max-width: 700px) {
  .io-grid { grid-template-columns: 1fr; }
}
</style>
