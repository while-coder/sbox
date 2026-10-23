<script setup lang="ts">
import { computed, nextTick, ref } from 'vue'
import { ChevronRight, Trash2 } from 'lucide-vue-next'
import SButton from '@qingfeng346/ui-kit/components/button/SButton.vue'
import SIconButton from '@qingfeng346/ui-kit/components/button/SIconButton.vue'
import SCheckbox from '@qingfeng346/ui-kit/components/checkbox/SCheckbox.vue'
import SInput from '@qingfeng346/ui-kit/components/input/SInput.vue'
import SSelect from '@qingfeng346/ui-kit/components/SSelect.vue'
import SModal from '@qingfeng346/ui-kit/components/SModal.vue'
import SCollapse from '@qingfeng346/ui-kit/components/collapse/SCollapse.vue'
import SCollapseItem from '@qingfeng346/ui-kit/components/collapse/SCollapseItem.vue'
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

const insertField = ref<HTMLElement | null>(null)
const tagPickerOpen = ref(false)
let insertStart = 0
let insertEnd = 0
const selectedTag = ref<string>('File_BaseName')
const tagInfo = computed(() => METADATA_TAGS.find(tag => tag.key === selectedTag.value)!)
const error = computed(() => ruleError(props.rule))
const label = computed(() => ruleLabel(props.rule))

async function insertTag() {
  if (props.rule.type !== 'insert') return
  const input = insertField.value?.querySelector('input')
  const start = insertStart
  const end = insertEnd
  const token = `{${selectedTag.value}}`
  emit('update', { text: props.rule.text.slice(0, start) + token + props.rule.text.slice(end) })
  if (selectedTag.value === 'Clipboard') emit('readClipboard')
  tagPickerOpen.value = false
  await nextTick()
  input?.focus()
  input?.setSelectionRange(start + token.length, start + token.length)
}

function openTagPicker() {
  if (props.rule.type !== 'insert') return
  const input = insertField.value?.querySelector('input')
  insertStart = input?.selectionStart ?? props.rule.text.length
  insertEnd = input?.selectionEnd ?? insertStart
  tagPickerOpen.value = true
}
</script>

