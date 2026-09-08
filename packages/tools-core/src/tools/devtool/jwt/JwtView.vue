<script setup lang="ts">
import { ref, computed } from 'vue'
import { decodeJwt, type DecodedJwt } from './jwt'

const input = ref('')
const error = ref('')
const copiedKey = ref('')

const decoded = computed<DecodedJwt | null>(() => {
  error.value = ''
  if (!input.value.trim()) return null
  try {
    return decodeJwt(input.value)
  } catch (e: any) {
    error.value = String(e?.message || e)
    return null
  }
})

const headerJson = computed(() => decoded.value ? JSON.stringify(decoded.value.header, null, 2) : '')
const payloadJson = computed(() => decoded.value ? JSON.stringify(decoded.value.payload, null, 2) : '')

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
    <h2>JWT 解码</h2>
    <p class="lead">解析 JWT 的 header 与 payload，并把时间类声明转成可读时间。仅本机解析，不校验签名、不上传。</p>

    <section class="card">
      <div class="label-row">
        <span class="label">JWT</span>
        <button class="link-btn" @click="input = ''">清空</button>
      </div>
      <textarea v-model="input" class="textarea" rows="5"
        placeholder="粘贴 eyJ... 形式的 token" />
    </section>

    <div v-if="decoded" class="result">
      <div class="io-grid">
        <section class="card">
          <div class="label-row">
            <span class="label">Header</span>
            <button class="link-btn" @click="copyValue('header', headerJson)">
              {{ copiedKey === 'header' ? '已复制 ✓' : '复制' }}
            </button>
          </div>
          <pre class="code">{{ headerJson }}</pre>
        </section>
        <section class="card">
          <div class="label-row">
            <span class="label">Payload</span>
            <button class="link-btn" @click="copyValue('payload', payloadJson)">
              {{ copiedKey === 'payload' ? '已复制 ✓' : '复制' }}
            </button>
          </div>
          <pre class="code">{{ payloadJson }}</pre>
        </section>
      </div>

      <section v-if="decoded.claims.length" class="card claims">
        <h3 class="group-title">时间声明</h3>
        <div class="claim-row" v-for="c in decoded.claims" :key="c.key">
          <span class="ck">{{ c.label }}</span>
          <span class="cv mono">{{ c.date }}</span>
          <span v-if="c.expired === true" class="badge bad">已过期</span>
          <span v-else-if="c.expired === false" class="badge ok">未过期</span>
        </div>
      </section>

      <section class="card">
        <div class="label-row">
          <span class="label">Signature（原始）</span>
        </div>
        <pre class="code sig">{{ decoded.signature || '（无签名段）' }}</pre>
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
  margin-bottom: 12px;
}
.group-title {
  font-size: 12px; font-weight: 600; color: var(--fg-muted);
  margin: 0 0 12px; text-transform: uppercase; letter-spacing: 0.5px;
}
.label-row { display: flex; align-items: center; justify-content: space-between; margin-bottom: 6px; }
.label { font-size: 12px; font-weight: 600; color: var(--fg-muted); }

.io-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
.io-grid .card { margin-bottom: 0; }

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
  word-break: break-all;
}
.textarea:focus { outline: none; border-color: var(--primary); }

.code {
  margin: 0;
  padding: 10px;
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: 4px;
  font: 13px/1.5 ui-monospace, SFMono-Regular, Consolas, monospace;
  white-space: pre-wrap;
  word-break: break-all;
  max-height: 320px;
  overflow: auto;
}
.code.sig { color: var(--fg-muted); }

.claim-row { display: flex; align-items: center; gap: 12px; padding: 4px 0; }
.ck { flex: 0 0 180px; font-size: 13px; color: var(--fg-muted); }
.cv { flex: 1 1 0; }
.mono { font: 13px ui-monospace, SFMono-Regular, Consolas, monospace; }
.badge { font-size: 11px; padding: 2px 8px; border-radius: 10px; font-weight: 600; }
.badge.ok { background: rgba(45,164,78,0.15); color: var(--success); }
.badge.bad { background: rgba(207,34,46,0.15); color: var(--danger); }

.link-btn {
  background: none; border: none; padding: 4px 8px;
  font-size: 12px; color: var(--fg-muted); cursor: pointer;
}
.link-btn:hover:not(:disabled) { color: var(--primary); }

.error { color: var(--danger); margin: 4px 0 0; font-size: 13px; }

@media (max-width: 700px) {
  .io-grid { grid-template-columns: 1fr; }
  .ck { flex-basis: 120px; }
}
</style>
