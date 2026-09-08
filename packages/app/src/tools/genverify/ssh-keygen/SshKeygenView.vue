<script setup lang="ts">
import { computed, ref } from 'vue'
import { save } from '@tauri-apps/plugin-dialog'
import { generateSshKey, type SshKeyGenerateResult, type SshKeyType } from './tauri'

type Phase = 'idle' | 'generating' | 'done'

const phase = ref<Phase>('idle')
const keyType = ref<SshKeyType>('ed25519')
const comment = ref('')
const passphrase = ref('')
const passphraseConfirm = ref('')
const error = ref('')
const copied = ref('')
const revealPrivateKey = ref(false)
const result = ref<SshKeyGenerateResult | null>(null)

const passphraseMismatch = computed(
  () => passphraseConfirm.value.length > 0 && passphrase.value !== passphraseConfirm.value,
)
const canSubmit = computed(
  () => phase.value === 'idle' && passphrase.value === passphraseConfirm.value,
)
const keyTypeHint = computed(() =>
  keyType.value === 'ed25519'
    ? '现代 SSH 服务的推荐选项，密钥更短、生成更快。'
    : '用于需要 RSA 的旧系统，生成速度较慢、密钥文件更大。',
)
const defaultFileName = computed(() => keyType.value === 'ed25519' ? 'id_ed25519' : 'id_rsa')
const keyTypeLabel = computed(() => keyType.value === 'ed25519' ? 'Ed25519' : 'RSA 4096')

async function submit() {
  error.value = ''
  if (!canSubmit.value) return

  const path = await save({
    title: '选择 SSH 私钥保存位置',
    defaultPath: defaultFileName.value,
  })
  if (!path) return

  phase.value = 'generating'
  try {
    result.value = await generateSshKey({
      path,
      keyType: keyType.value,
      comment: comment.value.trim(),
      passphrase: passphrase.value || undefined,
    })
    passphrase.value = ''
    passphraseConfirm.value = ''
    phase.value = 'done'
  } catch (e: any) {
    phase.value = 'idle'
    error.value = String(e?.message || e)
  }
}

async function copyValue(key: string, value: string) {
  try {
    await navigator.clipboard.writeText(value)
    copied.value = key
    window.setTimeout(() => {
      if (copied.value === key) copied.value = ''
    }, 2000)
  } catch (e: any) {
    error.value = `复制失败: ${String(e?.message || e)}`
  }
}

function reset() {
  phase.value = 'idle'
  result.value = null
  error.value = ''
  copied.value = ''
  revealPrivateKey.value = false
}
</script>

