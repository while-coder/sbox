<script setup lang="ts">
import { ref, computed } from 'vue'
import { GDRIVE_SCOPES, oauthLogin, type GdriveCreds } from './tauri'
import { saveTextFile } from '../../save'

type Phase = 'idle' | 'logging_in' | 'done'

const phase = ref<Phase>('idle')
const error = ref('')
const clientId = ref('')
const clientSecret = ref('')
const selectedScopes = ref<string[]>([GDRIVE_SCOPES.drive])
const creds = ref<GdriveCreds | null>(null)
const copiedKey = ref('')
const savedHint = ref('')

const scopeOptions = [
  {
    value: GDRIVE_SCOPES.drive,
    label: 'Google Drive',
    hint: '查看、下载、上传和管理 Google Drive 中的所有文件',
  },
  {
    value: GDRIVE_SCOPES.androidpublisher,
    label: 'Google Play 开发者',
    hint: '访问和管理 Google Play 开发者账号中的应用数据',
  },
]

/** 授权文件内容：Google auth 库的 authorized_user 标准格式。 */
const authFileJson = computed(() => {
  if (!creds.value) return ''
  return JSON.stringify(
    {
      type: 'authorized_user',
      client_id: creds.value.clientId,
      client_secret: creds.value.clientSecret,
      refresh_token: creds.value.refreshToken,
    },
    null,
    2,
  )
})

async function start() {
  error.value = ''
  copiedKey.value = ''
  savedHint.value = ''
  phase.value = 'logging_in'
  try {
    // 留空时由 Rust 侧回退到内置客户端
    creds.value = await oauthLogin(clientId.value.trim(), clientSecret.value.trim(), selectedScopes.value)
    phase.value = 'done'
  } catch (e: any) {
    phase.value = 'idle'
    error.value = String(e?.message || e)
  }
}

/** 把授权内容保存为可直接使用的授权文件。 */
async function saveAuthFile() {
  try {
    const ok = await saveTextFile(authFileJson.value, 'gdrive_token.json')
    if (ok) savedHint.value = '已保存，把该文件路径作为 --auth 参数传入即可'
  } catch (e: any) {
    error.value = `保存失败: ${String(e?.message || e)}`
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
  savedHint.value = ''
}
</script>

<template>
  <div class="gdrive">
    <h2>Google Drive 登录</h2>
    <p class="lead">
      用浏览器走一遍 Google OAuth 授权（系统内置公共客户端，直接登录即可），按需勾选
      Drive / Google Play 权限，拿到 refresh token 后保存为 Google auth 格式的授权文件。
    </p>

    <section v-if="phase === 'idle'" class="card">
      <div class="field-group">
        <h3 class="group-title">自定义客户端（可选）：拿到自己的 Client ID / Secret</h3>
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
        <p class="tip" style="margin: 0 0 10px">
          💡 程序已内置公共客户端，两个输入框<strong>留空即可直接登录</strong>。若你想用自己的
          API 配额，按上方步骤创建后填入自己的 Client ID / Secret 覆盖。
        </p>
        <div class="input-row">
          <label class="input-label">Client ID</label>
          <input v-model="clientId" class="input" placeholder="留空使用内置客户端" />
        </div>
        <div class="input-row">
          <label class="input-label">Client Secret</label>
          <input v-model="clientSecret" class="input" type="password" placeholder="留空使用内置客户端" />
        </div>
        <div class="input-row" style="align-items: flex-start">
          <label class="input-label" style="padding-top: 6px">授权范围</label>
          <div class="scope-list">
            <label v-for="option in scopeOptions" :key="option.value" class="scope-item">
              <input v-model="selectedScopes" type="checkbox" :value="option.value" />
              <span class="scope-text">
                <strong>{{ option.label }}</strong>
                <small>{{ option.hint }}</small>
              </span>
            </label>
          </div>
        </div>
      </div>
      <div class="actions">
        <button class="btn" @click="start">浏览器登录</button>
      </div>
      <p class="hint">
        全程在你本机完成、不经过第三方服务器。若提示「未返回 refresh token」，多半是该账号此前已授权过本应用，到
        <a class="link" href="https://myaccount.google.com/permissions" target="_blank" rel="noreferrer">账号权限页</a>
        撤销后重试。调整授权范围后需重新登录才会生效。
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
        <h3 class="group-title">授权文件（Google auth 格式）</h3>
        <p class="hint" style="margin: 0 0 8px">
          保存为 json 文件后，把文件路径作为 <code>--auth</code> 参数传入即可。
          文件内容含 refresh token，注意保管。
        </p>
        <button class="copy-btn snippet-copy" @click="saveAuthFile">保存授权文件…</button>
        <p v-if="savedHint" class="hint" style="color: var(--success); margin-top: 8px">{{ savedHint }}</p>
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

.scope-list { flex: 1 1 auto; min-width: 0; display: flex; flex-direction: column; gap: 6px; }
.scope-item { display: flex; align-items: flex-start; gap: 8px; cursor: pointer; }
.scope-item input { margin-top: 2px; }
.scope-text { display: flex; flex-direction: column; gap: 2px; }
.scope-text strong { font-size: 13px; font-weight: 500; }
.scope-text small { font-size: 12px; color: var(--fg-muted); line-height: 1.5; }

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

.snippet-copy { width: 100%; padding: 6px; }

.actions { display: flex; gap: 12px; margin-top: 8px; }
</style>
