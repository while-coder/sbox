<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'

// ── 实时当前时间 ──────────────────────────────────────────
const nowMs = ref(Date.now())
let timer: number | undefined
onMounted(() => { timer = window.setInterval(() => (nowMs.value = Date.now()), 1000) })
onUnmounted(() => { if (timer) clearInterval(timer) })

const nowSeconds = computed(() => Math.floor(nowMs.value / 1000))

// ── 时间戳 → 日期 ─────────────────────────────────────────
const tsInput = ref('')
const tsUnit = ref<'s' | 'ms'>('s')
const tzMode = ref<'local' | 'utc'>('local')
const error = ref('')
const copiedKey = ref('')

const parsedDate = computed<Date | null>(() => {
  const raw = tsInput.value.trim()
  if (!raw) return null
  const n = Number(raw)
  if (!Number.isFinite(n)) return null
  const ms = tsUnit.value === 's' ? n * 1000 : n
  const d = new Date(ms)
  return isNaN(d.getTime()) ? null : d
})

const tsResults = computed(() => {
  const d = parsedDate.value
  if (!d) return null
  const local = tzMode.value === 'local'
  return {
    iso: d.toISOString(),
    local: d.toLocaleString(),
    utc: d.toUTCString(),
    date: local ? d.toLocaleDateString() : d.toISOString().slice(0, 10),
    time: local ? d.toLocaleTimeString() : d.toISOString().slice(11, 19) + ' UTC',
    relative: relativeFrom(d.getTime()),
    weekday: d.toLocaleDateString(undefined, { weekday: 'long' }),
  }
})

// ── 日期 → 时间戳 ─────────────────────────────────────────
const dateInput = ref('')
const dateResult = computed(() => {
  const raw = dateInput.value.trim()
  if (!raw) return null
  const d = new Date(raw)
  if (isNaN(d.getTime())) return null
  return {
    seconds: Math.floor(d.getTime() / 1000),
    millis: d.getTime(),
    iso: d.toISOString(),
  }
})

// ── 相对时间 ──────────────────────────────────────────────
function relativeFrom(ms: number): string {
  const diff = ms - nowMs.value
  const abs = Math.abs(diff)
  const units: [number, string][] = [
    [1000, '秒'], [60_000, '分钟'], [3_600_000, '小时'],
    [86_400_000, '天'], [2_592_000_000, '个月'], [31_536_000_000, '年'],
  ]
  let unitMs = 1000, unitName = '秒'
  for (const [u, name] of units) {
    if (abs >= u) { unitMs = u; unitName = name }
  }
  const value = Math.round(abs / unitMs)
  if (value === 0) return '刚刚'
  return diff < 0 ? `${value} ${unitName}前` : `${value} ${unitName}后`
}

const localTzName = computed(() => {
  try { return Intl.DateTimeFormat().resolvedOptions().timeZone }
  catch { return '本地' }
})
const tzOffset = computed(() => {
  const off = -new Date().getTimezoneOffset()
  const sign = off >= 0 ? '+' : '-'
  const h = String(Math.floor(Math.abs(off) / 60)).padStart(2, '0')
  const m = String(Math.abs(off) % 60).padStart(2, '0')
  return `UTC${sign}${h}:${m}`
})

function fillNow() { tsInput.value = String(tsUnit.value === 's' ? nowSeconds.value : nowMs.value) }
function fillNowDate() { dateInput.value = new Date().toISOString() }

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
</script>