<template>
  <div class="ssh-keygen">
    <h2>SSH 密钥生成</h2>
    <p class="lead">
      生成 SSH 登录用的私钥和公钥。密钥只在本机生成并保存，不会上传到服务器。
    </p>

    <section v-if="phase === 'idle'" class="card">
      <div class="field-row">
        <label class="field-label" for="ssh-key-type">密钥类型</label>
        <div class="field-control">
          <select id="ssh-key-type" v-model="keyType" class="input select-input">
            <option value="ed25519">Ed25519（推荐）</option>
            <option value="rsa4096">RSA 4096（兼容旧系统）</option>
          </select>
          <span class="helper">{{ keyTypeHint }}私钥和公钥均使用 OpenSSH 格式。</span>
        </div>
      </div>

      <div class="field-row">
        <label class="field-label" for="ssh-comment">公钥备注</label>
        <div class="field-control">
          <input
            id="ssh-comment"
            v-model="comment"
            class="input"
            autocomplete="off"
            placeholder="可选，例如 name@example.com"
          />
          <span class="helper">备注会附在公钥末尾，便于识别用途。</span>
        </div>
      </div>

      <div class="field-row">
        <label class="field-label" for="ssh-passphrase">私钥密码</label>
        <div class="field-control">
          <input
            id="ssh-passphrase"
            v-model="passphrase"
            type="password"
            class="input"
            autocomplete="new-password"
            placeholder="可选；留空则不加密"
          />
          <span class="helper">建议个人长期使用的私钥设置密码；自动化部署密钥可按环境要求留空。</span>
        </div>
      </div>

      <div class="field-row">
        <label class="field-label" for="ssh-passphrase-confirm">确认密码</label>
        <div class="field-control">
          <input
            id="ssh-passphrase-confirm"
            v-model="passphraseConfirm"
            type="password"
            class="input"
            autocomplete="new-password"
            :aria-invalid="passphraseMismatch"
            aria-describedby="ssh-passphrase-error"
          />
          <span v-if="passphraseMismatch" id="ssh-passphrase-error" class="inline-error">两次输入的密码不一致</span>
        </div>
      </div>

      <div class="actions">
        <button type="button" class="btn" :disabled="!canSubmit" @click="submit">
          选择路径并生成
        </button>
      </div>
      <p v-if="error" class="error" role="alert">{{ error }}</p>
    </section>

    <section v-else-if="phase === 'generating'" class="card" aria-live="polite">
      <p class="status">正在生成 {{ keyTypeLabel }} 密钥…</p>
    </section>

    <section v-else-if="result" class="card" aria-live="polite">
      <p class="status success">生成成功，私钥和公钥已保存。</p>

      <div class="summary">
        <div class="summary-row compact">
          <span>类型</span>
          <strong>{{ result.keyType }}</strong>
        </div>
        <div class="summary-row">
          <span>私钥</span>
          <code>{{ result.privateKeyPath }}</code>
          <button type="button" class="copy-btn" @click="copyValue('private-path', result.privateKeyPath)">
            {{ copied === 'private-path' ? '已复制' : '复制路径' }}
          </button>
        </div>
        <div class="summary-row">
          <span>公钥</span>
          <code>{{ result.publicKeyPath }}</code>
          <button type="button" class="copy-btn" @click="copyValue('public-path', result.publicKeyPath)">
            {{ copied === 'public-path' ? '已复制' : '复制路径' }}
          </button>
        </div>
        <div class="summary-row">
          <span>指纹</span>
          <code>{{ result.fingerprint }}</code>
          <button type="button" class="copy-btn" @click="copyValue('fingerprint', result.fingerprint)">
            {{ copied === 'fingerprint' ? '已复制' : '复制' }}
          </button>
        </div>
        <div class="summary-row compact">
          <span>密码保护</span>
          <strong>{{ result.encrypted ? '已启用' : '未启用' }}</strong>
        </div>
      </div>

      <div class="key-block">
        <div class="key-heading">
          <h3>公钥</h3>
          <button type="button" class="copy-btn" @click="copyValue('public-key', result.publicKey)">
            {{ copied === 'public-key' ? '已复制' : '复制公钥' }}
          </button>
        </div>
        <textarea class="key-value" :value="result.publicKey" readonly spellcheck="false" />
      </div>

      <div class="key-block">
        <div class="key-heading">
          <h3>私钥</h3>
          <div class="key-actions">
            <button type="button" class="copy-btn" @click="revealPrivateKey = !revealPrivateKey">
              {{ revealPrivateKey ? '隐藏内容' : '显示内容' }}
            </button>
            <button type="button" class="copy-btn danger-copy" @click="copyValue('private-key', result.privateKey)">
              {{ copied === 'private-key' ? '已复制' : '复制私钥' }}
            </button>
          </div>
        </div>
        <textarea
          v-if="revealPrivateKey"
          class="key-value private-value"
          :value="result.privateKey"
          readonly
          spellcheck="false"
        />
        <p v-else class="private-hidden">私钥内容已隐藏。通常只需使用已保存的私钥文件。</p>
      </div>

      <p class="warning">
        私钥等同于登录凭据，请妥善备份，不要发送给他人，也不要提交到 Git 仓库。服务器和代码托管平台通常只需要公钥。
      </p>

      <div class="actions">
        <button type="button" class="btn btn-outline" @click="reset">再生成一对</button>
      </div>
      <p v-if="error" class="error" role="alert">{{ error }}</p>
    </section>
  </div>
</template>

