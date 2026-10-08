<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { Check, ChevronRight, FolderOpen, Plus, TriangleAlert, X } from 'lucide-vue-next'
import {
  previewRename,
  type PreviewItem,
  type RenameRule,
  type RuleType,
} from './rename-engine'
import { executeBatchRename, isDirPath, listDirEntries, statPaths, type RenameResultItem } from './tauri'
import RuleCard from './RuleCard.vue'

interface FileEntry {
  path: string
  name: string
  /** 字节；获取失败时不参与大小排序（视为最小） */
  size?: number
  /** Unix 毫秒；获取失败时同上 */
  createdMs?: number | null
  modifiedMs?: number | null
}

const files = ref<FileEntry[]>([])
const rules = ref<RenameRule[]>([])
const expanded = ref<Set<string>>(new Set())
/** 执行结果，key 为源路径 */
const results = ref<Record<string, RenameResultItem>>({})
const adding = ref(false)
/** 添加文件夹时是否递归收集子文件夹中的文件 */
const recursive = ref(false)
const executing = ref(false)
const error = ref('')

const preview = computed(() => previewRename(rules.value, files.value.map(f => f.name)))

const counts = computed(() => {
  let willRename = 0
  let unchanged = 0
  let conflict = 0
  for (const item of preview.value) {
    if (item.status === 'ok') willRename += 1
    else if (item.status === 'unchanged') unchanged += 1
    else conflict += 1
  }
  return { willRename, unchanged, conflict }
})

const canExecute = computed(() =>
  files.value.length > 0
  && counts.value.willRename > 0
  && counts.value.conflict === 0
  && !executing.value,
)

// ---- 列表拖拽排序（pointer 实现）：Windows 上 Tauri 接管了 WebView2 的原生 drop 处理，
// HTML5 DnD 的 drop 事件不会触发，因此规则与文件行都用 pointermove 手动换位 ----

type DragList = 'rule' | 'file'

const draggingList = ref<DragList | null>(null)

const ROW_SELECTOR: Record<DragList, string> = {
  rule: '.rule-slot',
  file: '.preview-row:not(.head-row)',
}

/** 行内表单控件与按钮上的按下不启动拖拽 */
function beginListDrag(event: PointerEvent, kind: DragList) {
  if (event.button !== 0) return
  if ((event.target as HTMLElement).closest('input, textarea, select, button, a, label')) return
  const rowEl = event.currentTarget as HTMLElement
  const container = rowEl.parentElement
  if (!container) return

  draggingList.value = kind
  rowEl.style.opacity = '0.5'
  document.body.style.userSelect = 'none'

  let raf = 0
  let lastY = event.clientY

  // rAF 中处理：Vue 的 DOM 重排是异步的，按帧读取行位置才能与列表状态同步
  const applyMove = () => {
    raf = 0
    // 拖到容器上下边缘时自动滚动
    const rect = container.getBoundingClientRect()
    if (lastY < rect.top + 32) container.scrollTop -= 14
    else if (lastY > rect.bottom - 32) container.scrollTop += 14

    const rows = [...container.querySelectorAll<HTMLElement>(ROW_SELECTOR[kind])]
    const at = rows.findIndex(row => {
      const r = row.getBoundingClientRect()
      return lastY >= r.top && lastY < r.bottom
    })
    const from = rows.indexOf(rowEl)
    if (at < 0 || from < 0 || from === at) return
    const list = kind === 'rule' ? rules.value : files.value
    const [item] = list.splice(from, 1)
    list.splice(at, 0, item)
    if (kind === 'file') sortKey.value = null
  }

  const onMove = (e: PointerEvent) => {
    lastY = e.clientY
    if (!raf) raf = requestAnimationFrame(applyMove)
  }
  const onUp = () => {
    draggingList.value = null
    rowEl.style.opacity = ''
    document.body.style.userSelect = ''
    window.removeEventListener('pointermove', onMove)
    window.removeEventListener('pointerup', onUp)
    window.removeEventListener('pointercancel', onUp)
  }
  window.addEventListener('pointermove', onMove)
  window.addEventListener('pointerup', onUp)
  window.addEventListener('pointercancel', onUp)
}

