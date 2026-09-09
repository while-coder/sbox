<script setup lang="ts">
import { computed, ref } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { getKeyHash, type KeyHashResult } from './tauri'

const props = defineProps<{ javaAvailable: boolean | null }>()

const path = ref('')
const alias = ref('')
const storePassword = ref('')
const showPassword = ref(false)
const loading = ref(false)
const error = ref('')
const result = ref<KeyHashResult | null>(null)
const copiedKey = ref('')

const canSubmit = computed(
  () => path.value.trim().length > 0 && alias.value.trim().length > 0 && props.javaAvailable === true && !loading.value,
)

/** 展示给用户的等价命令行，方便在别的环境复现 */
const cmdline = computed(() => {
  if (!path.value.trim()) return ''
  return `keytool -exportcert -alias "${alias.value.trim() || 'your_alias'}" -keystore "${path.value.trim()}" | openssl sha1 -binary | openssl base64`
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
    result.value = await getKeyHash({
      path: path.value.trim(),
      alias: alias.value.trim(),
      storePassword: storePassword.value || undefined,
    })
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
  <div class="ksh">
    <p class="lead">
      计算发布密钥散列：对指定别名的证书做 SHA-1 再 base64 编码。微信开放平台等平台「应用签名」一栏要求填的就是它（无需手动执行 keytool / openssl 管道命令）。
    </p>

    <section class="card">
      <div class="field-row">
        <label class="field-label">keystore 文件</label>
        <input v-model="path" class="input" placeholder="选择 .jks / .keystore / .p12 文件" readonly @click="chooseFile" />
        <button class="btn btn-outline" @click="chooseFile">浏览…</button>
      </div>

      <div class="field-row">
        <label class="field-label">Alias</label>
        <input v-model="alias" class="input" placeholder="签名密钥的别名（必填）" />
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
        <button class="btn" :disabled="!canSubmit" @click="submit">{{ loading ? '正在计算…' : '计算散列' }}</button>
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
      <div class="status success">别名「{{ result.alias }}」的发布密钥散列</div>

      <div class="fp-row">
        <span class="fp-label">散列</span>
        <code class="fp-value">{{ result.sha1Base64 }}</code>
        <button class="copy-btn" @click="copyValue('hash', result.sha1Base64)">
          {{ copiedKey === 'hash' ? '已复制 ✓' : '复制' }}
        </button>
      </div>
      <div class="fp-row">
        <span class="fp-label">SHA-1</span>
        <code class="fp-value">{{ result.sha1Hex }}</code>
        <button class="copy-btn" @click="copyValue('sha1', result.sha1Hex)">
          {{ copiedKey === 'sha1' ? '已复制 ✓' : '复制' }}
        </button>
      </div>

      <p class="hint">
        「散列」即 base64(SHA1(证书))，可直接粘贴到平台后台；「SHA-1」为同一证书的十六进制指纹，可与「查看指纹」标签页的输出互相核对。
      </p>
    </section>
  </div>
</template>

<style scoped>
.ksh { max-width: 720px; margin: 0 auto; }
.lead { color: var(--fg-muted); margin-bottom: 16px; }
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
