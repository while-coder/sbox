<script setup lang="ts">
import { computed, ref } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { listKeystore, type KeystoreEntry } from './tauri'
import { useKeytoolCheck } from '../../../shell/use-keytool'
import KeytoolHint from '../../../shell/KeytoolHint.vue'

const path = ref('')
const alias = ref('')
const storePassword = ref('')
const showPassword = ref(false)
const loading = ref(false)
const error = ref('')
const result = ref<{ storeType: string | null; entries: KeystoreEntry[] } | null>(null)
const { javaAvailable, javaPath } = useKeytoolCheck()
const copiedKey = ref('')

const canSubmit = computed(
  () => path.value.trim().length > 0 && javaAvailable.value === true && !loading.value,
)

/** 展示给用户的等价命令行，方便在别的环境复现 */
const cmdline = computed(() => {
  if (!path.value.trim()) return ''
  let cmd = `keytool -list -v -keystore "${path.value.trim()}"`
  if (alias.value.trim()) cmd += ` -alias "${alias.value.trim()}"`
  return cmd
})

async function chooseFile() {
  error.value = ''
  const selection = await open({
    title: '选择 keystore 文件',
    multiple: false,
    filters: [
      { name: 'Java KeyStore', extensions: ['jks', 'keystore', 'p12', 'pfx'] },
      { name: '所有文件', extensions: ['*'] },
    ],
  })
  if (typeof selection === 'string') {
    path.value = selection
    result.value = null
  }
}

async function submit() {
  if (!canSubmit.value) return
  loading.value = true
  error.value = ''
  result.value = null
  try {
    result.value = await listKeystore({
      path: path.value.trim(),
      alias: alias.value.trim() || undefined,
      storePassword: storePassword.value || undefined,
    })
    if (result.value.entries.length === 0) {
      error.value = '未解析到任何别名。请检查密码是否正确、文件是否为有效的 keystore。'
    }
  } catch (e: any) {
    error.value = String(e?.message || e)
  } finally {
    loading.value = false
  }
}

async function copyValue(key: string, value: string) {
  try {
    await navigator.clipboard.writeText(value)
    copiedKey.value = key
    setTimeout(() => {
      if (copiedKey.value === key) copiedKey.value = ''
    }, 2000)
  } catch (e: any) {
    error.value = `复制失败: ${String(e?.message || e)}`
  }
}
</script>

<template>
  <div class="ksi">
    <h2>Keystore 指纹</h2>
    <p class="lead">
      读取 keystore 文件中的证书指纹（SHA-1 / SHA-256 / MD5）。Android 应用在微信开放平台、高德、Google Play 等平台注册时需要填 SHA-1。
      文件与密码仅在本机传给 <code>keytool</code>，不会上传。
    </p>

    <KeytoolHint :available="javaAvailable" :path="javaPath" />

    <section class="card">
      <div class="field-row">
        <label class="field-label">keystore 文件</label>
        <input v-model="path" class="input" placeholder="选择 .jks / .keystore / .p12 文件" readonly @click="chooseFile" />
        <button class="btn btn-outline" @click="chooseFile">浏览…</button>
      </div>

      <div class="field-row">
        <label class="field-label">Alias（可选）</label>
        <input v-model="alias" class="input" placeholder="留空列出所有别名" />
      </div>

      <div class="field-row">
        <label class="field-label">keystore 密码</label>
        <input
          v-model="storePassword"
          :type="showPassword ? 'text' : 'password'"
          class="input"
          placeholder="不填则按无密码尝试"
          @keydown.enter="submit"
        />
        <button class="btn btn-outline" @click="showPassword = !showPassword">{{ showPassword ? '隐藏' : '显示' }}</button>
      </div>

      <div class="actions">
        <button class="btn" :disabled="!canSubmit" @click="submit">{{ loading ? '正在读取…' : '读取指纹' }}</button>
      </div>

      <p v-if="cmdline" class="hint cmdline">
        等价命令行：<code>{{ cmdline }}</code>
        <button class="copy-btn" @click="copyValue('cmd', cmdline)">
          {{ copiedKey === 'cmd' ? '已复制 ✓' : '复制' }}
        </button>
      </p>
      <p v-if="error" class="error">{{ error }}</p>
    </section>

    <section v-if="result" class="card">
      <div class="status success">
        解析到 {{ result.entries.length }} 个别名<template v-if="result.storeType">（{{ result.storeType }}）</template>
      </div>

      <div v-for="(entry, i) in result.entries" :key="entry.alias" class="entry">
        <h3 class="entry-title">{{ entry.alias }}</h3>

        <div v-if="entry.owner" class="meta-row"><span class="meta-label">所有者</span><span class="meta-value">{{ entry.owner }}</span></div>
        <div v-if="entry.validFrom" class="meta-row"><span class="meta-label">有效期</span><span class="meta-value">{{ entry.validFrom }} ~ {{ entry.validUntil ?? '' }}</span></div>

        <div v-if="entry.fingerprintSha1" class="fp-row">
          <span class="fp-label">SHA-1</span>
          <code class="fp-value">{{ entry.fingerprintSha1 }}</code>
          <button class="copy-btn" @click="copyValue(`sha1-${i}`, entry.fingerprintSha1)">
            {{ copiedKey === `sha1-${i}` ? '已复制 ✓' : '复制' }}
          </button>
        </div>
        <div v-if="entry.fingerprintSha256" class="fp-row">
          <span class="fp-label">SHA-256</span>
          <code class="fp-value">{{ entry.fingerprintSha256 }}</code>
          <button class="copy-btn" @click="copyValue(`sha256-${i}`, entry.fingerprintSha256)">
            {{ copiedKey === `sha256-${i}` ? '已复制 ✓' : '复制' }}
          </button>
        </div>
        <div v-if="entry.fingerprintMd5" class="fp-row">
          <span class="fp-label">MD5</span>
          <code class="fp-value">{{ entry.fingerprintMd5 }}</code>
          <button class="copy-btn" @click="copyValue(`md5-${i}`, entry.fingerprintMd5)">
            {{ copiedKey === `md5-${i}` ? '已复制 ✓' : '复制' }}
          </button>
        </div>
      </div>

      <p class="hint">
        平台要求的格式可能不同：多数平台直接粘贴上面带冒号的值即可；要求「去掉冒号」时，粘贴前删除冒号即可（字母大小写一般不敏感，按要求保留）。
      </p>
    </section>
  </div>