<style scoped>
.ssh-keygen { max-width: 760px; margin: 0 auto; }
.lead { color: var(--fg-muted); margin: 0 0 16px; }
.card {
  padding: 20px; margin-bottom: 16px;
  background: var(--card); border: 1px solid var(--border); border-radius: var(--radius);
}
.field-row { display: flex; align-items: flex-start; gap: 12px; margin-bottom: 16px; }
.field-label { flex: 0 0 112px; padding-top: 8px; font-size: 13px; color: var(--fg-muted); }
.field-control { flex: 1 1 auto; min-width: 0; }
.input {
  width: 100%; min-height: 38px; padding: 7px 10px;
  border: 1px solid var(--border); border-radius: 5px;
  background: var(--bg); color: var(--fg);
  font: 13px ui-monospace, SFMono-Regular, Consolas, monospace;
}
.input:focus-visible, .copy-btn:focus-visible {
  outline: 2px solid var(--primary); outline-offset: 2px;
}
.select-input { cursor: pointer; }
.input[aria-invalid="true"] { border-color: var(--danger); }
.helper { display: block; margin-top: 5px; font-size: 12px; color: var(--fg-muted); }
.inline-error, .error { color: var(--danger); font-size: 13px; }
.inline-error { display: block; margin-top: 5px; }
.error { margin: 12px 0 0; }
.actions { display: flex; gap: 12px; margin-top: 12px; }
.status { margin: 0; }
.status.success { color: var(--success); font-weight: 600; margin-bottom: 18px; }
.summary { margin-bottom: 20px; border-top: 1px solid var(--border); }
.summary-row {
  display: grid; grid-template-columns: 64px minmax(0, 1fr) auto;
  align-items: center; gap: 10px; min-height: 44px; border-bottom: 1px solid var(--border);
}
.summary-row > span { color: var(--fg-muted); font-size: 12px; }
.summary-row code { min-width: 0; overflow-wrap: anywhere; font-size: 12px; }
.summary-row.compact { grid-template-columns: 64px 1fr; }
.summary-row strong { font-size: 13px; font-weight: 500; }
.key-block { margin-top: 18px; }
.key-heading { display: flex; align-items: center; justify-content: space-between; gap: 12px; margin-bottom: 8px; }
.key-heading h3 { margin: 0; font-size: 13px; }
.key-actions { display: flex; gap: 8px; }
.key-value {
  display: block; width: 100%; min-height: 76px; resize: vertical; padding: 10px;
  border: 1px solid var(--border); border-radius: 5px;
  background: var(--bg); color: var(--fg);
  font: 12px/1.5 ui-monospace, SFMono-Regular, Consolas, monospace;
  overflow-wrap: anywhere;
}
.private-value { min-height: 190px; }
.private-hidden {
  min-height: 76px; margin: 0; padding: 16px;
  display: flex; align-items: center;
  border: 1px dashed var(--border); border-radius: 5px;
  color: var(--fg-muted); background: var(--bg); font-size: 13px;
}
.copy-btn {
  flex: 0 0 auto; min-height: 32px; padding: 5px 10px;
  border: 1px solid var(--border); border-radius: 5px;
  background: transparent; color: var(--fg); cursor: pointer;
  transition: border-color 0.15s, color 0.15s, background 0.15s;
}
.copy-btn:hover { border-color: var(--primary); color: var(--primary); }
.danger-copy:hover { border-color: var(--danger); color: var(--danger); }
.warning {
  margin: 18px 0 0; padding: 10px 12px;
  border: 1px solid #d97706; border-radius: var(--radius);
  color: #a15c06; background: color-mix(in srgb, #d97706 8%, var(--card)); font-size: 13px;
}
@media (prefers-color-scheme: dark) {
  .warning { color: #f0a94f; }
}
@media (max-width: 600px) {
  .card { padding: 16px; }
  .field-row { display: block; }
  .field-label { display: block; padding: 0; margin-bottom: 6px; }
  .summary-row { grid-template-columns: 54px minmax(0, 1fr); padding: 8px 0; }
  .summary-row .copy-btn { grid-column: 2; justify-self: start; }
  .key-heading { align-items: flex-start; }
}
</style>
