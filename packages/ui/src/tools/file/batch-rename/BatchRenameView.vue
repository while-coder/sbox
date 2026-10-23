<script setup lang="ts">
import { native, type FileDialogOptions } from '../../../platform/native'
const open = (options?: FileDialogOptions) => native.pickPaths(options)
const readText = () => native.readClipboardText()
import { computed, onMounted, onUnmounted, ref } from 'vue'



import { toast } from '@qingfeng346/ui-kit/composables/message.ts'
import { Check, ChevronRight, FolderOpen, Plus, TriangleAlert, X } from 'lucide-vue-next'
import {
  previewRename,
  type PreviewItem,
  type RenameRule,
  type RuleType,
} from './rename-engine'
import { executeBatchRename, isDirPath, listDirEntries, statPaths, type RenameResultItem } from './native'
import RuleCard from './RuleCard.vue'
import SButton from '@qingfeng346/ui-kit/components/button/SButton.vue'
import SCheckbox from '@qingfeng346/ui-kit/components/checkbox/SCheckbox.vue'
import SSplit from '@qingfeng346/ui-kit/components/SSplit.vue'
import SEllipsis from '@qingfeng346/ui-kit/components/SEllipsis.vue'
import SEmpty from '@qingfeng346/ui-kit/components/SEmpty.vue'
import SSortableList from '@qingfeng346/ui-kit/components/sortable/SSortableList.vue'
import SSortableItem from '@qingfeng346/ui-kit/components/sortable/SSortableItem.vue'

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
const clipboard = ref<string>()
const readingClipboard = ref(false)
const usesClipboard = computed(() => rules.value.some(rule => rule.enabled && rule.type === 'insert' && rule.text.includes('{Clipboard}')))

const preview = computed(() => previewRename(rules.value, files.value, { clipboard: clipboard.value }))

async function refreshClipboard() {
  if (readingClipboard.value) return
  readingClipboard.value = true
  clipboard.value = undefined
  try {
    clipboard.value = await readText()
  } catch (error) {
    toast.show('error', `读取剪贴板失败：${String(error)}`)
  } finally {
    readingClipboard.value = false
  }
}

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
// HTML5 DnD 的 drop 事件不会触发；规则使用 ui-kit 排序，文件行保留整行拖动 ----

const draggingFiles = ref(false)

/** 行内表单控件与按钮上的按下不启动拖拽 */
function beginFileDrag(event: PointerEvent) {
  if (event.button !== 0) return
  if ((event.target as HTMLElement).closest('input, textarea, select, button, a, label')) return
  const rowEl = event.currentTarget as HTMLElement
  const container = rowEl.parentElement
  if (!container) return

  draggingFiles.value = true
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

    const rows = [...container.querySelectorAll<HTMLElement>('.preview-row:not(.head-row)')]
    const at = rows.findIndex(row => {
      const r = row.getBoundingClientRect()
      return lastY >= r.top && lastY < r.bottom
    })
    const from = rows.indexOf(rowEl)
    if (at < 0 || from < 0 || from === at) return
    const list = files.value
    const [item] = list.splice(from, 1)
    list.splice(at, 0, item)
    sortKey.value = null
  }

  const onMove = (e: PointerEvent) => {
    lastY = e.clientY
    if (!raf) raf = requestAnimationFrame(applyMove)
  }
  const onUp = () => {
    draggingFiles.value = false
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
  const selection = await open({ title: '选择要重命名的文件', multiple: true })
  if (Array.isArray(selection)) await addFiles(selection)
  else if (typeof selection === 'string') await addFiles([selection])
}

async function chooseFolder() {
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
    toast.show('error', String(e?.message || e))
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
    toast.show('error', String(e?.message || e))
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
  unlistenDrop = await native.onPathDrop((paths) => { void addFiles(paths) })
})

onUnmounted(() => {
  unlistenDrop?.()
})
</script>