<template>
  <SCollapse class="rule-card" :default-expanded-names="[rule.id]">
    <SCollapseItem :name="rule.id">
      <template #header>
        <span class="rule-head">
        <ChevronRight class="collapse-arrow" :size="16" />
        <SSelect class="type-select" :value="rule.type" :options="TYPES.map(t => ({ value: t, label: ruleTypeLabel(t) }))" size="small" @click.stop @update:value="emit('changeType', $event)" />
        <span class="seq">#{{ index + 1 }}</span>
        <span class="rule-label" :title="label">{{ label }}</span>
        <SCheckbox v-model:checked="rule.enabled" title="启用/停用此规则" @click.stop>启用</SCheckbox>
        <SIconButton type="error" :size="28" title="删除规则" aria-label="删除规则" @click.stop="$emit('remove')">
          <Trash2 :size="16" />
        </SIconButton>
        </span>
      </template>

    <div v-if="error" class="rule-error">{{ error }}</div>

    <div class="rule-body">
      <!-- 搜索替换 -->
      <template v-if="rule.type === 'replace'">
        <div class="field-row">
          <label class="field">
            <span>查找</span>
            <SInput v-model:value="rule.find" spellcheck="false" placeholder="查找内容" />
          </label>
          <label class="field">
            <span>替换为</span>
            <SInput v-model:value="rule.replacement" spellcheck="false" placeholder="替换文本（留空即删除）" />
          </label>
        </div>
        <div class="checks">
          <SCheckbox v-model:checked="rule.useRegex">使用正则</SCheckbox>
          <SCheckbox v-model:checked="rule.caseSensitive">区分大小写</SCheckbox>
          <SCheckbox v-model:checked="rule.applyToExtension">同时作用于扩展名</SCheckbox>
        </div>
      </template>
      <!-- 插入文本 -->
      <template v-if="rule.type === 'insert'">
        <div class="field-row">
          <label ref="insertField" class="field">
            <span>插入内容</span>
            <SInput v-model:value="rule.text" spellcheck="false" placeholder="文本或标签，如 {File_BaseName}_" />
          </label>
          <label class="field">
            <span>位置</span>
            <SSelect v-model:value="rule.position" :options="[{ value: 'start', label: '文件名开头' }, { value: 'end', label: '文件名末尾' }, { value: 'index', label: '指定位置' }]" />
          </label>
          <label v-if="rule.position === 'index'" class="field narrow">
            <span>字符位置</span>
            <SInput v-model:value="rule.index" type="number" min="0" />
          </label>
        </div>
        <div class="metadata-tools">
          <SButton size="small" aria-haspopup="dialog" :aria-expanded="tagPickerOpen" :aria-controls="`metadata-picker-${rule.id}`" @click="openTagPicker">插入元数据标签</SButton>
          <span class="field-hint">标签可与普通文字混用，按每个文件的原始信息展开。</span>
        </div>
        <div class="checks">
          <SCheckbox v-model:checked="rule.beforeExtension">末尾位置时插在扩展名之前</SCheckbox>
        </div>
      </template>
      <!-- 序号 -->
      <template v-if="rule.type === 'sequence'">
        <div class="field-row">
          <label class="field narrow">
            <span>起始值</span>
            <SInput v-model:value="rule.start" type="number" />
          </label>
          <label class="field narrow">
            <span>步长</span>
            <SInput v-model:value="rule.step" type="number" />
          </label>
          <label class="field narrow">
            <span>补零位数</span>
            <SInput v-model:value="rule.pad" type="number" min="0" />
          </label>
          <label class="field">
            <span>插入位置</span>
            <SSelect v-model:value="rule.position" :options="[{ value: 'prefix', label: '作为前缀' }, { value: 'suffix', label: '作为后缀' }, { value: 'index', label: '指定位置' }]" />
          </label>
        </div>
        <div class="field-row">
          <label class="field">
            <span>序号模板</span>
            <SInput v-model:value="rule.template" spellcheck="false" placeholder="留空则纯数字；{n} 为序号占位，如 file_{n}" />
          </label>
        </div>
      </template>
      <!-- 大小写 -->
      <template v-if="rule.type === 'case'">
        <div class="field-row">
          <label class="field">
            <span>转换方式</span>
            <SSelect v-model:value="rule.mode" :options="[{ value: 'upper', label: '全部大写' }, { value: 'lower', label: '全部小写' }, { value: 'title', label: '首字母大写' }]" />
          </label>
        </div>
        <div class="checks">
          <SCheckbox v-model:checked="rule.applyToExtension">同时作用于扩展名</SCheckbox>
        </div>
      </template>
      <!-- 删除 -->
      <template v-if="rule.type === 'remove'">
        <div class="field-row">
          <label class="field">
            <span>删除方式</span>
            <SSelect v-model:value="rule.mode" :options="[{ value: 'range', label: '按位置范围' }, { value: 'chars', label: '按字符' }, { value: 'extension', label: '删除扩展名' }]" />
          </label>
          <label v-if="rule.mode === 'range'" class="field narrow">
            <span>起始位置</span>
            <SInput v-model:value="rule.rangeStart" type="number" min="0" />
          </label>
          <label v-if="rule.mode === 'range'" class="field narrow">
            <span>结束位置（不含）</span>
            <SInput v-model:value="rule.rangeEnd" type="number" min="0" />
          </label>
          <label v-if="rule.mode === 'chars'" class="field">
            <span>要删除的字符</span>
            <SInput v-model:value="rule.chars" spellcheck="false" placeholder="要删除的字符，如 _ - ." />
          </label>
        </div>
        <div class="checks">
          <SCheckbox v-model:checked="rule.applyToExtension">同时作用于扩展名</SCheckbox>
        </div>
      </template>
    </div>
    </SCollapseItem>
  </SCollapse>
  <SModal v-model:show="tagPickerOpen" :id="`metadata-picker-${rule.id}`" title="插入元数据标签" :width="620" :mask-closable="false">
      <div class="metadata-picker">
        <label class="field">
          <span>选择标签</span>
          <select v-model="selectedTag" class="s-select tag-list" size="9" autofocus>
            <option v-for="tag in METADATA_TAGS" :key="tag.key" :value="tag.key">{{ tag.label }} · {{ tag.key }}</option>
          </select>
        </label>
        <div class="tag-description">
          <code>{{ '{' + selectedTag + '}' }}</code>
          <p class="field-hint">{{ tagInfo.hint }}</p>
          <p class="field-hint">可与普通文字混用，按每个文件的原始信息展开。无法读取的元数据会在预览中提示。</p>
        </div>
      </div>
      <template #footer>
        <SButton size="small" @click="tagPickerOpen = false">取消</SButton>
        <SButton type="primary" size="small" @click="insertTag">插入</SButton>
      </template>
  </SModal>
</template>

<style scoped>
.rule-card { background: var(--bg); }
.rule-card :deep(.s-collapse-summary) { padding: 8px 10px 8px 36px; }
.rule-card :deep(.s-collapse-summary > span:first-child) { display: block; width: 100%; min-width: 0; }
.rule-head { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
.collapse-arrow { flex: 0 0 auto; }
.rule-card :deep(details[open]) .collapse-arrow { transform: rotate(90deg); }
.type-select { width: 104px; }
.seq { color: var(--fg-muted); font-size: 12px; }
.rule-label { flex: 1 1 160px; min-width: 0; font-size: 12px; color: var(--fg-muted); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.rule-error { margin: 6px 0 0; font-size: 12px; color: var(--danger); }
.rule-body { margin-top: 8px; }
.field-row { display: flex; gap: 8px; flex-wrap: wrap; margin-top: 8px; align-items: flex-end; }
.field-row:first-child { margin-top: 0; }
.field { display: flex; flex-direction: column; gap: 4px; min-width: 0; flex: 1 1 150px; }
.field.narrow { flex: 0 0 auto; width: 104px; }
.field span { font-size: 12px; color: var(--fg-muted); }
.checks { display: flex; gap: 14px; flex-wrap: wrap; margin-top: 10px; align-items: center; }
.metadata-tools { display: flex; gap: 10px; align-items: center; flex-wrap: wrap; margin-top: 8px; }
.field-hint { color: var(--fg-muted); font-size: 12px; }
.metadata-picker { display: grid; grid-template-columns: minmax(0, 1.4fr) minmax(0, 1fr); gap: 16px; }
.metadata-picker .s-select { width: 100%; min-width: 0; }
.tag-list { padding: 4px; }
.tag-list option { padding: 6px 8px; }
.tag-description { min-width: 0; overflow-wrap: anywhere; }
.tag-description code { color: var(--primary); font-size: 13px; }
.tag-description p { line-height: 1.6; }
@media (max-width: 520px) {
  .metadata-picker { grid-template-columns: 1fr; }
}
</style>
