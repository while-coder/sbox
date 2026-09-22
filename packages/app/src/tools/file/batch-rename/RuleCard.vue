<script setup lang="ts">
import { computed } from 'vue'
import { GripVertical, Trash2 } from 'lucide-vue-next'
import { ruleError, ruleLabel, ruleTypeLabel, type RenameRule, type RuleType } from './rename-engine'

const props = defineProps<{
  rule: RenameRule
  index: number
}>()

const emit = defineEmits<{
  update: [patch: Partial<RenameRule>]
  changeType: [type: RuleType]
  remove: []
}>()

const TYPES: RuleType[] = ['replace', 'insert', 'sequence', 'case', 'remove']

const error = computed(() => ruleError(props.rule))
const label = computed(() => ruleLabel(props.rule))

function onTypeChange(event: Event) {
  emit('changeType', (event.target as HTMLSelectElement).value as RuleType)
}
</script>

<template>
  <article class="rule-card">
    <header class="rule-head">
      <span class="grip" title="拖动排序"><GripVertical :size="14" /></span>
      <select class="select type-select" :value="rule.type" @change="onTypeChange">
        <option v-for="t in TYPES" :key="t" :value="t">{{ ruleTypeLabel(t) }}</option>
      </select>
      <span class="seq">#{{ index + 1 }}</span>
      <span class="spacer" />
      <label class="check" title="启用/停用此规则">
        <input v-model="rule.enabled" type="checkbox"> 启用
      </label>
      <button type="button" class="icon-btn danger" title="删除规则" @click="$emit('remove')">
        <Trash2 :size="16" />
      </button>
    </header>

    <p class="rule-label">{{ label }}</p>

    <div v-if="error" class="rule-error">{{ error }}</div>

    <div class="rule-body">
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
            <input v-model="rule.text" class="input" spellcheck="false" placeholder="要插入的文本">
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
</template>

<style scoped>
.rule-card { padding: 14px; border: 1px solid var(--border); border-radius: var(--radius); background: var(--bg); }
.rule-head { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
.grip { display: inline-flex; color: var(--fg-muted); cursor: grab; }
.select, .input, .icon-btn { border: 1px solid var(--border); border-radius: 5px; background: var(--card); color: var(--fg); }
.select, .input { min-height: 32px; padding: 5px 8px; font-size: 13px; }
.input { font: 13px ui-monospace, SFMono-Regular, Consolas, monospace; }
.select { cursor: pointer; }
.icon-btn { display: inline-flex; align-items: center; justify-content: center; width: 28px; height: 28px; cursor: pointer; }
.icon-btn:disabled { opacity: 0.4; cursor: not-allowed; }
.icon-btn.danger:hover { border-color: var(--danger); color: var(--danger); }
button:focus-visible, .select:focus-visible, .input:focus-visible { outline: 2px solid var(--primary); outline-offset: 2px; }
.type-select { width: 104px; }
.seq { color: var(--fg-muted); font-size: 12px; }
.spacer { flex: 1 1 auto; }
.check { display: inline-flex; gap: 5px; align-items: center; font-size: 12px; color: var(--fg-muted); cursor: pointer; }
.check input[type="checkbox"] { accent-color: var(--primary); }
.rule-label { margin: 8px 0 0; font-size: 12px; color: var(--fg-muted); overflow-wrap: anywhere; }
.rule-error { margin: 6px 0 0; font-size: 12px; color: var(--danger); }
.rule-body { margin-top: 10px; padding-top: 10px; border-top: 1px solid var(--border); }
.field-row { display: flex; gap: 8px; flex-wrap: wrap; margin-top: 8px; align-items: flex-end; }
.field-row:first-child { margin-top: 0; }
.field { display: flex; flex-direction: column; gap: 4px; min-width: 0; flex: 1 1 150px; }
.field.narrow { flex: 0 0 auto; width: 104px; }
.field span { font-size: 12px; color: var(--fg-muted); }
.checks { display: flex; gap: 14px; flex-wrap: wrap; margin-top: 10px; align-items: center; }
</style>