<template>
  <div class="batch-rename">
    <h2>批量重命名</h2>
    <p class="lead">添加或拖入文件，在上方配置规则，在下方预览结果后执行重命名。</p>

    <section class="card">
      <div class="toolbar">
        <SButton size="small" :disabled="adding" @click="chooseFiles">
          <Plus :size="15" /> 添加文件
        </SButton>
        <SButton size="small" :disabled="adding" @click="chooseFolder">
          <FolderOpen :size="15" /> 添加文件夹
        </SButton>
        <SCheckbox v-model:checked="recursive" title="勾选后添加文件夹时会递归收集所有子文件夹中的文件">包含子文件夹</SCheckbox>
        <SButton size="small" type="error" tertiary :disabled="!files.length" @click="clearFiles">
          <X :size="15" /> 清空列表
        </SButton>
        <span class="toolbar-note">{{ adding ? '正在读取…' : files.length ? `已添加 ${files.length} 个文件` : '尚未添加文件' }}</span>
      </div>
    </section>

    <section class="card workbench">
      <SSplit class="panes" direction="vertical" :default-size="0.4" :min="0.2" :max="0.7">
        <template #1>
        <div class="rules-pane">
          <div class="pane-head">
            <h3>规则（按顺序执行）</h3>
            <SButton size="small" @click="addRule">
              <Plus :size="15" /> 添加规则
            </SButton>
          </div>
          <SEmpty v-if="!rules.length" description="还没有规则。点击「添加规则」，规则会从上到下依次作用于每个文件名。" />
          <SSortableList :items="rules" label="规则" @reorder="rules = $event">
          <SSortableItem
            v-for="(rule, index) in rules"
            :key="rule.id"
            class="rule-slot"
            :index="index"
            :label="`规则 ${index + 1}`"
          >
            <RuleCard
              :rule="rule"
              :index="index"
              @update="fields => Object.assign(rule, fields)"
              @change-type="(type: RuleType) => changeType(rule, type)"
              @remove="removeRule(index)"
              @read-clipboard="refreshClipboard"
            />
          </SSortableItem>
          </SSortableList>
        </div>
        </template>
        <template #2>
        <div class="preview-pane">
          <div class="pane-head stacked">
            <div class="head-line">
              <h3>预览</h3>
              <SButton size="small" text v-if="usesClipboard" :disabled="readingClipboard" @click="refreshClipboard">{{ readingClipboard ? '正在读取…' : '重新读取剪贴板' }}</SButton>
              <span class="summary">
                {{ files.length }} 个文件 · {{ counts.willRename }} 将重命名 · {{ counts.unchanged }} 无变化 · {{ counts.conflict }} 冲突
              </span>
            </div>
            <div v-if="files.length > 1" class="sort-row">
              <span class="sort-label">排序</span>
              <SButton size="small" text v-for="opt in sortOptions"
                :key="opt.key"

                class="sort-btn"
                :class="{ active: sortKey === opt.key }"
                @click="sortBy(opt.key)">{{ opt.label }}{{ sortKey === opt.key ? (sortAsc ? ' ↑' : ' ↓') : '' }}</SButton>
              <span class="sort-hint">拖动行可手动调整顺序</span>
            </div>
          </div>
          <SEmpty v-if="!files.length" description="还没有文件。点击上方「添加文件」或直接把文件拖进窗口。" />
          <div v-else class="preview-list">
            <div class="preview-row head-row">
              <span>原始文件名</span>
              <span>新文件名</span>
              <span>所在文件夹</span>
              <span>状态</span>
              <span>操作</span>
            </div>
            <div
              v-for="(item, i) in preview"
              :key="files[i].path"
              class="preview-row"
              :class="[`status-${item.status}`, { dragging: draggingFiles }]"
              @pointerdown="beginFileDrag($event)"
            >
              <span class="name" :title="files[i].path">{{ item.originalName }}</span>
              <span class="name final">
                <ChevronRight class="arrow" :size="14" />
                {{ item.finalName }}
              </span>
              <SEllipsis class="folder-path">{{ parentDir(files[i].path) }}</SEllipsis>
              <span class="file-status" :title="item.statusDetail">{{ item.status === 'ok' ? '将重命名' : statusText(item) }}</span>
              <span class="row-actions">
                <SButton size="small" text v-if="item.steps.length"


                  @click="toggleExpand(files[i].path)">{{ expanded.has(files[i].path) ? '收起' : '步骤' }}</SButton>
                <SButton size="small" text @click="removeFile(i)">移除</SButton>
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
        </template>
      </SSplit>
    </section>

    <section class="card footer-bar">
      <span class="footer-note">
        <template v-if="!files.length">先添加要重命名的文件</template>
        <template v-else-if="counts.conflict">存在命名冲突，请调整规则后再执行</template>
        <template v-else-if="!counts.willRename">所有文件名均无变化</template>
        <template v-else>将重命名 {{ counts.willRename }} 个文件</template>
      </span>
      <SButton size="small" type="primary" :disabled="!canExecute" @click="execute">
        {{ executing ? '正在执行…' : '执行重命名' }}
      </SButton>
    </section>
  </div>
