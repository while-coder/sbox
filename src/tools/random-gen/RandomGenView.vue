<script setup lang="ts">
import { ref, reactive, computed } from 'vue'
import {
  randomUuids, generatePasswords, randomString,
  buildCharset, entropyBits, type PasswordOptions,
} from './gen'

type Tab = 'password' | 'uuid' | 'string'
const activeTab = ref<Tab>('password')
const error = ref('')
const copied = ref(false)
const results = ref<string[]>([])

// ── 密码 ──────────────────────────────────────────────────
const pw = reactive<PasswordOptions>({
  length: 16,
  lower: true,
  upper: true,
  digits: true,
  symbols: true,
  excludeAmbiguous: false,
})
const pwCount = ref(5)

const pwCharsetSize = computed(() => buildCharset(pw).length)
const pwEntropy = computed(() => entropyBits(pwCharsetSize.value, pw.length))
const pwStrength = computed(() => {
  const e = pwEntropy.value
  if (e < 50) return { label: '弱', cls: 'bad' }
  if (e < 80) return { label: '中', cls: 'mid' }
  return { label: '强', cls: 'ok' }
})

// ── UUID ──────────────────────────────────────────────────
const uuidCount = ref(5)

// ── 随机字符串 ────────────────────────────────────────────
const strSet = ref('abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789')
const strLen = ref(24)
const strCount = ref(5)

function generate() {
  error.value = ''
  try {
    if (activeTab.value === 'password') {
      results.value = generatePasswords(pw, Math.max(1, pwCount.value))
    } else if (activeTab.value === 'uuid') {
      results.value = randomUuids(Math.max(1, uuidCount.value))
    } else {
      results.value = randomString(strSet.value, Math.max(1, strLen.value), Math.max(1, strCount.value))
    }
  } catch (e: any) {
    results.value = []
    error.value = String(e?.message || e)
  }
}

async function copyAll() {
  if (!results.value.length) return
  try {
    await navigator.clipboard.writeText(results.value.join('\n'))
    copied.value = true
    setTimeout(() => (copied.value = false), 2000)
  } catch (e: any) {
    error.value = `复制失败: ${String(e?.message || e)}`
  }
}

async function copyOne(value: string) {
  try { await navigator.clipboard.writeText(value) } catch { /* ignore */ }
}

const tabs: { key: Tab; label: string }[] = [
  { key: 'password', label: '密码' },
  { key: 'uuid', label: 'UUID' },
  { key: 'string', label: '随机字符串' },
]

function switchTab(t: Tab) { activeTab.value = t; results.value = []; error.value = '' }
</script>

<template>
  <div class="tool">
    <h2>随机生成器</h2>
    <p class="lead">基于系统强随机（crypto）生成密码、UUID 与随机字符串，全程本机。</p>

    <div class="tab-bar">
      <button v-for="t in tabs" :key="t.key" class="tab"
        :class="{ active: activeTab === t.key }" @click="switchTab(t.key)">
        {{ t.label }}
      </button>
    </div>

    <!-- 密码 -->
    <section v-if="activeTab === 'password'" class="card">
      <div class="row">
        <label class="field">
          <span class="label">长度</span>
          <input v-model.number="pw.length" type="number" min="1" max="256" class="input num" />
        </label>
        <label class="field">
          <span class="label">数量</span>
          <input v-model.number="pwCount" type="number" min="1" max="100" class="input num" />
        </label>
      </div>
      <div class="row checks">
        <label class="check"><input type="checkbox" v-model="pw.lower" /> 小写 a-z</label>
        <label class="check"><input type="checkbox" v-model="pw.upper" /> 大写 A-Z</label>
        <label class="check"><input type="checkbox" v-model="pw.digits" /> 数字 0-9</label>
        <label class="check"><input type="checkbox" v-model="pw.symbols" /> 符号</label>
        <label class="check"><input type="checkbox" v-model="pw.excludeAmbiguous" /> 排除易混淆 0O1lI</label>
      </div>
      <div class="meta">
        字符集 {{ pwCharsetSize }} 种 · 熵约 {{ pwEntropy }} bit ·
        强度 <span class="badge" :class="pwStrength.cls">{{ pwStrength.label }}</span>
      </div>
    </section>

    <!-- UUID -->
    <section v-else-if="activeTab === 'uuid'" class="card">
      <label class="field">
        <span class="label">数量</span>
        <input v-model.number="uuidCount" type="number" min="1" max="100" class="input num" />
      </label>
      <p class="meta">生成 UUID v4（随机版本）。</p>
    </section>

    <!-- 随机字符串 -->
    <section v-else class="card">
      <div class="field-block">
        <span class="label">字符集</span>
        <input v-model="strSet" class="input input-grow mono" placeholder="可用字符" />
      </div>
      <div class="row">
        <label class="field">
          <span class="label">长度</span>
          <input v-model.number="strLen" type="number" min="1" max="1024" class="input num" />
        </label>
        <label class="field">
          <span class="label">数量</span>
          <input v-model.number="strCount" type="number" min="1" max="100" class="input num" />
        </label>
      </div>
    </section>

    <div class="actions">
      <button class="btn" @click="generate">生成</button>
      <button class="btn btn-outline" :disabled="!results.length" @click="copyAll">
        {{ copied ? '已复制全部 ✓' : '复制全部' }}
      </button>
    </div>

    <section v-if="results.length" class="card results">
      <div class="result-row" v-for="(r, i) in results" :key="i" @click="copyOne(r)" title="点击复制">
        <span class="rv mono">{{ r }}</span>
        <span class="copy-hint">复制</span>
      </div>
    </section>

    <p v-if="error" class="error">{{ error }}</p>
  </div>
