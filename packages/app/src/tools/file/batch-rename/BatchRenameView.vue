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
import { executeBatchRename, isDirPath, listDirEntries, type RenameResultItem } from './tauri'
import RuleCard from './RuleCard.vue'

interface FileEntry {
  path: string
  name: string
}

const files = ref<FileEntry[]>([])
const rules = ref<RenameRule[]>([])
const expanded = ref<Set<string>>(new Set())
/** 执行结果，key 为源路径 */
const results = ref<Record<string, RenameResultItem>>({})
const adding = ref(false)
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

const draggingIndex = ref(-1)

function basename(p: string): string {
  return p.slice(Math.max(p.lastIndexOf('\\'), p.lastIndexOf('/')) + 1)
}

function parentDir(p: string): string {
  const i = Math.max(p.lastIndexOf('\\'), p.lastIndexOf('/'))
  return i >= 0 ? p.slice(0, i + 1) : ''
}

/** 添加文件路径（过滤文件夹），按完整路径去重（Windows 不区分大小写） */
async function addFiles(paths: string[]) {
  if (!paths.length) return
  adding.value = true
  error.value = ''
  try {
    const flags = await Promise.all(paths.map(p => isDirPath(p).catch(() => false)))
    const seen = new Set(files.value.map(f => f.path.toLowerCase()))
    paths.forEach((p, i) => {
      if (flags[i]) return
      const key = p.toLowerCase()
      if (seen.has(key)) return
      seen.add(key)
      files.value.push({ path: p, name: basename(p) })
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

function onRuleDrop(targetIndex: number) {
  const from = draggingIndex.value
  draggingIndex.value = -1
  if (from < 0 || from === targetIndex) return
  const [rule] = rules.value.splice(from, 1)
  rules.value.splice(targetIndex, 0, rule)
}

async function chooseFiles() {
  error.value = ''
  const selection = await open({ title: '选择要重命名的文件', multiple: true })
  if (Array.isArray(selection)) await addFiles(selection)
  else if (typeof selection === 'string') await addFiles([selection])
}

async function chooseFolder() {
  error.value = ''
  const selection = await open({ title: '选择文件夹（只添加其中的文件，不含子文件夹）', directory: true, multiple: false })
  if (typeof selection !== 'string') return
  try {
    const entries = await listDirEntries(selection)
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
  // Set 的 delete/add 需要触发响应式更新
  expanded.value = new Set(expanded.value)
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
            draggable="true"
            @dragstart="draggingIndex = index"
            @dragover.prevent
            @drop="onRuleDrop(index)"
            @dragend="draggingIndex = -1"
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
          <div class="pane-head">
            <h3>预览</h3>
            <span class="summary">
              {{ files.length }} 个文件 · {{ counts.willRename }} 将重命名 · {{ counts.unchanged }} 无变化 · {{ counts.conflict }} 冲突
            </span>
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
              :class="`status-${item.status}`"
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
.batch-rename { max-width: 1100px; margin: 0 auto; }
.lead { margin: 0 0 16px; color: var(--fg-muted); }
.card { padding: 16px 20px; margin-bottom: 16px; border: 1px solid var(--border); border-radius: var(--radius); background: var(--card); }
h3 { margin: 0; font-size: 14px; }
.toolbar { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
.btn { display: inline-flex; gap: 6px; align-items: center; }
.toolbar-note { color: var(--fg-muted); font-size: 13px; }
.btn-outline.danger:hover { border-color: var(--danger); color: var(--danger); }
.error { margin: 10px 0 0; font-size: 13px; color: var(--danger); }
.columns { display: grid; grid-template-columns: minmax(280px, 2fr) minmax(0, 3fr); gap: 20px; }
.pane-head { display: flex; gap: 12px; align-items: center; justify-content: space-between; flex-wrap: wrap; margin-bottom: 12px; }
.summary { color: var(--fg-muted); font-size: 12px; }
.empty { margin: 8px 0; font-size: 13px; color: var(--fg-muted); }
.rules-pane { min-width: 0; }
.rule-slot { margin-bottom: 10px; }
.rule-slot:active { cursor: grabbing; }
.preview-pane { min-width: 0; }
.preview-list { border: 1px solid var(--border); border-radius: 6px; overflow: hidden; }
.preview-row { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1.4fr) auto; gap: 10px; align-items: start; padding: 8px 12px; border-top: 1px solid var(--border); font-size: 13px; }
.preview-row.head-row { border-top: none; background: var(--bg); color: var(--fg-muted); font-size: 12px; }
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
.footer-bar { display: flex; gap: 16px; align-items: center; justify-content: space-between; }
.footer-note { font-size: 13px; color: var(--fg-muted); }
@media (max-width: 860px) {
  .columns { grid-template-columns: 1fr; }
}
</style>
