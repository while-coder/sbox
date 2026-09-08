<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import {
  deleteEnvVar,
  listEnvVars,
  readHosts,
  setEnvVar,
  writeHosts,
  type EnvVarEntry,
  type EnvVarScope,
  type EnvVarsListResult,
} from './tauri'

type ToolTab = EnvVarScope | 'hosts'

const CRITICAL_VARS = new Set(['PATH', 'TEMP', 'TMP', 'PATHEXT', 'COMSPEC'])
const VALUE_WARN_LENGTH = 2048
const VALUE_MAX_LENGTH = 32000
const EXPAND_THRESHOLD = 120

interface EditDialogState {
  mode: 'create' | 'edit'
  originalName: string
  name: string
  value: string
  originalTypeName: 'REG_SZ' | 'REG_EXPAND_SZ'
  /** PATH 列表编辑器的条目；null 表示非列表模式 */
  pathEntries: string[] | null
  useListEditor: boolean
}

const tab = ref<ToolTab>('user')
const data = ref<EnvVarsListResult | null>(null)
const loading = ref(false)
const error = ref('')
const status = ref('')
const query = ref('')
const unsupported = ref(false)
const expanded = ref(new Set<string>())
const pendingDelete = ref<string | null>(null)
const dialog = ref<EditDialogState | null>(null)
const dialogError = ref('')
const saving = ref(false)

let statusTimer: number | undefined
let pendingDeleteTimer: number | undefined

// Hosts 文件
const hostsContent = ref('')
const hostsElevated = ref<boolean | null>(null)
const hostsDirty = ref(false)
const hostsSaving = ref(false)
const hostsLoading = ref(false)
const hostsError = ref('')
const hostsStatus = ref('')
let hostsStatusTimer: number | undefined

const canSaveHosts = computed(
  () => hostsElevated.value !== null && hostsDirty.value && !hostsSaving.value && !hostsLoading.value,
)

async function loadHosts() {
  hostsLoading.value = true
  hostsError.value = ''
  try {
    const result = await readHosts()
    hostsContent.value = result.content
    hostsElevated.value = result.elevated
    hostsDirty.value = false
  } catch (e: any) {
    const message = String(e?.message || e)
    hostsError.value = message
    if (message.includes('仅支持 Windows')) unsupported.value = true
  } finally {
    hostsLoading.value = false
  }
}

async function saveHosts() {
  if (!canSaveHosts.value) return
  hostsSaving.value = true
  hostsError.value = ''
  try {
    await writeHosts(hostsContent.value)
    hostsDirty.value = false
    showHostsStatus('已保存 hosts（原文件已备份为 hosts.bak），DNS 缓存已刷新')
  } catch (e: any) {
    hostsError.value = String(e?.message || e)
  } finally {
    hostsSaving.value = false
  }
}

function showHostsStatus(message: string) {
  hostsStatus.value = message
  window.clearTimeout(hostsStatusTimer)
  hostsStatusTimer = window.setTimeout(() => {
    hostsStatus.value = ''
  }, 4000)
}

const filteredVars = computed(() => {
  const vars = data.value?.scope === tab.value ? data.value.vars : []
  const keyword = query.value.trim().toLowerCase()
  return vars
    .filter(
      (entry) =>
        !keyword ||
        entry.name.toLowerCase().includes(keyword) ||
        entry.rawValue.toLowerCase().includes(keyword),
    )
    .sort((a, b) => a.name.localeCompare(b.name, undefined, { sensitivity: 'base' }))
})

const canSave = computed(() => {
  const state = dialog.value
  if (!state || saving.value) return false
  if (state.mode === 'create' && !state.name.trim()) return false
  return effectiveValueLength.value <= VALUE_MAX_LENGTH
})

const dialogIsPath = computed(() => dialog.value?.name.trim().toUpperCase() === 'PATH')

const showListEditor = computed(() => !!dialog.value?.pathEntries && dialog.value.useListEditor)

const effectiveValue = computed(() => {
  const state = dialog.value
  if (!state) return ''
  return state.pathEntries && state.useListEditor ? state.pathEntries.join(';') : state.value
})

const effectiveValueLength = computed(() => effectiveValue.value.length)

const hasEmptyEntry = computed(() => !!dialog.value?.pathEntries?.some((entry) => !entry))

const nameConflict = computed(() => {
  const state = dialog.value
  if (!state || state.mode !== 'create') return null
  const name = state.name.trim().toUpperCase()
  if (!name) return null
  const existing = (data.value?.vars ?? []).find((entry) => entry.name.toUpperCase() === name)
  return existing ? existing.name : null
})