function basename(p: string): string {
  return p.slice(Math.max(p.lastIndexOf('\\'), p.lastIndexOf('/')) + 1)
}

function parentDir(p: string): string {
  const i = Math.max(p.lastIndexOf('\\'), p.lastIndexOf('/'))
  return i >= 0 ? p.slice(0, i + 1) : ''
}

/** 添加文件路径（过滤文件夹），按完整路径去重（Windows 不区分大小写），并批量取大小/时间元数据供排序 */
async function addFiles(paths: string[]) {
  if (!paths.length) return
  adding.value = true
  error.value = ''
  try {
    const flags = await Promise.all(paths.map(p => isDirPath(p).catch(() => false)))
    const seen = new Set(files.value.map(f => f.path.toLowerCase()))
    const toAdd: string[] = []
    paths.forEach((p, i) => {
      if (flags[i]) return
      const key = p.toLowerCase()
      if (seen.has(key)) return
      seen.add(key)
      toAdd.push(p)
    })
    if (!toAdd.length) return
    const metas = await statPaths(toAdd).catch(() => toAdd.map(() => null))
    toAdd.forEach((p, i) => {
      files.value.push({ path: p, name: basename(p), ...metas[i] ?? {} })
    })
  } finally {
    adding.value = false
  }
}

function createRule(type: RuleType): RenameRule {
  const base = { id: crypto.randomUUID(), enabled: true, applyToExtension: false }
  switch (type) {
    case 'replace':
      return { ...base, type, find: '', replacement: '', useRegex: false, caseSensitive: false }
    case 'insert':
      return { ...base, type, text: '', position: 'end', index: 0, beforeExtension: true }
    case 'sequence':
      return { ...base, type, start: 1, step: 1, pad: 0, position: 'suffix', index: 0, template: '' }
    case 'case':
      return { ...base, type, mode: 'lower' }
    case 'remove':
      return { ...base, type, mode: 'range', rangeStart: 0, rangeEnd: 1, chars: '' }
  }
}

function addRule() {
  rules.value.push(createRule('replace'))
}

/** 切换类型：生成该类型默认参数，保留 id（预览步骤链不断）与启用状态 */
function changeType(rule: RenameRule, type: RuleType) {
  const index = rules.value.indexOf(rule)
  if (index < 0) return
  rules.value[index] = { ...createRule(type), id: rule.id, enabled: rule.enabled }
}

function removeRule(index: number) {
  rules.value.splice(index, 1)
}

// ---- 文件列表排序：拖动行手动调序，或按名称/大小/时间一键排序 ----

type SortKey = 'name' | 'size' | 'created' | 'modified'

const sortOptions: { key: SortKey; label: string }[] = [
  { key: 'name', label: '名称' },
  { key: 'size', label: '大小' },
  { key: 'created', label: '创建时间' },
  { key: 'modified', label: '修改时间' },
]

const sortKey = ref<SortKey | null>(null)
const sortAsc = ref(true)

/** 点击已选中的键切换升降序，点击新键默认升序；直接重排 files（预览与执行都按该顺序） */
function sortBy(key: SortKey) {
  if (sortKey.value === key) sortAsc.value = !sortAsc.value
  else {
    sortKey.value = key
    sortAsc.value = true
  }
  const dir = sortAsc.value ? 1 : -1
  const valueOf = (f: FileEntry): string | number =>
    key === 'name' ? f.name.toLowerCase()
    : key === 'size' ? f.size ?? -1
    : key === 'created' ? f.createdMs ?? -1
    : f.modifiedMs ?? -1
  files.value.sort((a, b) => {
    const va = valueOf(a)
    const vb = valueOf(b)
    const cmp = typeof va === 'string' || typeof vb === 'string'
      ? String(va).localeCompare(String(vb), undefined, { numeric: true })
      : va - vb
    return cmp * dir
  })
}

async function chooseFiles() {
  error.value = ''
  const selection = await open({ title: '选择要重命名的文件', multiple: true })
  if (Array.isArray(selection)) await addFiles(selection)
  else if (typeof selection === 'string') await addFiles([selection])
}

