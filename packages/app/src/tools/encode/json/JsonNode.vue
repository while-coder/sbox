<script setup lang="ts">
import { ref, computed } from 'vue'

const props = defineProps<{
  /** 键名；根节点为 null */
  k: string | number | null
  value: unknown
  path: string
  depth: number
}>()

// 深层默认折叠，避免大对象一次铺满
const collapsed = ref(props.depth >= 3)
const copiedWhat = ref<'value' | 'path' | ''>('')

const type = computed(() => {
  const v = props.value
  if (v === null) return 'null'
  if (Array.isArray(v)) return 'array'
  return typeof v
})
const isBranch = computed(() => type.value === 'object' || type.value === 'array')

const entries = computed<[string | number, unknown][]>(() => {
  if (type.value === 'array') return (props.value as unknown[]).map((v, i) => [i, v])
  if (type.value === 'object') return Object.entries(props.value as object)
  return []
})

const count = computed(() => entries.value.length)
const bracket = computed(() => (type.value === 'array' ? ['[', ']'] : ['{', '}']))

// 长字符串默认截断为单行，点击「展开」查看完整内容
const MAX_STR = 160
const rawString = computed(() => (type.value === 'string' ? (props.value as string) : ''))
const stringTooLong = computed(() => rawString.value.length > MAX_STR)
const expandedStr = ref(false)

const displayValue = computed(() => {
  if (type.value === 'string') {
    if (stringTooLong.value && !expandedStr.value) {
      return JSON.stringify(rawString.value.slice(0, MAX_STR)) + `… (+${rawString.value.length - MAX_STR} 字符)`
    }
    return JSON.stringify(rawString.value)
  }
  return String(props.value)
})

// 字符串本身是 JSON（对象/数组）时，可内联解析成子树
const subJson = computed<unknown>(() => {
  if (type.value !== 'string') return null
  const s = rawString.value.trim()
  if (s.length > 200_000) return null
  if (!(s.startsWith('{') && s.endsWith('}')) && !(s.startsWith('[') && s.endsWith(']'))) return null
  try { return JSON.parse(s) } catch { return null }
})
const showSubJson = ref(false)

function childPath(key: string | number): string {
  if (typeof key === 'number') return `${props.path}[${key}]`
  return /^[A-Za-z_$][\w$]*$/.test(key) ? `${props.path}.${key}` : `${props.path}[${JSON.stringify(key)}]`
}

async function copy(what: 'value' | 'path') {
  // 叶子字符串复制原始文本（不带引号）；对象/数组/其它仍复制 JSON
  const valueText = typeof props.value === 'string'
    ? props.value
    : JSON.stringify(props.value, null, 2)
  const text = what === 'path' ? props.path : valueText
  try {
    await navigator.clipboard.writeText(text)
    copiedWhat.value = what
    setTimeout(() => { if (copiedWhat.value === what) copiedWhat.value = '' }, 1500)
  } catch { /* ignore */ }
}
</script>

<template>
  <div class="node">
    <div class="line" :class="{ branch: isBranch }">
      <span v-if="isBranch" class="toggle" @click="collapsed = !collapsed">{{ collapsed ? '▶' : '▼' }}</span>
      <span v-else class="toggle spacer" />

      <span v-if="k !== null" class="key">{{ k }}</span>
      <span v-if="k !== null" class="colon">:</span>

      <template v-if="isBranch">
        <span class="brace" @click="collapsed = !collapsed">
          {{ bracket[0] }}<span v-if="collapsed" class="muted"> … {{ count }} </span><template v-if="collapsed">{{ bracket[1] }}</template>
        </span>
        <span v-if="!collapsed" class="muted count">{{ count }} 项</span>
      </template>
      <span v-else-if="type === 'string'" class="val string" :class="{ expanded: expandedStr }">{{ displayValue }}</span>
      <span v-else class="val" :class="type">{{ displayValue }}</span>

      <template v-if="type === 'string'">
        <button v-if="stringTooLong" class="mini inline" @click="expandedStr = !expandedStr">
          {{ expandedStr ? '收起' : `展开 (共 ${rawString.length} 字符)` }}
        </button>
        <button v-if="subJson !== null" class="mini inline" @click="showSubJson = !showSubJson">
          {{ showSubJson ? '收起解析' : '解析为 JSON' }}
        </button>
      </template>

      <span class="actions">
        <button class="mini" @click="copy('value')" title="复制该节点的值（JSON）">{{ copiedWhat === 'value' ? '✓' : '复制值' }}</button>
        <button class="mini" @click="copy('path')" title="复制访问路径">{{ copiedWhat === 'path' ? '✓' : '路径' }}</button>
      </span>
    </div>

    <div v-if="isBranch && !collapsed" class="children">
      <JsonNode
        v-for="[ck, cv] in entries"
        :key="String(ck)"
        :k="ck"
        :value="cv"
        :path="childPath(ck)"
        :depth="depth + 1"
      />
      <div class="line closing"><span class="toggle spacer" /><span class="brace">{{ bracket[1] }}</span></div>
    </div>

    <!-- 字符串值本身是 JSON 时，内联展示解析后的子树 -->
    <div v-if="!isBranch && showSubJson && subJson !== null" class="children sub-json">
      <JsonNode :k="null" :value="subJson" :path="path" :depth="0" />
    </div>
  </div>
</template>

<style scoped>
.node { font: 13px/1.7 ui-monospace, SFMono-Regular, Consolas, monospace; }
.line { display: flex; align-items: center; gap: 4px; border-radius: 4px; padding: 0 4px; }
.line:hover { background: var(--bg); }
.line:hover .actions { opacity: 1; }
.closing:hover { background: none; }

.toggle { width: 14px; flex: 0 0 14px; cursor: pointer; color: var(--fg-muted); font-size: 10px; text-align: center; user-select: none; }
.toggle.spacer { cursor: default; }

.key { color: var(--primary); }
.colon { color: var(--fg-muted); }
.brace { cursor: pointer; color: var(--fg-muted); }
.muted { color: var(--fg-muted); }
.count { font-size: 11px; margin-left: 4px; }

.val { min-width: 0; }
.val.string { color: var(--success); }
.val.string:not(.expanded) { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; flex: 0 1 auto; }
.val.string.expanded { white-space: pre-wrap; overflow-wrap: anywhere; flex: 1 1 auto; }
.val.number { color: #b8860b; }
.val.boolean { color: #8250df; }
.val.null { color: var(--fg-muted); font-style: italic; }

.children { padding-left: 16px; border-left: 1px solid var(--border); margin-left: 6px; }
.children.sub-json { margin-top: 2px; background: var(--bg); border-radius: 4px; padding: 6px 8px 6px 16px; }

.actions { opacity: 0; display: inline-flex; gap: 4px; margin-left: 8px; flex: 0 0 auto; transition: opacity 0.1s; }
.mini {
  background: none; border: 1px solid var(--border); border-radius: 3px;
  font-size: 10px; padding: 0 6px; line-height: 16px; color: var(--fg-muted); cursor: pointer;
  white-space: nowrap; flex: 0 0 auto;
}
.mini.inline { flex: 0 0 auto; opacity: 0.8; }
.mini:hover { border-color: var(--primary); color: var(--primary); }
</style>