</template>

<style scoped>
.tool { max-width: 880px; margin: 0 auto; }
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
  margin-bottom: 12px;
}

.row { display: flex; align-items: flex-end; gap: 16px; flex-wrap: wrap; }
.checks { margin-top: 14px; gap: 16px; }
.field { display: flex; flex-direction: column; gap: 6px; }
.field-block { display: flex; flex-direction: column; gap: 6px; margin-bottom: 14px; }
.label { font-size: 12px; font-weight: 600; color: var(--fg-muted); }
.check { font-size: 13px; color: var(--fg); display: inline-flex; align-items: center; gap: 5px; cursor: pointer; }

.input {
  padding: 6px 10px;
  border: 1px solid var(--border);
  border-radius: 4px;
  background: var(--bg);
  color: var(--fg);
  font: 13px ui-monospace, SFMono-Regular, Consolas, monospace;
}
.input:focus { outline: none; border-color: var(--primary); }
.num { width: 90px; }
.input-grow { width: 100%; box-sizing: border-box; }
.mono { font-family: ui-monospace, SFMono-Regular, Consolas, monospace; }

.meta { margin-top: 14px; font-size: 12px; color: var(--fg-muted); }
.badge { font-size: 11px; padding: 1px 8px; border-radius: 10px; font-weight: 600; }
.badge.ok { background: rgba(45,164,78,0.15); color: var(--success); }
.badge.mid { background: rgba(218,165,32,0.18); color: #b8860b; }
.badge.bad { background: rgba(207,34,46,0.15); color: var(--danger); }

.actions { display: flex; gap: 8px; margin-bottom: 12px; }

.results { display: flex; flex-direction: column; gap: 4px; }
.result-row {
  display: flex; align-items: center; justify-content: space-between;
  padding: 7px 10px; border-radius: 4px; cursor: pointer;
}
.result-row:hover { background: var(--bg); }
.result-row:hover .copy-hint { opacity: 1; }
.rv { overflow-wrap: anywhere; }
.copy-hint { font-size: 11px; color: var(--primary); opacity: 0; flex: 0 0 auto; margin-left: 12px; }

.btn {
  background: var(--primary); color: #fff; border: none;
  padding: 6px 14px; border-radius: 4px; cursor: pointer;
  font-size: 13px; font-weight: 500;
}
.btn:hover:not(:disabled) { background: var(--primary-hover); }
.btn:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-outline { background: transparent; color: var(--fg); border: 1px solid var(--border); }
.btn-outline:hover:not(:disabled) { border-color: var(--primary); color: var(--primary); }

.error { color: var(--danger); margin: 4px 0 0; font-size: 13px; }
</style>