async function chooseFolder() {
  error.value = ''
  const selection = await open({
    title: recursive.value ? '选择文件夹（包含子文件夹中的所有文件）' : '选择文件夹（只添加其中的文件，不含子文件夹）',
    directory: true,
    multiple: false,
  })
  if (typeof selection !== 'string') return
  try {
    const entries = await listDirEntries(selection, recursive.value)
    await addFiles(entries.filter(e => !e.isDir).map(e => e.path))
  } catch (e: any) {
    error.value = String(e?.message || e)
  }
}

function clearFiles() {
  files.value = []
  expanded.value.clear()
  results.value = {}
}

function removeFile(index: number) {
  const [entry] = files.value.splice(index, 1)
  if (entry) expanded.value.delete(entry.path)
}

function toggleExpand(path: string) {
  if (expanded.value.has(path)) expanded.value.delete(path)
  else expanded.value.add(path)
}

async function execute() {
  if (!canExecute.value) return
  executing.value = true
  error.value = ''
  results.value = {}
  try {
    const items = preview.value
      .map((item, i) => ({ item, file: files.value[i] }))
      .filter(({ item }) => item.status === 'ok')
      .map(({ item, file }) => ({ from: file.path, to: parentDir(file.path) + item.finalName }))
    const result = await executeBatchRename(items)
    const byPath: Record<string, RenameResultItem> = {}
    for (const item of result.items) byPath[item.from] = item
    results.value = byPath
    // 已改名的文件原地更新列表，便于继续调整规则
    for (const item of result.items) {
      if (item.status !== 'renamed') continue
      const entry = files.value.find(f => f.path.toLowerCase() === item.from.toLowerCase())
      if (entry) {
        entry.path = item.to
        entry.name = basename(item.to)
      }
    }
  } catch (e: any) {
    error.value = String(e?.message || e)
  } finally {
    executing.value = false
  }
}

function statusText(item: PreviewItem): string {
  switch (item.status) {
    case 'unchanged': return '无变化'
    case 'conflict-duplicate': return '批内重名'
    case 'conflict-exists': return '与批内原始名冲突'
    case 'invalid': return '无效'
    default: return ''
  }
}

function execText(result: RenameResultItem): string {
  if (result.status === 'renamed') return '已重命名'
  if (result.status === 'skipped') return result.error ?? '已跳过'
  return result.error ?? '失败'
}

let unlistenDrop: (() => void) | null = null

onMounted(async () => {
  unlistenDrop = await getCurrentWebview().onDragDropEvent((event) => {
    if (event.payload.type === 'drop') void addFiles(event.payload.paths)
  })
})

onUnmounted(() => {
  unlistenDrop?.()
})
</script>

