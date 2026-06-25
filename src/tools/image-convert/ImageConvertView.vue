<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { join } from '@tauri-apps/api/path'
import {
  OUTPUT_FORMATS, INPUT_HINT, extOf, convertToFile, formatBytes,
  type ConvertFileResult,
} from './image-convert'

interface Item {
  id: number
  file: File
  name: string          // 去扩展名的基础文件名
  ext: string
  srcUrl: string        // 源预览（object URL）
  srcBroken: boolean    // 浏览器无法预览（如 HEIC）
  status: 'pending' | 'busy' | 'done' | 'error'
  error: string
  result: ConvertFileResult | null
}

const items = ref<Item[]>([])
const error = ref('')
let seq = 0

// ── 转换选项 ──────────────────────────────────────────────
const target = ref('png')
const quality = ref(85)
const width = ref<number | null>(null)
const height = ref<number | null>(null)
const keepAspect = ref(true)

const targetFmt = computed(() => OUTPUT_FORMATS.find(f => f.value === target.value)!)
const isLossy = computed(() => targetFmt.value?.lossy ?? false)
const isBusy = computed(() => items.value.some(i => i.status === 'busy'))
const canConvert = computed(() => items.value.length > 0 && !isBusy.value)

// ── 输入 ──────────────────────────────────────────────────
function addFiles(files: FileList | File[]) {
  error.value = ''
  for (const file of Array.from(files)) {
    const ext = extOf(file.name)
    const name = file.name.includes('.') ? file.name.slice(0, file.name.lastIndexOf('.')) : file.name
    items.value.push({
      id: ++seq, file, name, ext,
      srcUrl: URL.createObjectURL(file),
      srcBroken: false,
      status: 'pending', error: '', result: null,
    })
  }
}

function onFilePicked(e: Event) {
  const fs = (e.target as HTMLInputElement).files
  if (fs?.length) addFiles(fs)
  ;(e.target as HTMLInputElement).value = ''
}

function onDrop(e: DragEvent) {
  const fs = e.dataTransfer?.files
  if (fs?.length) addFiles(fs)
}

function onPaste(e: ClipboardEvent) {
  const imgs = [...(e.clipboardData?.items ?? [])]
    .filter(i => i.type.startsWith('image/'))
    .map(i => i.getAsFile())
    .filter((f): f is File => !!f)
  if (imgs.length) { e.preventDefault(); addFiles(imgs) }
}
onMounted(() => window.addEventListener('paste', onPaste))
onUnmounted(() => {
  window.removeEventListener('paste', onPaste)
  items.value.forEach(i => URL.revokeObjectURL(i.srcUrl))
})

function removeItem(id: number) {
  const idx = items.value.findIndex(i => i.id === id)
  if (idx >= 0) {
    URL.revokeObjectURL(items.value[idx].srcUrl)
    items.value.splice(idx, 1)
  }
}

function clearAll() {
  items.value.forEach(i => URL.revokeObjectURL(i.srcUrl))
  items.value = []
  error.value = ''
}

// ── 转换 ──────────────────────────────────────────────────
function buildOptions() {
  return {
    target: target.value,
    quality: isLossy.value ? quality.value : undefined,
    width: width.value && width.value > 0 ? width.value : undefined,
    height: height.value && height.value > 0 ? height.value : undefined,
    keepAspect: keepAspect.value,
  }
}

async function convertItem(item: Item, outputPath: string) {
  item.status = 'busy'
  item.error = ''
  item.result = null
  try {
    const buf = new Uint8Array(await item.file.arrayBuffer())
    item.result = await convertToFile(buf, outputPath, buildOptions())
    item.status = 'done'
  } catch (e: any) {
    item.status = 'error'
    item.error = String(e?.message || e)
  }
}

function buildOutputNames(): Map<number, string> {
  const used = new Set<string>()
  const names = new Map<number, string>()
  const ext = targetFmt.value.ext
  for (const item of items.value) {
    const base = item.name || 'image'
    let outputName = `${base}.${ext}`
    let suffix = 2
    while (used.has(outputName.toLowerCase())) {
      outputName = `${base}-${suffix}.${ext}`
      suffix += 1
    }
    used.add(outputName.toLowerCase())
    names.set(item.id, outputName)
  }
  return names
}

async function convertAll() {
  if (!items.value.length || isBusy.value) return
  error.value = ''
  try {
    const dir = await open({ directory: true, title: '选择保存目录' })
    if (!dir || typeof dir !== 'string') return
    const outputNames = buildOutputNames()
    for (const item of items.value) {
      const outputName = outputNames.get(item.id) ?? `${item.name || 'image'}.${targetFmt.value.ext}`
      const outputPath = await join(dir, outputName)
      await convertItem(item, outputPath)
    }
  } catch (e: any) { error.value = String(e?.message || e) }
}

function statusText(item: Item): string {
  switch (item.status) {
    case 'busy': return '转换中…'
    case 'done': return item.result
      ? `已保存 · ${item.result.width}×${item.result.height} · ${formatBytes(item.result.bytes)}`
      : '已保存'
    case 'error': return item.error || '失败'
    default: return '待转换'
  }
}
</script>