</template>

<style scoped>
/* 上方规则、下方文件预览，两个区域独立滚动，工具栏和执行栏固定。 */
.batch-rename { max-width: 96%; margin: 0 auto; height: 100%; display: flex; flex-direction: column; }
.lead { margin: 0 0 12px; color: var(--fg-muted); flex: 0 0 auto; font-size: 13px; }
.card { padding: 12px 16px; margin-bottom: 12px; border: 1px solid var(--border); border-radius: var(--radius); background: var(--card); }
h3 { margin: 0; font-size: 14px; }
.toolbar { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; flex: 0 0 auto; }
.toolbar-note { color: var(--fg-muted); font-size: 13px; }
.workbench { flex: 1 1 auto; min-height: 0; overflow: hidden; display: flex; flex-direction: column; padding: 0; }
.panes { flex: 1 1 auto; min-height: 0; }
.panes :deep(.s-split-pane) { overflow: hidden; }
.rules-pane { height: 100%; box-sizing: border-box; min-width: 0; min-height: 0; overflow-y: auto; padding: 12px 16px; }
.preview-pane { height: 100%; box-sizing: border-box; min-width: 0; min-height: 0; display: flex; flex-direction: column; border-top: 1px solid var(--border); padding: 12px 16px; }
.preview-pane > .pane-head { flex: 0 0 auto; }
.pane-head { position: sticky; top: 0; z-index: 1; display: flex; gap: 12px; align-items: center; justify-content: space-between; flex-wrap: wrap; padding-bottom: 10px; background: var(--card); }
.pane-head.stacked { flex-direction: column; align-items: stretch; gap: 8px; }
.head-line { display: flex; gap: 12px; align-items: center; justify-content: space-between; flex-wrap: wrap; }
.sort-row { display: flex; gap: 10px; align-items: center; flex-wrap: wrap; }
.sort-label, .sort-hint { font-size: 12px; color: var(--fg-muted); }
.sort-hint { margin-left: auto; }
.sort-btn.active { font-weight: 600; }
.summary { color: var(--fg-muted); font-size: 12px; }
.rule-slot { margin-bottom: 6px; }
.rule-slot :deep(> .sort-handle) { position: absolute; top: 10px; left: 8px; z-index: 1; }
.preview-list { border: 1px solid var(--border); border-radius: 6px; overflow-y: auto; min-height: 0; flex: 1 1 auto; }
.preview-row { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1.2fr) minmax(0, 1fr) minmax(80px, 0.4fr) 76px; gap: 10px; align-items: start; padding: 6px 12px; border-top: 1px solid var(--border); font-size: 13px; cursor: grab; }
.preview-row:active { cursor: grabbing; }
.preview-row.head-row { position: sticky; top: 0; z-index: 1; border-top: none; background: var(--bg); color: var(--fg-muted); font-size: 12px; cursor: default; }
.name { overflow-wrap: anywhere; font: 12px ui-monospace, SFMono-Regular, Consolas, monospace; }
.folder-path { color: var(--fg-muted); font: 12px ui-monospace, SFMono-Regular, Consolas, monospace; }
.final { display: flex; gap: 4px; align-items: baseline; flex-wrap: wrap; }
.arrow { flex: 0 0 auto; align-self: center; color: var(--fg-muted); }
.status-unchanged .final { color: var(--fg-muted); }
.status-conflict-duplicate .final, .status-conflict-exists .final, .status-invalid .final { color: var(--danger); }
.file-status { color: var(--fg-muted); font-size: 12px; overflow-wrap: anywhere; }
.status-ok .file-status { color: var(--success); }
.status-conflict-duplicate .file-status, .status-conflict-exists .file-status, .status-invalid .file-status { color: var(--danger); }
.row-actions { display: flex; gap: 8px; }
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
@media (max-width: 600px) {
  .rules-pane, .preview-pane { padding: 10px; }
  .preview-row { grid-template-columns: minmax(0, 1fr) minmax(0, 1fr) minmax(0, 0.8fr) 64px 60px; gap: 6px; padding: 6px 8px; }
  .row-actions { gap: 4px; flex-wrap: wrap; }
}
</style>
