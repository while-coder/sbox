<script setup lang="ts">
import { computed, ref } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { fileInfo, type FileInfoResult, type KeystoreEntry } from './tauri'

const props = defineProps<{ javaAvailable: boolean | null }>()

const path = ref('')
const alias = ref('')
const storePassword = ref('')
const showPassword = ref(false)
const loading = ref(false)
const error = ref('')
const result = ref<FileInfoResult | null>(null)
const copiedKey = ref('')

/** 选中的文件类型（按扩展名本地判断，与后端 classify 一致） */
const kind = computed<'keystore' | 'cert' | 'apk'>(() => {
  if (/\.(apk|aab)$/i.test(path.value.trim())) return 'apk'
  if (/\.(der|pem|crt|cer|p7b|p7c)$/i.test(path.value.trim())) return 'cert'
  return 'keystore'
})
/** 证书与 APK 走纯 Rust 解析，不依赖 keytool */
const needsJava = computed(() => kind.value === 'keystore')

const canSubmit = computed(
  () => path.value.trim().length > 0 && !loading.value && (!needsJava.value || props.javaAvailable === true),
)

const kindLabel = { keystore: 'keystore', cert: '证书', apk: 'APK / AAB' } as const

/** 展示给用户的等价命令行，方便在别的环境复现 */
const cmdlines = computed<{ label: string; cmd: string }[]>(() => {
  const p = path.value.trim()
  if (!p) return []
  if (kind.value === 'apk') {
    return [{ label: '证书指纹', cmd: `apksigner verify --print-certs --verbose "${p}"` }]
  }
  if (kind.value === 'cert') {
    return [
      { label: '证书指纹', cmd: `keytool -printcert -file "${p}"` },
      { label: '密钥散列', cmd: `openssl x509 -in "${p}" -outform der | openssl sha1 -binary | openssl base64` },
    ]
  }
  const aliasArg = alias.value.trim() ? ` -alias "${alias.value.trim()}"` : ''
  return [
    { label: '证书指纹', cmd: `keytool -list -v -keystore "${p}"${aliasArg}` },
    { label: '密钥散列', cmd: `keytool -exportcert -keystore "${p}"${aliasArg} | openssl sha1 -binary | openssl base64` },
  ]
})