</template>

<style scoped>
.ksi { max-width: 720px; margin: 0 auto; }
.lead { color: var(--fg-muted); margin-bottom: 16px; }
.warn {
  background: var(--card); border: 1px solid var(--warning, #d97706);
  border-radius: var(--radius); padding: 10px 12px; font-size: 13px;
  color: var(--warning, #d97706); margin: 12px 0;
}
.hint { font-size: 12px; color: var(--fg-muted); margin: 8px 0 0; }
.hint.cmdline { margin-top: 12px; display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
.error { color: var(--danger); margin: 12px 0 0; font-size: 13px; }

.card {
  background: var(--card); border: 1px solid var(--border); border-radius: var(--radius);
  padding: 20px; margin-bottom: 16px;
}
.status { font-size: 14px; margin-bottom: 16px; }
.status.success { color: var(--success); font-weight: 500; }

.field-row { display: flex; align-items: center; gap: 10px; margin-bottom: 12px; }
.field-label { flex: 0 0 110px; font-size: 13px; color: var(--fg-muted); }
.input {
  flex: 1 1 auto; padding: 6px 10px;
  border: 1px solid var(--border); border-radius: 4px;
  background: var(--bg); color: var(--fg);
  font: 13px ui-monospace, SFMono-Regular, Consolas, monospace;
}
.input:focus { outline: none; border-color: var(--primary); }

.btn {
  padding: 7px 16px; font-size: 13px;
  background: var(--primary); color: #fff;
  border: 1px solid var(--primary); border-radius: 4px; cursor: pointer;
}
.btn:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-outline {
  background: transparent; color: var(--fg);
  border: 1px solid var(--border);
}
.btn-outline:hover { background: var(--border); }

.actions { display: flex; gap: 12px; margin-top: 8px; }

.entry {
  border: 1px solid var(--border); border-radius: var(--radius);
  padding: 12px 14px; margin-bottom: 12px;
}
.entry-title { font-size: 14px; font-weight: 600; margin: 0 0 8px; }
.meta-row { display: flex; gap: 10px; font-size: 12px; margin-bottom: 4px; }
.meta-label { flex: 0 0 60px; color: var(--fg-muted); }
.meta-value { color: var(--fg-muted); word-break: break-all; }

.fp-row { display: flex; align-items: center; gap: 8px; padding: 4px 0; }
.fp-label {
  flex: 0 0 64px; font-size: 12px; color: var(--fg-muted);
  font-family: ui-monospace, SFMono-Regular, Consolas, monospace;
}
.fp-value {
  flex: 1 1 auto; min-width: 0;
  font: 12px/1.5 ui-monospace, SFMono-Regular, Consolas, monospace;
  word-break: break-all; user-select: text;
}
.copy-btn {
  flex: 0 0 auto; padding: 4px 10px; font-size: 12px;
  border: 1px solid var(--border); background: var(--card);
  border-radius: 4px; cursor: pointer;
}
.copy-btn:hover { background: var(--border); }
</style>