<template>
  <div class="tool">
    <h2>图片格式转换</h2>
    <p class="lead">常规与非常规格式互转，支持拖拽 / Ctrl+V 粘贴 / 多选批量。输入可识别：{{ INPUT_HINT }}。</p>

    <div class="layout">
      <!-- 左：输入与列表 -->
      <section class="card col-main">
        <div class="dropzone" @drop.prevent="onDrop" @dragover.prevent>
          <p class="dz-title">拖拽图片到此 · 或 Ctrl+V 粘贴</p>
          <div class="row dz-actions">
            <label class="btn btn-outline file-label">
              选择图片
              <input type="file" multiple class="hidden-file"
                accept="image/*,.heic,.heif,.avif,.svg,.svgz,.tga,.qoi,.dds,.exr,.hdr"
                @change="onFilePicked" />
            </label>
            <button class="btn btn-outline" :disabled="!items.length" @click="clearAll">清空列表</button>
          </div>
        </div>

        <ul v-if="items.length" class="items">
          <li v-for="item in items" :key="item.id" class="item" :class="item.status">
            <img v-if="!item.srcBroken" :src="item.srcUrl" class="thumb" alt=""
              @error="item.srcBroken = true" />
            <div v-else class="thumb thumb-fallback">{{ item.ext.toUpperCase() || '?' }}</div>

            <div class="item-body">
              <div class="item-name" :title="item.file.name">{{ item.file.name }}</div>
              <div class="item-status" :class="item.status" :title="item.result?.path || item.error">
                {{ statusText(item) }}
              </div>
            </div>

            <div class="item-actions">
              <button class="link-btn" @click="removeItem(item.id)">移除</button>
            </div>
          </li>
        </ul>
        <p v-else class="placeholder">列表为空，先添加图片</p>
      </section>

      <!-- 右：选项 -->
      <section class="card col-side">
        <label class="field">
          <span class="label">目标格式</span>
          <select v-model="target" class="input">
            <option v-for="f in OUTPUT_FORMATS" :key="f.value" :value="f.value">{{ f.label }}</option>
          </select>
        </label>

        <label v-if="isLossy" class="field">
          <span class="label">质量 {{ quality }}</span>
          <input v-model.number="quality" type="range" min="1" max="100" step="1" />
        </label>

        <div class="field">
          <span class="label">缩放（像素，留空=原尺寸）</span>
          <div class="row">
            <input v-model.number="width" type="number" min="1" placeholder="宽" class="input size-input" />
            <span class="times">×</span>
            <input v-model.number="height" type="number" min="1" placeholder="高" class="input size-input" />
          </div>
          <label class="check">
            <input v-model="keepAspect" type="checkbox" /> 保持纵横比
          </label>
        </div>

        <div class="actions">
          <button class="btn" :disabled="!canConvert" @click="convertAll">
            {{ isBusy ? '转换中…' : '选择目录并转换' }}
          </button>
        </div>
      </section>
    </div>

    <p v-if="error" class="error">{{ error }}</p>
  </div>
</template>

<style scoped>
.tool { max-width: 1000px; margin: 0 auto; }
.lead { color: var(--fg-muted); margin-bottom: 16px; }

.layout { display: grid; grid-template-columns: 1fr 280px; gap: 12px; align-items: start; }

.card {
  background: var(--card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 16px;
}

.dropzone { display: flex; flex-direction: column; gap: 12px; }
.dz-title { margin: 0; color: var(--fg-muted); font-size: 13px; text-align: center; padding: 12px; border: 1px dashed var(--border); border-radius: 6px; }
.dz-actions { gap: 8px; }
.file-label { display: inline-flex; align-items: center; cursor: pointer; }
.hidden-file { display: none; }

.items { list-style: none; margin: 14px 0 0; padding: 0; display: flex; flex-direction: column; gap: 8px; }
.item { display: flex; align-items: center; gap: 10px; padding: 8px; border: 1px solid var(--border); border-radius: 6px; }
.item.error { border-color: var(--danger); }
.thumb { width: 44px; height: 44px; object-fit: cover; border-radius: 4px; background: #fff; border: 1px solid var(--border); flex-shrink: 0; }
.thumb-fallback { display: flex; align-items: center; justify-content: center; font-size: 11px; font-weight: 600; color: var(--fg-muted); background: var(--bg); }
.item-body { flex: 1; min-width: 0; }
.item-name { font-size: 13px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.item-status { font-size: 12px; color: var(--fg-muted); margin-top: 2px; }
.item-status.done { color: var(--primary); }
.item-status.error { color: var(--danger); }
.item-actions { display: flex; gap: 4px; flex-shrink: 0; }

.placeholder { color: var(--fg-muted); font-size: 13px; margin: 14px 0 0; }

.col-side { display: flex; flex-direction: column; gap: 16px; }
.field { display: flex; flex-direction: column; gap: 6px; }
.label { font-size: 12px; font-weight: 600; color: var(--fg-muted); }
.size-input { width: 80px; }
.times { color: var(--fg-muted); }
.check { font-size: 12px; color: var(--fg-muted); display: flex; align-items: center; gap: 6px; cursor: pointer; }

.actions { display: flex; flex-direction: column; gap: 8px; margin-top: 4px; }

.row { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }

.input {
  padding: 6px 10px; border: 1px solid var(--border); border-radius: 4px;
  background: var(--bg); color: var(--fg);
  font: 13px ui-monospace, SFMono-Regular, Consolas, monospace;
}
.input:focus { outline: none; border-color: var(--primary); }

.btn {
  background: var(--primary); color: #fff; border: none;
  padding: 6px 14px; border-radius: 4px; cursor: pointer; font-size: 13px; font-weight: 500;
}
.btn:hover:not(:disabled) { background: var(--primary-hover); }
.btn:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-outline { background: transparent; color: var(--fg); border: 1px solid var(--border); }
.btn-outline:hover:not(:disabled) { border-color: var(--primary); color: var(--primary); }

.link-btn { background: none; border: none; padding: 4px 8px; font-size: 12px; color: var(--fg-muted); cursor: pointer; }
.link-btn:hover:not(:disabled) { color: var(--primary); }

.error { color: var(--danger); margin: 12px 0 0; font-size: 13px; }

@media (max-width: 760px) {
  .layout { grid-template-columns: 1fr; }
}
</style>