async function chooseFile() {
  error.value = ''
  const selection = await open({
    title: '选择签名文件',
    multiple: false,
    filters: [
      { name: '支持的文件', extensions: ['jks', 'keystore', 'p12', 'pfx', 'der', 'pem', 'crt', 'cer', 'p7b', 'p7c', 'apk', 'aab'] },
      { name: 'Java KeyStore', extensions: ['jks', 'keystore', 'p12', 'pfx'] },
      { name: '证书文件', extensions: ['der', 'pem', 'crt', 'cer', 'p7b', 'p7c'] },
      { name: 'APK / AAB', extensions: ['apk', 'aab'] },
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
    result.value = await fileInfo({
      path: path.value.trim(),
      alias: alias.value.trim() || undefined,
      storePassword: storePassword.value || undefined,
    })
    if (result.value.entries.length === 0) {
      error.value = '未解析到任何证书。请检查文件类型或密码是否正确。'
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

function entryTitle(entry: KeystoreEntry): string {
  return kind.value === 'keystore' ? entry.alias : `${entry.alias}${entry.entryType ? ` · ${entry.entryType}` : ''}`
}
</script>

<template>
  <div class="ksd">
    <p class="lead">
      读取签名文件的证书详情：所有者、有效期、MD5 / SHA-1 / SHA-256 指纹，以及密钥散列（base64(SHA1)，Facebook「Android Key Hashes」、微信开放平台「应用签名」要求填的就是它）。
      支持 keystore、裸证书（.der / .pem / .p7b 等）与 APK / AAB。查看「线上实际签名」请用从应用商店下载的 APK——Play App Signing 场景下自己导出的 AAB 签的是上传密钥，不是线上密钥。
    </p>

    <section class="card">
      <div class="field-row">
        <label class="field-label">文件</label>
        <input v-model="path" class="input" placeholder="选择 keystore / 证书 / APK 文件" readonly @click="chooseFile" />
        <button class="btn btn-outline" @click="chooseFile">浏览…</button>
      </div>
      <p v-if="path" class="hint">识别为：{{ kindLabel[kind] }}<template v-if="kind !== 'keystore'">（无需安装 JDK）</template></p>

      <template v-if="kind === 'keystore'">
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
      </template>

      <div class="actions">
        <button class="btn" :disabled="!canSubmit" @click="submit">{{ loading ? '正在读取…' : '读取详情' }}</button>
      </div>

      <div v-if="cmdlines.length" class="cmdlines">
        <p class="hint cmdline-title">等价命令行：</p>
        <div v-for="c in cmdlines" :key="c.label" class="cmdline">
          <span class="cmdline-label">{{ c.label }}</span>
          <code class="cmdline-text">{{ c.cmd }}</code>
          <button class="copy-btn" @click="copyValue(c.cmd, c.cmd)">
            {{ copiedKey === c.cmd ? '已复制 ✓' : '复制' }}
          </button>
        </div>
      </div>
      <p v-if="error" class="error">{{ error }}</p>
    </section>

    <section v-if="result" class="card">
      <div class="status success">
        解析到 {{ result.entries.length }} 条证书<template v-if="result.storeType">（{{ result.storeType }}）</template>
      </div>

      <div v-for="(entry, i) in result.entries" :key="`${entry.alias}-${i}`" class="entry">
        <h3 class="entry-title">
          {{ entryTitle(entry) }}
          <span v-if="kind === 'keystore' && entry.entryType" class="entry-type">{{ entry.entryType }}</span>
        </h3>

        <div class="meta">
          <div v-if="entry.owner" class="meta-row"><span class="meta-label">所有者</span><span class="meta-value">{{ entry.owner }}</span></div>
          <div v-if="entry.issuer" class="meta-row"><span class="meta-label">发布者</span><span class="meta-value">{{ entry.issuer }}</span></div>
          <div v-if="entry.serialNumber" class="meta-row"><span class="meta-label">序列号</span><span class="meta-value mono">{{ entry.serialNumber }}</span></div>
          <div v-if="entry.creationDate" class="meta-row"><span class="meta-label">创建日期</span><span class="meta-value">{{ entry.creationDate }}</span></div>
          <div v-if="entry.validFrom || entry.validUntil" class="meta-row">
            <span class="meta-label">有效期</span>
            <span class="meta-value">{{ entry.validFrom }} ~ {{ entry.validUntil ?? '' }}</span>
          </div>
          <div v-if="entry.signatureAlgorithm" class="meta-row"><span class="meta-label">签名算法</span><span class="meta-value mono">{{ entry.signatureAlgorithm }}</span></div>
          <div v-if="entry.keyAlgorithm" class="meta-row"><span class="meta-label">密钥</span><span class="meta-value">{{ entry.keyAlgorithm }}</span></div>
        </div>

        <div v-if="entry.sha1Base64" class="fp-row highlight">
          <span class="fp-label">密钥散列</span>
          <code class="fp-value">{{ entry.sha1Base64 }}</code>
          <button class="copy-btn" @click="copyValue(`hash-${i}`, entry.sha1Base64!)">
            {{ copiedKey === `hash-${i}` ? '已复制 ✓' : '复制' }}
          </button>
        </div>
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
        「密钥散列」即 base64(SHA1(证书))，可直接粘贴到 Facebook「Android Key Hashes」、微信「应用签名」等平台后台。
        平台要求指纹「去掉冒号」时，粘贴前删除冒号即可（大小写一般不敏感，按要求保留）。
        密钥散列的管道命令需要 openssl（Git Bash 自带）；本工具直接由 SHA-1 指纹换算，无需 openssl。
      </p>
    </section>
  </div>
</template>

<style scoped>
.ksd { max-width: 720px; margin: 0 auto; }
.lead { color: var(--fg-muted); margin-bottom: 16px; }
.hint { font-size: 12px; color: var(--fg-muted); margin: 8px 0 0; }
.cmdlines { margin-top: 12px; }
.cmdline-title { margin: 0 0 4px; }
.cmdline { display: flex; align-items: center; gap: 8px; margin-top: 4px; flex-wrap: wrap; }
.cmdline-label { flex: 0 0 60px; font-size: 12px; color: var(--fg-muted); }
.cmdline-text {
  flex: 1 1 auto; min-width: 0;
  font: 12px/1.5 ui-monospace, SFMono-Regular, Consolas, monospace;
  color: var(--fg-muted); word-break: break-all; user-select: text;
}
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
.entry-title {
  font-size: 14px; font-weight: 600; margin: 0 0 8px;
  display: flex; align-items: center; gap: 8px;
}
.entry-type {
  font-size: 11px; font-weight: 400; color: var(--fg-muted);
  background: var(--bg); border: 1px solid var(--border);
  border-radius: 4px; padding: 1px 7px;
}
.meta {
  margin-bottom: 8px; padding-bottom: 8px;
  border-bottom: 1px dashed var(--border);
}
.meta-row { display: flex; gap: 10px; font-size: 12px; margin-bottom: 4px; }
.meta-label { flex: 0 0 60px; color: var(--fg-muted); }
.meta-value { color: var(--fg-muted); word-break: break-all; }
.meta-value.mono { font-family: ui-monospace, SFMono-Regular, Consolas, monospace; }

.fp-row { display: flex; align-items: center; gap: 8px; padding: 4px 0; }
.fp-row.highlight code { color: var(--success); }
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