<template>
  <div class="batch-rename">
    <h2>批量重命名</h2>
    <p class="lead">添加文件，按顺序叠加规则（搜索替换、插入、序号、大小写、删除），实时预览最终文件名后一次性执行。文件可直接拖入窗口。</p>

    <section class="card">
      <div class="toolbar">
        <button type="button" class="btn btn-outline" :disabled="adding" @click="chooseFiles">
          <Plus :size="15" /> 添加文件
        </button>
        <button type="button" class="btn btn-outline" :disabled="adding" @click="chooseFolder">
          <FolderOpen :size="15" /> 添加文件夹
        </button>
        <label class="check-inline" title="勾选后添加文件夹时会递归收集所有子文件夹中的文件">
          <input v-model="recursive" type="checkbox" /> 包含子文件夹
        </label>
        <button type="button" class="btn btn-outline danger" :disabled="!files.length" @click="clearFiles">
          <X :size="15" /> 清空列表
        </button>
        <span class="toolbar-note">{{ adding ? '正在读取…' : files.length ? `已添加 ${files.length} 个文件` : '尚未添加文件' }}</span>
      </div>
      <p v-if="error" class="error" role="alert">{{ error }}</p>
    </section>

    <section class="card workbench">
      <div class="columns">
        <div class="rules-pane">
          <div class="pane-head">
            <h3>规则（按顺序执行）</h3>
            <button type="button" class="btn btn-outline" @click="addRule">
              <Plus :size="15" /> 添加规则
            </button>
          </div>
          <p v-if="!rules.length" class="empty">还没有规则。点击「添加规则」，规则会从上到下依次作用于每个文件名。</p>
          <div
            v-for="(rule, index) in rules"
            :key="rule.id"
            class="rule-slot"
            :class="{ dragging: draggingList === 'rule' }"
            @pointerdown="beginListDrag($event, 'rule')"
          >
            <RuleCard
              :rule="rule"
              :index="index"
              @update="fields => Object.assign(rule, fields)"
              @change-type="(type: RuleType) => changeType(rule, type)"
              @remove="removeRule(index)"
            />
          </div>
        </div>

        <div class="preview-pane">
          <div class="pane-head stacked">
            <div class="head-line">
              <h3>预览</h3>
              <span class="summary">
                {{ files.length }} 个文件 · {{ counts.willRename }} 将重命名 · {{ counts.unchanged }} 无变化 · {{ counts.conflict }} 冲突
              </span>
            </div>
            <div v-if="files.length > 1" class="sort-row">
              <span class="sort-label">排序</span>
              <button
                v-for="opt in sortOptions"
                :key="opt.key"
                type="button"
                class="link-btn sort-btn"
                :class="{ active: sortKey === opt.key }"
                @click="sortBy(opt.key)"
              >{{ opt.label }}{{ sortKey === opt.key ? (sortAsc ? ' ↑' : ' ↓') : '' }}</button>
              <span class="sort-hint">拖动行可手动调整顺序</span>
            </div>
          </div>
          <p v-if="!files.length" class="empty">还没有文件。点击上方「添加文件」或直接把文件拖进窗口。</p>
          <div v-else class="preview-list">
            <div class="preview-row head-row">
              <span>原始文件名</span>
              <span>新文件名</span>
              <span />
            </div>
            <div
              v-for="(item, i) in preview"
              :key="files[i].path"
              class="preview-row"
              :class="[`status-${item.status}`, { dragging: draggingList === 'file' }]"
              @pointerdown="beginListDrag($event, 'file')"
            >
              <span class="name" :title="files[i].path">{{ item.originalName }}</span>
              <span class="name final">
                <ChevronRight class="arrow" :size="14" />
                {{ item.finalName }}
                <span v-if="item.status !== 'ok'" class="status-tag">{{ statusText(item) }}</span>
              </span>
              <span class="row-actions">
                <button
                  v-if="item.steps.length"
                  type="button"
                  class="link-btn"
                  @click="toggleExpand(files[i].path)"
                >{{ expanded.has(files[i].path) ? '收起' : '步骤' }}</button>
                <button type="button" class="link-btn" @click="removeFile(i)">移除</button>
              </span>
              <div v-if="expanded.has(files[i].path) && item.steps.length" class="steps">
                <div v-for="(step, s) in item.steps" :key="step.ruleId + s" class="step">
                  <span class="step-name">{{ s + 1 }}. {{ step.ruleLabel }}</span>
                  <code>{{ step.name }}</code>
                </div>
              </div>
              <div v-if="results[files[i].path]" class="exec-result" :class="`exec-${results[files[i].path].status}`">
                <Check v-if="results[files[i].path].status === 'renamed'" :size="13" />
                <TriangleAlert v-else :size="13" />
                {{ execText(results[files[i].path]) }}
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>

    <section class="card footer-bar">
      <span class="footer-note">
        <template v-if="!files.length">先添加要重命名的文件</template>
        <template v-else-if="counts.conflict">存在命名冲突，请调整规则后再执行</template>
        <template v-else-if="!counts.willRename">所有文件名均无变化</template>
        <template v-else>将重命名 {{ counts.willRename }} 个文件</template>
      </span>
      <button type="button" class="btn" :disabled="!canExecute" @click="execute">
        {{ executing ? '正在执行…' : '执行重命名' }}
      </button>
    </section>
  </div>
</template>

