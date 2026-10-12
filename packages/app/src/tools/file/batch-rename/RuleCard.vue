<script setup lang="ts">
import { computed, nextTick, ref } from 'vue'
import { ChevronDown, ChevronRight, GripVertical, Trash2 } from 'lucide-vue-next'
import { METADATA_TAGS, ruleError, ruleLabel, ruleTypeLabel, type RenameRule, type RuleType } from './rename-engine'

const props = defineProps<{
  rule: RenameRule
  index: number
}>()

const emit = defineEmits<{
  update: [patch: Partial<RenameRule>]
  changeType: [type: RuleType]
  remove: []
  readClipboard: []
}>()

const TYPES: RuleType[] = ['replace', 'insert', 'sequence', 'case', 'remove']

const collapsed = ref(false)
const insertInput = ref<HTMLInputElement | null>(null)
const tagPickerOpen = ref(false)
const tagDialog = ref<HTMLDialogElement | null>(null)
let insertStart = 0
let insertEnd = 0
const selectedTag = ref<string>('File_BaseName')
const tagInfo = computed(() => METADATA_TAGS.find(tag => tag.key === selectedTag.value)!)
const error = computed(() => ruleError(props.rule))
const label = computed(() => ruleLabel(props.rule))

function onTypeChange(event: Event) {
  emit('changeType', (event.target as HTMLSelectElement).value as RuleType)
}

function toggleHeader(event: MouseEvent) {
  if ((event.target as Element).closest('input, select, button, label, .grip')) return
  collapsed.value = !collapsed.value
}

async function insertTag() {
  if (props.rule.type !== 'insert') return
  const input = insertInput.value
  const start = insertStart
  const end = insertEnd
  const token = `{${selectedTag.value}}`
  emit('update', { text: props.rule.text.slice(0, start) + token + props.rule.text.slice(end) })
  if (selectedTag.value === 'Clipboard') emit('readClipboard')
  tagDialog.value?.close()
  await nextTick()
  input?.focus()
  input?.setSelectionRange(start + token.length, start + token.length)
}

function openTagPicker() {
  if (props.rule.type !== 'insert') return
  insertStart = insertInput.value?.selectionStart ?? props.rule.text.length
  insertEnd = insertInput.value?.selectionEnd ?? insertStart
  tagPickerOpen.value = true
  tagDialog.value?.showModal()
}
</script>

