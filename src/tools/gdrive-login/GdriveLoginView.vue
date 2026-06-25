<script setup lang="ts">
import { ref, computed } from 'vue'
import { oauthLogin, type GdriveCreds } from './tauri'

type Phase = 'idle' | 'logging_in' | 'done'

const phase = ref<Phase>('idle')
const error = ref('')
const clientId = ref('')
const clientSecret = ref('')
const creds = ref<GdriveCreds | null>(null)
const copiedKey = ref('')

/** sbot wiki.gdrive（OAuth 模式）所需的配置片段，方便整段粘贴。 */
const configSnippet = computed(() => {
  if (!creds.value) return ''
  return JSON.stringify(
    {
      authMethod: 'oauth',
      clientId: creds.value.clientId,
      clientSecret: creds.value.clientSecret,
      refreshToken: creds.value.refreshToken,
    },
    null,
    2,
  )
})

async function start() {
  if (!clientId.value.trim() || !clientSecret.value.trim()) {
    error.value = '请先填写 Client ID 与 Client Secret'
    return
  }
  error.value = ''
  copiedKey.value = ''
  phase.value = 'logging_in'
  try {
    creds.value = await oauthLogin(clientId.value.trim(), clientSecret.value.trim())
    phase.value = 'done'
  } catch (e: any) {
    phase.value = 'idle'
    error.value = String(e?.message || e)
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

function reset() {
  phase.value = 'idle'
  creds.value = null
  error.value = ''
  copiedKey.value = ''
}
</script>

<template>
  <div class="gdrive">
    <h2>Google Drive 登录</h2>
    <p class="lead">
      用浏览器走一遍 Google OAuth 授权，自动拿到 sbot <code>wiki.gdrive</code>（OAuth 模式）所需的
      refresh token，免去手动 OAuth Playground。
    </p>

    <section v-if="phase === 'idle'" class="card">
      <div class="field-group">
        <h3 class="group-title">前置准备：拿到 Client ID / Secret</h3>
        <ol class="guide">
          <li>
            打开
            <a class="link" href="https://console.cloud.google.com/" target="_blank" rel="noreferrer">Google Cloud Console</a>，
            在顶部项目选择器里新建或选中一个项目。
          </li>
          <li>
            进入「API 和服务 → 库」，搜索 <strong>Google Drive API</strong> 并点击<strong>启用</strong>。
          </li>
          <li>
            打开左侧
            <a class="link" href="https://console.cloud.google.com/auth/overview" target="_blank" rel="noreferrer">Google Auth Platform</a>
            （即旧版「OAuth 权限请求页面」，首次使用会引导你配置）：在
            <strong>品牌塑造（Branding）</strong>里填好应用名称、支持邮箱等必填项，User Type 选
            <strong>外部（External）</strong>。
          </li>
          <li>
            切到
            <strong>目标对象（Audience）</strong>，把你自己的 Google 账号加到
            <strong>测试用户（Test users）</strong>
            （应用处于「测试」状态时只有测试用户能授权）。
          </li>
          <li>
            切到 <strong>客户端（Clients）→ 创建客户端</strong>，应用类型必须选
            <strong>桌面应用（Desktop app）</strong>
            ——该类型会自动放行本地回环重定向，无需登记端口。创建后弹窗里就有
            <code>Client ID</code> 与 <code>Client Secret</code>。
          </li>
        </ol>
        <p class="tip">
          💡 顺序别反：必须先做完第 3、4 步（同意页），第 5 步才建得了客户端。若在
          Google Drive API 页直接点「创建凭证」而同意页未配好，会被拦下来先要求配置同意页。
        </p>
      </div>

      <div class="field-group">
        <h3 class="group-title">填入凭据并登录</h3>
        <ol class="guide" start="6">
          <li>把上一步的 Client ID 与 Client Secret 分别粘贴到下面两个输入框。</li>
          <li>
            点击<strong>浏览器登录</strong>，系统默认浏览器会打开 Google 同意页；选择刚才加入的测试账号并同意。
          </li>
          <li>
            授权成功后浏览器会自动跳回本地并提示「登录成功」，回到 sbox 即可看到
            <code>refreshToken</code>，按下方提示复制配置即可。
          </li>
        </ol>
        <div class="input-row">
          <label class="input-label">Client ID</label>
          <input v-model="clientId" class="input" placeholder="xxxxxx.apps.googleusercontent.com" />
        </div>
        <div class="input-row">
          <label class="input-label">Client Secret</label>
          <input v-model="clientSecret" class="input" type="password" placeholder="GOCSPX-..." />
        </div>
      </div>
      <div class="actions">
        <button class="btn" @click="start">浏览器登录</button>
      </div>
      <p class="hint">
        本工具仅请求 <code>drive.readonly</code> 只读权限，全程在你本机完成、不经过第三方服务器。
        若提示「未返回 refresh token」，多半是该账号此前已授权过本应用，到
        <a class="link" href="https://myaccount.google.com/permissions" target="_blank" rel="noreferrer">账号权限页</a>
        撤销后重试。
      </p>
      <p v-if="error" class="error">{{ error }}</p>
    </section>

    <section v-else-if="phase === 'logging_in'" class="card">
      <div class="status">已在系统浏览器打开 Google 授权页，等待你完成授权…</div>
      <p class="hint">在浏览器里选择账号并同意授权后，这里会自动继续。最长等待 5 分钟。</p>
    </section>

    <section v-else-if="phase === 'done' && creds" class="card">
      <div class="status success">授权成功，refresh token 已就绪</div>

      <div class="field-group">
        <h3 class="group-title">凭据</h3>
        <div class="field">
          <span class="field-label">clientId</span>
          <code class="field-value">{{ creds.clientId }}</code>
          <button class="copy-btn" @click="copyValue('clientId', creds.clientId)">
            {{ copiedKey === 'clientId' ? '已复制 ✓' : '复制' }}
          </button>
        </div>
        <div class="field">
          <span class="field-label">clientSecret</span>
          <code class="field-value">{{ creds.clientSecret }}</code>
          <button class="copy-btn" @click="copyValue('clientSecret', creds.clientSecret)">
            {{ copiedKey === 'clientSecret' ? '已复制 ✓' : '复制' }}
          </button>
        </div>
        <div class="field">
          <span class="field-label">refreshToken</span>
          <code class="field-value">{{ creds.refreshToken }}</code>
          <button class="copy-btn" @click="copyValue('refreshToken', creds.refreshToken)">
            {{ copiedKey === 'refreshToken' ? '已复制 ✓' : '复制' }}
          </button>
        </div>
      </div>

      <div class="field-group">
        <h3 class="group-title">sbot wiki.gdrive 配置（整段粘贴）</h3>
        <pre class="snippet">{{ configSnippet }}</pre>
        <button class="copy-btn snippet-copy" @click="copyValue('snippet', configSnippet)">
          {{ copiedKey === 'snippet' ? '已复制 ✓' : '复制配置' }}
        </button>
      </div>

      <div class="actions">
        <button class="btn btn-outline" @click="reset">返回</button>
      </div>
      <p v-if="error" class="error">{{ error }}</p>
    </section>
  </div>
</template>

<style scoped>
.gdrive { max-width: 720px; margin: 0 auto; }
.lead { color: var(--fg-muted); margin-bottom: 20px; }
.card {
  background: var(--card); border: 1px solid var(--border); border-radius: var(--radius);
  padding: 20px; margin-bottom: 16px;
}
.status { font-size: 14px; margin-bottom: 12px; }
.status.success { color: var(--success); font-weight: 500; margin-bottom: 16px; }
.hint { font-size: 12px; color: var(--fg-muted); margin: 12px 0 0; line-height: 1.6; }
.error { color: var(--danger); margin: 12px 0 0; font-size: 13px; }

.guide {
  margin: 0 0 4px; padding-left: 22px;
  font-size: 13px; line-height: 1.7; color: var(--fg);
}
.guide li { margin-bottom: 6px; }
.guide li:last-child { margin-bottom: 0; }
.link { color: var(--primary); text-decoration: none; }
.link:hover { text-decoration: underline; }
.tip {
  margin: 10px 0 0; padding: 8px 12px;
  background: color-mix(in srgb, var(--primary) 8%, transparent);
  border-left: 3px solid var(--primary); border-radius: 4px;
  font-size: 12px; line-height: 1.6; color: var(--fg);
}

.field-group { margin-bottom: 20px; }
.field-group:last-of-type { margin-bottom: 12px; }
.group-title {
  font-size: 13px; font-weight: 600; color: var(--fg-muted);
  margin: 0 0 10px; text-transform: uppercase; letter-spacing: 0.5px;
}

.input-row { display: flex; align-items: center; gap: 8px; padding: 6px 0; }
.input-label { flex: 0 0 100px; font-size: 12px; color: var(--fg-muted); }
.input {
  flex: 1 1 auto; min-width: 0; padding: 8px 10px;
  border: 1px solid var(--border); border-radius: 4px;
  background: var(--bg); color: inherit;
  font: 13px/1.5 ui-monospace, SFMono-Regular, Consolas, monospace;
}

.field {
  display: flex; align-items: center; gap: 8px;
  padding: 6px 0; border-bottom: 1px solid var(--border);
}
.field:last-child { border-bottom: none; }
.field-label { flex: 0 0 100px; font-size: 12px; color: var(--fg-muted); }
.field-value {
  flex: 1 1 auto; min-width: 0;
  font: 12px/1.5 ui-monospace, SFMono-Regular, Consolas, monospace;
  background: transparent; padding: 4px 6px; border-radius: 4px;
  word-break: break-all; user-select: text;
}
.copy-btn {
  flex: 0 0 auto; padding: 4px 10px; font-size: 12px;
  border: 1px solid var(--border); background: var(--card);
  border-radius: 4px; cursor: pointer;
}
.copy-btn:hover { background: var(--border); }

.snippet {
  margin: 0 0 8px; padding: 10px; background: var(--bg);
  border: 1px solid var(--border); border-radius: 4px;
  font: 12px/1.5 ui-monospace, SFMono-Regular, Consolas, monospace;
  white-space: pre-wrap; word-break: break-all; user-select: text;
}
.snippet-copy { width: 100%; padding: 6px; }

.actions { display: flex; gap: 12px; margin-top: 8px; }
</style>