/** 保存时的注册表类型：编辑保持原类型；新建按值里是否含 %VAR% 引用自动判断。 */
const effectiveTypeName = computed<'REG_SZ' | 'REG_EXPAND_SZ'>(() => {
  const state = dialog.value
  if (!state) return 'REG_SZ'
  if (state.mode === 'edit') return state.originalTypeName
  return /%[A-Za-z_][\w() .\-]*%/.test(effectiveValue.value) ? 'REG_EXPAND_SZ' : 'REG_SZ'
})

async function load(target: EnvVarScope, options: { silent?: boolean } = {}) {
  if (!options.silent) {
    data.value = null
    expanded.value = new Set()
  }
  loading.value = true
  error.value = ''
  try {
    data.value = await listEnvVars(target)
  } catch (e: any) {
    const message = String(e?.message || e)
    error.value = message
    if (message.includes('仅支持 Windows')) unsupported.value = true
  } finally {
    loading.value = false
  }
}

async function switchTab(target: ToolTab) {
  if (target === tab.value) return
  tab.value = target
  query.value = ''
  pendingDelete.value = null
  if (target === 'hosts') await loadHosts()
  else await load(target)
}

function typeLabel(typeName: string) {
  if (typeName === 'REG_EXPAND_SZ') return '展开型'
  if (typeName === 'REG_SZ') return '文本'
  if (typeName === 'REG_DWORD') return '数字'
  return typeName
}

function isCritical(name: string) {
  return CRITICAL_VARS.has(name.toUpperCase())
}

function canExpand(entry: EnvVarEntry) {
  return entry.rawValue.length > EXPAND_THRESHOLD || entry.rawValue.includes('\n')
}

/** PATH 这类分号分隔的列表变量，按条展示比一整段原文可读。 */
function isListVar(entry: EnvVarEntry) {
  return entry.name.trim().toUpperCase() === 'PATH' && entry.rawValue.includes(';')
}

function listEntries(entry: EnvVarEntry) {
  return entry.rawValue.split(';')
}

/** 展开后值有变化才展示；长文本收起时不展示，避免两段大段重复文本。 */
function showExpandedHelper(entry: EnvVarEntry) {
  if (!entry.expandedValue || entry.expandedValue === entry.rawValue) return false
  return !canExpand(entry) || expanded.value.has(entry.name)
}

function toggleExpand(entry: EnvVarEntry) {
  const next = new Set(expanded.value)
  if (!next.delete(entry.name)) next.add(entry.name)
  expanded.value = next
}

function showStatus(message: string) {
  status.value = message
  window.clearTimeout(statusTimer)
  statusTimer = window.setTimeout(() => {
    status.value = ''
  }, 4000)
}

function askDelete(name: string) {
  pendingDelete.value = name
  window.clearTimeout(pendingDeleteTimer)
  pendingDeleteTimer = window.setTimeout(() => {
    pendingDelete.value = null
  }, 3000)
}

async function remove(name: string) {
  error.value = ''
  try {
    await deleteEnvVar(tab.value as EnvVarScope, name)
    showStatus(`已删除 ${name}，新启动的程序会立即生效`)
  } catch (e: any) {
    error.value = String(e?.message || e)
  } finally {
    pendingDelete.value = null
    window.clearTimeout(pendingDeleteTimer)
    await load(tab.value as EnvVarScope, { silent: true })
  }
}

function openCreate() {
  dialogError.value = ''
  dialog.value = {
    mode: 'create',
    originalName: '',
    name: '',
    value: '',
    originalTypeName: 'REG_SZ',
    pathEntries: null,
    useListEditor: false,
  }
}

function openEdit(entry: EnvVarEntry) {
  if (entry.typeName !== 'REG_SZ' && entry.typeName !== 'REG_EXPAND_SZ') return
  dialogError.value = ''
  const isPath = entry.name.trim().toUpperCase() === 'PATH'
  dialog.value = {
    mode: 'edit',
    originalName: entry.name,
    name: entry.name,
    value: entry.rawValue,
    originalTypeName: entry.typeName,
    pathEntries: isPath ? entry.rawValue.split(';') : null,
    useListEditor: isPath,
  }
}

function closeDialog() {
  dialog.value = null
  dialogError.value = ''
}