<template>
  <article class="rule-card">
    <header class="rule-head" @click="toggleHeader">
      <span class="grip" title="拖动排序" @click.stop><GripVertical :size="14" /></span>
      <button
        type="button"
        class="icon-btn collapse-btn"
        :title="collapsed ? '展开规则' : '收起规则'"
        :aria-label="`${collapsed ? '展开' : '收起'}规则 ${index + 1}`"
        :aria-expanded="!collapsed"
        :aria-controls="`rule-body-${rule.id}`"
        @click="collapsed = !collapsed"
      >
        <ChevronRight v-if="collapsed" :size="16" />
        <ChevronDown v-else :size="16" />
      </button>
      <select class="select type-select" :value="rule.type" @change="onTypeChange">
        <option v-for="t in TYPES" :key="t" :value="t">{{ ruleTypeLabel(t) }}</option>
      </select>
      <span class="seq">#{{ index + 1 }}</span>
      <span class="rule-label" :title="label">{{ label }}</span>
      <label class="check" title="启用/停用此规则">
        <input v-model="rule.enabled" type="checkbox"> 启用
      </label>
      <button type="button" class="icon-btn danger" title="删除规则" @click="$emit('remove')">
        <Trash2 :size="16" />
      </button>
    </header>

    <div v-if="error && !collapsed" class="rule-error">{{ error }}</div>

    <div v-show="!collapsed" :id="`rule-body-${rule.id}`" class="rule-body">
      <!-- 搜索替换 -->
      <template v-if="rule.type === 'replace'">
        <div class="field-row">
          <label class="field">
            <span>查找</span>
            <input v-model="rule.find" class="input" spellcheck="false" placeholder="查找内容">
          </label>
          <label class="field">
            <span>替换为</span>
            <input v-model="rule.replacement" class="input" spellcheck="false" placeholder="替换文本（留空即删除）">
          </label>
        </div>
        <div class="checks">
          <label class="check">
            <input v-model="rule.useRegex" type="checkbox"> 使用正则
          </label>
          <label class="check">
            <input v-model="rule.caseSensitive" type="checkbox"> 区分大小写
          </label>
          <label class="check">
            <input v-model="rule.applyToExtension" type="checkbox"> 同时作用于扩展名
          </label>
        </div>
      </template>
      <!-- 插入文本 -->
      <template v-if="rule.type === 'insert'">
        <div class="field-row">
          <label class="field">
            <span>插入内容</span>
            <input ref="insertInput" v-model="rule.text" class="input" spellcheck="false" placeholder="文本或标签，如 {File_BaseName}_">
          </label>
          <label class="field">
            <span>位置</span>
            <select v-model="rule.position" class="select">
              <option value="start">文件名开头</option>
              <option value="end">文件名末尾</option>
              <option value="index">指定位置</option>
            </select>
          </label>
          <label v-if="rule.position === 'index'" class="field narrow">
            <span>字符位置</span>
            <input v-model.number="rule.index" type="number" class="input" min="0">
          </label>
        </div>
        <div class="metadata-tools">
          <button type="button" class="metadata-btn" aria-haspopup="dialog" :aria-expanded="tagPickerOpen" :aria-controls="`metadata-picker-${rule.id}`" @click="openTagPicker">插入元数据标签</button>
          <span class="field-hint">标签可与普通文字混用，按每个文件的原始信息展开。</span>
        </div>
        <div class="checks">
          <label class="check">
            <input v-model="rule.beforeExtension" type="checkbox"> 末尾位置时插在扩展名之前
          </label>
        </div>
      </template>
      <!-- 序号 -->
      <template v-if="rule.type === 'sequence'">
        <div class="field-row">
          <label class="field narrow">
            <span>起始值</span>
            <input v-model.number="rule.start" type="number" class="input">
          </label>
          <label class="field narrow">
            <span>步长</span>
            <input v-model.number="rule.step" type="number" class="input">
          </label>
          <label class="field narrow">
            <span>补零位数</span>
            <input v-model.number="rule.pad" type="number" class="input" min="0">
          </label>
          <label class="field">
            <span>插入位置</span>
            <select v-model="rule.position" class="select">
              <option value="prefix">作为前缀</option>
              <option value="suffix">作为后缀</option>
              <option value="index">指定位置</option>
            </select>
          </label>
        </div>
        <div class="field-row">
          <label class="field">
            <span>序号模板</span>
            <input v-model="rule.template" class="input" spellcheck="false" placeholder="留空则纯数字；{n} 为序号占位，如 file_{n}">
          </label>
        </div>
      </template>
      <!-- 大小写 -->
      <template v-if="rule.type === 'case'">
        <div class="field-row">
          <label class="field">
            <span>转换方式</span>
            <select v-model="rule.mode" class="select">
              <option value="upper">全部大写</option>
              <option value="lower">全部小写</option>
              <option value="title">首字母大写</option>
            </select>
          </label>
        </div>
        <div class="checks">
          <label class="check">
            <input v-model="rule.applyToExtension" type="checkbox"> 同时作用于扩展名
          </label>
        </div>
      </template>
      <!-- 删除 -->
      <template v-if="rule.type === 'remove'">
        <div class="field-row">
          <label class="field">
            <span>删除方式</span>
            <select v-model="rule.mode" class="select">
              <option value="range">按位置范围</option>
              <option value="chars">按字符</option>
              <option value="extension">删除扩展名</option>
            </select>
          </label>
          <label v-if="rule.mode === 'range'" class="field narrow">
            <span>起始位置</span>
            <input v-model.number="rule.rangeStart" type="number" class="input" min="0">
          </label>
          <label v-if="rule.mode === 'range'" class="field narrow">
            <span>结束位置（不含）</span>
            <input v-model.number="rule.rangeEnd" type="number" class="input" min="0">
          </label>
          <label v-if="rule.mode === 'chars'" class="field">
            <span>要删除的字符</span>
            <input v-model="rule.chars" class="input" spellcheck="false" placeholder="要删除的字符，如 _ - .">
          </label>
        </div>
        <div class="checks">
          <label class="check">
            <input v-model="rule.applyToExtension" type="checkbox"> 同时作用于扩展名
          </label>
        </div>
      </template>
    </div>
  </article>
  <Teleport to="body">
    <dialog ref="tagDialog" :id="`metadata-picker-${rule.id}`" class="metadata-dialog" :aria-labelledby="`metadata-title-${rule.id}`" @close="tagPickerOpen = false" @pointerdown.stop>
      <h3 :id="`metadata-title-${rule.id}`">插入元数据标签</h3>
      <div class="metadata-picker">
        <label class="field">
          <span>选择标签</span>
          <select v-model="selectedTag" class="select tag-list" size="9" autofocus>
            <option v-for="tag in METADATA_TAGS" :key="tag.key" :value="tag.key">{{ tag.label }} · {{ tag.key }}</option>
          </select>
        </label>
        <div class="tag-description">
          <code>{{ '{' + selectedTag + '}' }}</code>
          <p class="field-hint">{{ tagInfo.hint }}</p>
          <p class="field-hint">可与普通文字混用，按每个文件的原始信息展开。无法读取的元数据会在预览中提示。</p>
        </div>
      </div>
      <div class="dialog-actions">
        <button type="button" class="metadata-btn" @click="tagDialog?.close()">取消</button>
        <button type="button" class="metadata-btn insert-tag-btn" @click="insertTag">插入</button>
      </div>
    </dialog>
  </Teleport>