<template>
  <div class="tool">
    <h2>时间戳 / 时区转换</h2>
    <p class="lead">Unix 时间戳与日期互转、相对时间与时区。本机时区：{{ localTzName }}（{{ tzOffset }}）。</p>

    <section class="card now-card">
      <div class="now-row">
        <span class="label">当前时间</span>
        <span class="now-iso">{{ new Date(nowMs).toLocaleString() }}</span>
      </div>
      <div class="now-grid">
        <button class="chip" @click="copyValue('nowS', String(nowSeconds))">
          秒：<b>{{ nowSeconds }}</b> {{ copiedKey === 'nowS' ? '✓' : '' }}
        </button>
        <button class="chip" @click="copyValue('nowMs', String(nowMs))">
          毫秒：<b>{{ nowMs }}</b> {{ copiedKey === 'nowMs' ? '✓' : '' }}
        </button>
      </div>
    </section>

    <div class="io-grid">
      <!-- 时间戳 → 日期 -->
      <section class="card">
        <h3 class="group-title">时间戳 → 日期</h3>
        <div class="row">
          <input v-model="tsInput" class="input input-grow" placeholder="1733654400" />
          <button class="btn btn-outline" @click="fillNow">当前</button>
        </div>
        <div class="row row-spaced">
          <label class="radio"><input type="radio" value="s" v-model="tsUnit" /> 秒</label>
          <label class="radio"><input type="radio" value="ms" v-model="tsUnit" /> 毫秒</label>
          <span class="divider" />
          <label class="radio"><input type="radio" value="local" v-model="tzMode" /> 本地</label>
          <label class="radio"><input type="radio" value="utc" v-model="tzMode" /> UTC</label>
        </div>

        <div v-if="tsResults" class="result-list">
          <div class="result-row" v-for="(item, i) in [
            { k: 'iso', label: 'ISO 8601', v: tsResults.iso },
            { k: 'local', label: '本地', v: tsResults.local },
            { k: 'utc', label: 'UTC', v: tsResults.utc },
            { k: 'weekday', label: '星期', v: tsResults.weekday },
            { k: 'rel', label: '相对', v: tsResults.relative },
          ]" :key="i">
            <span class="rk">{{ item.label }}</span>
            <span class="rv mono">{{ item.v }}</span>
            <button class="link-btn" @click="copyValue('ts-' + item.k, item.v)">
              {{ copiedKey === 'ts-' + item.k ? '✓' : '复制' }}
            </button>
          </div>
        </div>
        <p v-else-if="tsInput" class="hint bad">无法解析为时间戳</p>
      </section>

      <!-- 日期 → 时间戳 -->
      <section class="card">
        <h3 class="group-title">日期 → 时间戳</h3>
        <div class="row">
          <input v-model="dateInput" class="input input-grow" placeholder="2026-01-01 12:00:00 或 ISO" />
          <button class="btn btn-outline" @click="fillNowDate">当前</button>
        </div>

        <div v-if="dateResult" class="result-list result-list-spaced">
          <div class="result-row">
            <span class="rk">秒</span>
            <span class="rv mono">{{ dateResult.seconds }}</span>
            <button class="link-btn" @click="copyValue('d-s', String(dateResult.seconds))">
              {{ copiedKey === 'd-s' ? '✓' : '复制' }}
            </button>
          </div>
          <div class="result-row">
            <span class="rk">毫秒</span>
            <span class="rv mono">{{ dateResult.millis }}</span>
            <button class="link-btn" @click="copyValue('d-ms', String(dateResult.millis))">
              {{ copiedKey === 'd-ms' ? '✓' : '复制' }}
            </button>
          </div>
          <div class="result-row">
            <span class="rk">ISO</span>
            <span class="rv mono">{{ dateResult.iso }}</span>
            <button class="link-btn" @click="copyValue('d-iso', dateResult.iso)">
              {{ copiedKey === 'd-iso' ? '✓' : '复制' }}
            </button>
          </div>
        </div>
        <p v-else-if="dateInput" class="hint bad">无法解析为日期</p>
      </section>
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
  padding: 16px;
}
.now-card { margin-bottom: 12px; }
.now-row { display: flex; align-items: baseline; gap: 12px; margin-bottom: 10px; }
.now-iso { font: 14px ui-monospace, SFMono-Regular, Consolas, monospace; }
.now-grid { display: flex; gap: 8px; flex-wrap: wrap; }
.chip {
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 6px 12px;
  cursor: pointer;
  font: 13px ui-monospace, SFMono-Regular, Consolas, monospace;
  color: var(--fg);
}
.chip:hover { border-color: var(--primary); }
.chip b { color: var(--primary); }

.io-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
.group-title {
  font-size: 12px; font-weight: 600; color: var(--fg-muted);
  margin: 0 0 12px; text-transform: uppercase; letter-spacing: 0.5px;
}

.row { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
.row-spaced { margin-top: 10px; }
.input-grow { flex: 1 1 0; min-width: 0; }
.radio { font-size: 13px; color: var(--fg-muted); display: inline-flex; align-items: center; gap: 4px; cursor: pointer; }
.divider { width: 1px; height: 16px; background: var(--border); margin: 0 4px; }

.result-list { margin-top: 14px; display: flex; flex-direction: column; gap: 8px; }
.result-list-spaced { margin-top: 16px; }
.result-row { display: flex; align-items: center; gap: 10px; }
.rk { flex: 0 0 60px; font-size: 12px; color: var(--fg-muted); }
.rv { flex: 1 1 0; min-width: 0; overflow-wrap: anywhere; }
.mono { font: 13px ui-monospace, SFMono-Regular, Consolas, monospace; }

.label { font-size: 12px; font-weight: 600; color: var(--fg-muted); }
.hint { font-size: 12px; margin: 10px 0 0; }
.hint.bad { color: var(--danger); }

.input {
  padding: 6px 10px;
  border: 1px solid var(--border);
  border-radius: 4px;
  background: var(--bg);
  color: var(--fg);
  font: 13px ui-monospace, SFMono-Regular, Consolas, monospace;
}
.input:focus { outline: none; border-color: var(--primary); }

.btn {
  background: var(--primary); color: #fff; border: none;
  padding: 6px 14px; border-radius: 4px; cursor: pointer;
  font-size: 13px; font-weight: 500;
}
.btn:hover:not(:disabled) { background: var(--primary-hover); }
.btn-outline { background: transparent; color: var(--fg); border: 1px solid var(--border); }
.btn-outline:hover:not(:disabled) { border-color: var(--primary); color: var(--primary); }

.link-btn {
  background: none; border: none; padding: 4px 8px;
  font-size: 12px; color: var(--fg-muted); cursor: pointer; flex: 0 0 auto;
}
.link-btn:hover:not(:disabled) { color: var(--primary); }

.error { color: var(--danger); margin: 12px 0 0; font-size: 13px; }

@media (max-width: 700px) {
  .io-grid { grid-template-columns: 1fr; }
}
</style>