async function saveDialog() {
  const state = dialog.value
  if (!state || !canSave.value) return
  saving.value = true
  dialogError.value = ''
  try {
    const name = state.name.trim()
    await setEnvVar(tab.value as EnvVarScope, name, effectiveValue.value, effectiveTypeName.value)
    dialog.value = null
    showStatus(`已保存 ${name}，新启动的程序会立即生效`)
    await load(tab.value as EnvVarScope, { silent: true })
  } catch (e: any) {
    dialogError.value = String(e?.message || e)
  } finally {
    saving.value = false
  }
}

async function copy(value: string) {
  try {
    await navigator.clipboard.writeText(value)
    showStatus('已复制到剪贴板')
  } catch (e: any) {
    error.value = `复制失败: ${String(e?.message || e)}`
  }
}

function switchToListEditor() {
  const state = dialog.value
  if (!state) return
  state.pathEntries = state.value.split(';')
  state.useListEditor = true
}

function switchToTextEditor() {
  const state = dialog.value
  if (!state?.pathEntries) return
  state.value = state.pathEntries.join(';')
  state.useListEditor = false
}

function addPathEntry() {
  dialog.value?.pathEntries?.push('')
}

function removePathEntry(index: number) {
  dialog.value?.pathEntries?.splice(index, 1)
}

function movePathEntry(index: number, offset: number) {
  const entries = dialog.value?.pathEntries
  if (!entries) return
  const target = index + offset
  if (target < 0 || target >= entries.length) return
  const [entry] = entries.splice(index, 1)
  entries.splice(target, 0, entry)
}

function onKeyDown(event: KeyboardEvent) {
  if (event.key === 'Escape' && dialog.value) closeDialog()
}