</template>

<style scoped>
.rule-card { padding: 8px 10px; border: 1px solid var(--border); border-radius: 5px; background: var(--bg); }
.rule-head { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; cursor: pointer; }
.grip { display: inline-flex; color: var(--fg-muted); cursor: grab; padding: 6px 2px; touch-action: none; }
.grip:active { cursor: grabbing; }
.select, .input, .icon-btn { border: 1px solid var(--border); border-radius: 5px; background: var(--card); color: var(--fg); }
.select, .input { min-height: 32px; padding: 5px 8px; font-size: 13px; }
.input { font: 13px ui-monospace, SFMono-Regular, Consolas, monospace; }
.select { cursor: pointer; }
.icon-btn { display: inline-flex; align-items: center; justify-content: center; width: 28px; height: 28px; cursor: pointer; }
.icon-btn:disabled { opacity: 0.4; cursor: not-allowed; }
.collapse-btn:hover { border-color: var(--primary); color: var(--primary); }
.icon-btn.danger:hover { border-color: var(--danger); color: var(--danger); }
button:focus-visible, .select:focus-visible, .input:focus-visible { outline: 2px solid var(--primary); outline-offset: 2px; }
.type-select { width: 104px; }
.seq { color: var(--fg-muted); font-size: 12px; }
.check { display: inline-flex; gap: 5px; align-items: center; font-size: 12px; color: var(--fg-muted); cursor: pointer; }
.check input[type="checkbox"] { accent-color: var(--primary); }
.rule-label { flex: 1 1 160px; min-width: 0; font-size: 12px; color: var(--fg-muted); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.rule-error { margin: 6px 0 0; font-size: 12px; color: var(--danger); }
.rule-body { margin-top: 10px; padding-top: 10px; border-top: 1px solid var(--border); }
.field-row { display: flex; gap: 8px; flex-wrap: wrap; margin-top: 8px; align-items: flex-end; }
.field-row:first-child { margin-top: 0; }
.field { display: flex; flex-direction: column; gap: 4px; min-width: 0; flex: 1 1 150px; }
.field.narrow { flex: 0 0 auto; width: 104px; }
.field span { font-size: 12px; color: var(--fg-muted); }
.checks { display: flex; gap: 14px; flex-wrap: wrap; margin-top: 10px; align-items: center; }
.metadata-tools { display: flex; gap: 10px; align-items: center; flex-wrap: wrap; margin-top: 8px; }
.metadata-btn { min-height: 32px; padding: 5px 10px; border: 1px solid var(--border); border-radius: 5px; background: var(--card); color: var(--fg); font-size: 12px; cursor: pointer; }
.metadata-btn:hover { border-color: var(--primary); color: var(--primary); }
.field-hint { color: var(--fg-muted); font-size: 12px; }
.metadata-dialog { width: min(620px, calc(100vw - 40px)); max-height: calc(100vh - 40px); box-sizing: border-box; padding: 20px; border: 1px solid var(--border); border-radius: var(--radius); background: var(--card); color: var(--fg); }
.metadata-dialog::backdrop { background: rgb(0 0 0 / 40%); }
.metadata-dialog h3 { margin: 0 0 16px; font-size: 16px; }
.metadata-picker { display: grid; grid-template-columns: minmax(0, 1.4fr) minmax(0, 1fr); gap: 16px; }
.metadata-picker .select { width: 100%; min-width: 0; }
.tag-list { padding: 4px; }
.tag-list option { padding: 6px 8px; }
.tag-description { min-width: 0; overflow-wrap: anywhere; }
.tag-description code { color: var(--primary); font-size: 13px; }
.tag-description p { line-height: 1.6; }
.dialog-actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 16px; }
.insert-tag-btn { background: var(--primary); border-color: var(--primary); color: var(--primary-fg, white); }
.insert-tag-btn:hover { color: var(--primary-fg, white); opacity: 0.9; }
@media (max-width: 520px) {
  .metadata-picker { grid-template-columns: 1fr; }
}
</style>