<style scoped>
/* 参考 JSON 查看器：整页占满窗口高度，工具栏/底栏固定，双栏各自内部滚动 */
.batch-rename { max-width: 92%; margin: 0 auto; height: 100%; display: flex; flex-direction: column; }
.lead { margin: 0 0 16px; color: var(--fg-muted); flex: 0 0 auto; }
.card { padding: 16px 20px; margin-bottom: 16px; border: 1px solid var(--border); border-radius: var(--radius); background: var(--card); }
h3 { margin: 0; font-size: 14px; }
.toolbar { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; flex: 0 0 auto; }
.btn { display: inline-flex; gap: 6px; align-items: center; }
.toolbar-note { color: var(--fg-muted); font-size: 13px; }
.check-inline { display: inline-flex; gap: 5px; align-items: center; font-size: 13px; color: var(--fg-muted); cursor: pointer; user-select: none; }
.btn-outline.danger:hover { border-color: var(--danger); color: var(--danger); }
.error { margin: 10px 0 0; font-size: 13px; color: var(--danger); }
.workbench { flex: 1 1 auto; min-height: 0; overflow: hidden; display: flex; flex-direction: column; }
.columns { display: grid; grid-template-columns: minmax(280px, 2fr) minmax(0, 3fr); gap: 20px; flex: 1 1 auto; min-height: 0; }
.rules-pane { min-width: 0; min-height: 220px; overflow-y: auto; }
.preview-pane { min-width: 0; min-height: 220px; overflow-y: auto; }
.pane-head { position: sticky; top: 0; z-index: 1; display: flex; gap: 12px; align-items: center; justify-content: space-between; flex-wrap: wrap; padding-bottom: 10px; background: var(--card); }
.pane-head.stacked { flex-direction: column; align-items: stretch; gap: 8px; }
.head-line { display: flex; gap: 12px; align-items: center; justify-content: space-between; flex-wrap: wrap; }
.sort-row { display: flex; gap: 10px; align-items: center; flex-wrap: wrap; }
.sort-label, .sort-hint { font-size: 12px; color: var(--fg-muted); }
.sort-hint { margin-left: auto; }
.sort-btn.active { font-weight: 600; }
.summary { color: var(--fg-muted); font-size: 12px; }
.empty { margin: 8px 0; font-size: 13px; color: var(--fg-muted); }
.rule-slot { margin-bottom: 10px; }
.rule-slot:active { cursor: grabbing; }
.preview-list { border: 1px solid var(--border); border-radius: 6px; overflow: hidden; }
.preview-row { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1.4fr) auto; gap: 10px; align-items: start; padding: 8px 12px; border-top: 1px solid var(--border); font-size: 13px; cursor: grab; }
.preview-row:active { cursor: grabbing; }
.preview-row.head-row { border-top: none; background: var(--bg); color: var(--fg-muted); font-size: 12px; cursor: default; }
.name { overflow-wrap: anywhere; font: 12px ui-monospace, SFMono-Regular, Consolas, monospace; }
.final { display: flex; gap: 4px; align-items: baseline; flex-wrap: wrap; }
.arrow { flex: 0 0 auto; align-self: center; color: var(--fg-muted); }
.status-unchanged .final { color: var(--fg-muted); }
.status-conflict-duplicate .final, .status-conflict-exists .final, .status-invalid .final { color: var(--danger); }
.status-tag { padding: 0 6px; border: 1px solid var(--danger); border-radius: 999px; font-size: 11px; }
.status-unchanged .status-tag { border-color: var(--border); }
.row-actions { display: flex; gap: 8px; }
.link-btn { padding: 0; border: none; background: none; color: var(--primary); font-size: 12px; cursor: pointer; }
.link-btn:hover { text-decoration: underline; }
.steps { grid-column: 1 / -1; display: grid; gap: 4px; padding: 8px 0 2px; }
.step { display: flex; gap: 10px; align-items: baseline; font-size: 12px; }
.step-name { flex: 0 0 auto; color: var(--fg-muted); }
.steps code { font: 12px ui-monospace, SFMono-Regular, Consolas, monospace; overflow-wrap: anywhere; }
.exec-result { grid-column: 1 / -1; display: inline-flex; gap: 5px; align-items: center; font-size: 12px; }
.exec-renamed { color: var(--success); }
.exec-skipped { color: var(--fg-muted); }
.exec-error { color: var(--danger); }
.footer-bar { display: flex; gap: 16px; align-items: center; justify-content: space-between; flex: 0 0 auto; margin-bottom: 0; }
.footer-note { font-size: 13px; color: var(--fg-muted); }
@media (max-width: 860px) {
  .columns { grid-template-columns: 1fr; }
}
</style>