onMounted(() => {
  load('user')
  window.addEventListener('keydown', onKeyDown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', onKeyDown)
  window.clearTimeout(statusTimer)
  window.clearTimeout(pendingDeleteTimer)
  window.clearTimeout(hostsStatusTimer)
})
</script>

<template>
  <div class="env-vars">
    <h2>环境变量</h2>
    <p class="lead">查看和编辑 Windows 用户环境变量（存储在注册表 HKCU\Environment），修改后会广播设置变更，新启动的程序立即生效。</p>

    <section v-if="unsupported" class="card">
      <p class="empty">环境变量管理目前仅支持 Windows 桌面端。在 macOS 上可使用终端编辑 ~/.zshrc 或用 launchctl setenv 管理环境变量。</p>
    </section>

    <template v-else>
      <div class="tabs" role="tablist" aria-label="系统配置范围">
        <button
          type="button"
          role="tab"
          :aria-selected="tab === 'user'"
          :class="{ active: tab === 'user' }"
          @click="switchTab('user')"
        >用户变量</button>
        <button
          type="button"
          role="tab"
          :aria-selected="tab === 'system'"
          :class="{ active: tab === 'system' }"
          @click="switchTab('system')"
        >系统变量</button>
        <button
          type="button"
          role="tab"
          :aria-selected="tab === 'hosts'"
          :class="{ active: tab === 'hosts' }"
          @click="switchTab('hosts')"
        >Hosts 文件</button>
      </div>

      <template v-if="tab === 'hosts'">
        <section class="card">
          <div class="toolbar">
            <span class="count hosts-path">C:\Windows\System32\drivers\etc\hosts</span>
            <button type="button" class="btn btn-outline" :disabled="hostsLoading" @click="loadHosts">重载</button>
            <button type="button" class="btn" :disabled="!canSaveHosts" @click="saveHosts">
              {{ hostsSaving ? '正在保存…' : '保存' }}
            </button>
          </div>

          <p v-if="hostsElevated === false" class="warning">
            当前以普通权限运行，保存时会弹出 UAC 授权窗口；取消授权则不会修改文件。
          </p>
          <p v-if="hostsStatus" class="status" aria-live="polite">{{ hostsStatus }}</p>
          <p v-if="hostsError" class="error" role="alert">{{ hostsError }}</p>

          <textarea
            v-model="hostsContent"
            class="input hosts-input"
            rows="22"
            spellcheck="false"
            aria-label="hosts 文件内容"
            :disabled="hostsLoading"
            @input="hostsDirty = true"
          ></textarea>
          <p class="helper">每行一条映射：IP 地址 主机名 [别名...]，# 开头为注释。保存前自动备份为 hosts.bak，保存后自动刷新 DNS 缓存，新解析立即生效。</p>
        </section>
      </template>

      <template v-else>
      <p v-if="tab === 'system' && data && !data.elevated" class="warning">
        系统环境变量位于注册表 HKLM\SYSTEM\...\Session Manager\Environment。当前以普通权限运行，保存或删除系统变量时会弹出 UAC 授权窗口；取消授权则不会修改。
      </p>

      <section class="card">
        <div class="toolbar">
          <input
            v-model="query"
            class="input search"
            type="search"
            autocomplete="off"
            spellcheck="false"
            placeholder="搜索变量名或值"
            aria-label="搜索环境变量"
          >
          <span class="count">{{ loading ? '加载中…' : `共 ${filteredVars.length} 项` }}</span>
          <button v-if="data?.writable" type="button" class="btn" @click="openCreate">新建变量</button>
          <button type="button" class="btn btn-outline" :disabled="loading" @click="load(tab as EnvVarScope)">刷新</button>
        </div>

        <p v-if="status" class="status" aria-live="polite">{{ status }}</p>
        <p v-if="error" class="error" role="alert">{{ error }}</p>

        <div v-if="filteredVars.length" class="vars">
          <article v-for="entry in filteredVars" :key="entry.name" class="var-row">
            <div class="var-main">
              <div class="var-name">
                <strong>{{ entry.name }}</strong>
                <span class="badge" :class="{ expand: entry.typeName === 'REG_EXPAND_SZ' }">{{ typeLabel(entry.typeName) }}</span>
              </div>
              <div class="var-value">
                <template v-if="isListVar(entry)">
                  <p class="helper list-count">共 {{ listEntries(entry).length }} 条路径</p>
                  <ul class="value-text path-items" :class="{ clamped: canExpand(entry) && !expanded.has(entry.name) }">
                    <li v-for="(item, index) in listEntries(entry)" :key="index">{{ item || '（空）' }}</li>
                  </ul>
                </template>
                <p v-else class="value-text" :class="{ clamped: canExpand(entry) && !expanded.has(entry.name) }">{{ entry.rawValue || '（空）' }}</p>
                <p v-if="showExpandedHelper(entry)" class="helper">
                  含 %VAR% 引用，展开后：{{ entry.expandedValue }}
                </p>
                <button
                  v-if="canExpand(entry)"
                  type="button"
                  class="link-btn"
                  @click="toggleExpand(entry)"
                >{{ expanded.has(entry.name) ? '收起' : '展开' }}</button>
              </div>
            </div>
            <div class="var-actions">
              <button type="button" class="copy-btn" @click="copy(entry.rawValue)">复制</button>
              <template v-if="data?.writable && entry.editable">
                <button type="button" class="copy-btn" @click="openEdit(entry)">编辑</button>
                <button v-if="pendingDelete === entry.name" type="button" class="confirm-btn" @click="remove(entry.name)">确认删除</button>
                <button v-else type="button" class="copy-btn danger" @click="askDelete(entry.name)">删除</button>
              </template>
            </div>
            <p v-if="pendingDelete === entry.name && isCritical(entry.name)" class="warning">
              {{ entry.name }} 是常见的系统关键变量，删除后可能导致程序无法正常运行。
            </p>
          </article>
        </div>
        <p v-else-if="!loading" class="empty">没有匹配的环境变量。{{ query ? '换个关键词试试，' : '' }}点击右上角「刷新」重新读取。</p>
      </section>
      </template>
    </template>

    <div v-if="dialog" class="overlay" @click.self="closeDialog">
      <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="env-dialog-title">
        <h3 id="env-dialog-title">{{ dialog.mode === 'create' ? '新建变量' : `编辑 ${dialog.originalName}` }}</h3>

        <label for="env-var-name">变量名</label>
        <input
          id="env-var-name"
          v-model="dialog.name"
          class="input"
          autocomplete="off"
          spellcheck="false"
          :disabled="dialog.mode === 'edit'"
          placeholder="例如 SBOX_HOME"
        >
        <p v-if="nameConflict" class="warning">已存在同名变量（Windows 变量名不区分大小写），保存将覆盖 {{ nameConflict }}。</p>

        <p v-if="dialog.mode === 'edit' && dialog.originalTypeName === 'REG_EXPAND_SZ'" class="helper">
          该变量为展开型（REG_EXPAND_SZ），值中的 %VAR% 引用会在程序读取时自动展开，保存时保持该类型。
        </p>
        <p v-else-if="effectiveTypeName === 'REG_EXPAND_SZ'" class="helper">
          值中含 %VAR% 引用，将以展开型（REG_EXPAND_SZ）保存，程序读取时自动展开为实际路径。
        </p>

        <label for="env-var-value">值</label>
        <template v-if="showListEditor">
          <div class="path-list">
            <div v-for="(entry, index) in dialog.pathEntries" :key="index" class="path-entry">
              <input
                v-model="dialog.pathEntries[index]"
                class="input"
                autocomplete="off"
                spellcheck="false"
                :aria-label="`第 ${index + 1} 条路径`"
                placeholder="(空)"
              >
              <div class="path-entry-actions">
                <button
                  type="button"
                  class="copy-btn icon-btn"
                  :disabled="index === 0"
                  aria-label="上移"
                  title="上移"
                  @click="movePathEntry(index, -1)"
                >↑</button>
                <button
                  type="button"
                  class="copy-btn icon-btn"
                  :disabled="index === dialog.pathEntries.length - 1"
                  aria-label="下移"
                  title="下移"
                  @click="movePathEntry(index, 1)"
                >↓</button>
                <button type="button" class="copy-btn danger" @click="removePathEntry(index)">删除</button>
              </div>
            </div>
          </div>
          <div class="path-list-footer">
            <button type="button" class="copy-btn" @click="addPathEntry">添加条目</button>
            <button type="button" class="link-btn" @click="switchToTextEditor">切换为文本编辑</button>
            <span class="count">{{ dialog.pathEntries.length }} 条 · {{ effectiveValueLength }} 个字符</span>
          </div>
          <p v-if="hasEmptyEntry" class="warning">存在空条目，保存后会原样保留（分号间为空）。</p>
        </template>
        <template v-else>
          <textarea
            id="env-var-value"
            v-model="dialog.value"
            class="input value-input"
            rows="5"
            spellcheck="false"
          ></textarea>
          <p class="helper">{{ dialog.value.length }} 个字符</p>
          <p v-if="dialogIsPath" class="helper">
            PATH 是分号分隔的列表，建议逐条编辑。
            <button type="button" class="link-btn" @click="switchToListEditor">切换为列表编辑</button>
          </p>
        </template>
        <p v-if="effectiveValueLength > VALUE_MAX_LENGTH" class="error">值过长（超过 {{ VALUE_MAX_LENGTH }} 个字符），无法保存。</p>
        <p v-else-if="effectiveValueLength > VALUE_WARN_LENGTH" class="warning">值较长，部分程序可能无法完整读取。</p>

        <p v-if="dialogError" class="error" role="alert">{{ dialogError }}</p>

        <div class="dialog-actions">
          <button type="button" class="btn btn-outline" @click="closeDialog">取消</button>
          <button type="button" class="btn" :disabled="!canSave" @click="saveDialog">
            {{ saving ? '正在保存…' : '保存' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.env-vars { max-width: 900px; margin: 0 auto; }
.lead { margin: 0 0 16px; color: var(--fg-muted); }
.card { padding: 20px; margin-bottom: 16px; border: 1px solid var(--border); border-radius: var(--radius); background: var(--card); }
.input { min-width: 0; padding: 7px 10px; border: 1px solid var(--border); border-radius: 5px; background: var(--bg); color: var(--fg); font: 13px ui-monospace, SFMono-Regular, Consolas, monospace; }
.input:disabled { opacity: 0.6; }
.input:focus-visible, button:focus-visible { outline: 2px solid var(--primary); outline-offset: 2px; }
.helper, .error, .warning, .empty, .status { margin: 10px 0 0; font-size: 13px; }
.helper { color: var(--fg-muted); }
.error { color: var(--danger); }
.warning { padding: 10px 12px; border: 1px solid #d97706; border-radius: 5px; color: #a15c06; background: color-mix(in srgb, #d97706 8%, var(--card)); }
.status { color: var(--success); }
.empty { color: var(--fg-muted); }

.tabs { display: flex; gap: 4px; margin-bottom: 14px; border-bottom: 1px solid var(--border); }
.tabs button { padding: 9px 14px; border: none; border-bottom: 2px solid transparent; background: transparent; color: var(--fg-muted); font-size: 14px; cursor: pointer; }
.tabs button:hover { color: var(--fg); }
.tabs button.active { border-bottom-color: var(--primary); color: var(--fg); font-weight: 600; }
.tabs + .warning { margin: 0 0 14px; }

.toolbar { display: flex; gap: 8px; align-items: center; }
.toolbar .search { flex: 1 1 auto; min-height: 36px; }
.count { flex: 0 0 auto; color: var(--fg-muted); font-size: 13px; white-space: nowrap; }

.vars { display: grid; gap: 10px; margin-top: 16px; }
.var-row { padding: 12px 14px; border: 1px solid var(--border); border-radius: 6px; background: var(--bg); }
.var-row .warning { margin-top: 8px; }
.var-main { display: grid; grid-template-columns: 220px minmax(0, 1fr); gap: 14px; }
.var-name { display: flex; flex-wrap: wrap; gap: 6px; align-items: center; overflow-wrap: anywhere; }
.badge { padding: 1px 7px; border: 1px solid var(--border); border-radius: 999px; color: var(--fg-muted); font-size: 12px; white-space: nowrap; }
.badge.expand { color: var(--primary); border-color: color-mix(in srgb, var(--primary) 40%, var(--border)); }
.var-value { min-width: 0; }
.value-text { margin: 0; overflow-wrap: anywhere; white-space: pre-wrap; font: 13px ui-monospace, SFMono-Regular, Consolas, monospace; }
.value-text.clamped { display: -webkit-box; overflow: hidden; -webkit-box-orient: vertical; -webkit-line-clamp: 3; }
.list-count { margin-top: 0; }
.path-items { margin: 0; padding: 0; list-style: none; }
.path-items li { position: relative; padding: 1px 0 1px 14px; overflow-wrap: anywhere; }
.path-items li::before { content: ''; position: absolute; left: 2px; top: 0.75em; width: 5px; height: 5px; border-radius: 50%; background: var(--fg-muted); opacity: 0.5; }
.link-btn { margin-top: 2px; padding: 0; border: none; background: transparent; color: var(--primary); font-size: 13px; cursor: pointer; }
.var-actions { display: flex; gap: 8px; justify-content: flex-end; margin-top: 10px; }
.copy-btn { min-height: 30px; padding: 4px 10px; border: 1px solid var(--border); border-radius: 5px; background: transparent; color: var(--fg); font-size: 13px; cursor: pointer; }
.copy-btn:hover { border-color: var(--primary); color: var(--primary); }
.copy-btn.danger:hover { border-color: var(--danger); color: var(--danger); }
.confirm-btn { min-height: 30px; padding: 4px 10px; border: 1px solid var(--danger); border-radius: 5px; background: color-mix(in srgb, var(--danger) 12%, transparent); color: var(--danger); font-size: 13px; cursor: pointer; }

.overlay { position: fixed; inset: 0; z-index: 20; display: flex; align-items: center; justify-content: center; padding: 20px; background: rgb(0 0 0 / 0.45); }
.dialog { width: min(560px, 100%); max-height: calc(100vh - 40px); overflow: auto; padding: 20px; border: 1px solid var(--border); border-radius: var(--radius); background: var(--card); }
.dialog h3 { margin: 0 0 16px; font-size: 16px; }
.dialog label { display: block; margin: 14px 0 6px; color: var(--fg-muted); font-size: 13px; }
.dialog .input { width: 100%; box-sizing: border-box; }
.value-input { resize: vertical; line-height: 1.5; }
.path-list { display: grid; gap: 6px; max-height: 300px; overflow: auto; padding: 2px; }
.path-entry { display: flex; gap: 6px; align-items: center; }
.path-entry .input { flex: 1 1 auto; }
.path-entry-actions { display: flex; gap: 4px; flex: 0 0 auto; }
.icon-btn { width: 30px; padding: 4px 0; font-family: inherit; }
.icon-btn:disabled { opacity: 0.4; cursor: default; }
.path-list-footer { display: flex; gap: 10px; align-items: center; margin-top: 8px; }
.path-list-footer .count { color: var(--fg-muted); font-size: 13px; }
.hosts-path { font: 12px ui-monospace, SFMono-Regular, Consolas, monospace; }
.hosts-input { display: block; width: 100%; box-sizing: border-box; margin-top: 12px; resize: vertical; line-height: 1.5; }
.dialog-actions { display: flex; gap: 10px; justify-content: flex-end; margin-top: 20px; }

@media (max-width: 720px) {
  .card { padding: 16px; }
  .toolbar { flex-wrap: wrap; }
  .toolbar .search { flex-basis: 100%; }
  .var-main { grid-template-columns: 1fr; gap: 8px; }
  .var-actions { justify-content: flex-start; }
}
</style>
