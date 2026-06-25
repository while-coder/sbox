<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { join } from '@tauri-apps/api/path'
import {
  OUTPUT_FORMATS, extOf, convertToFile, formatBytes,
  type ConvertFileResult, type RotateMode,
} from './image-convert'

interface Picked {
  file: File
  name: string        // 去扩展名的基础文件名
  ext: string
  srcUrl: string      // 源预览（object URL）
  srcBroken: boolean  // 浏览器无法预览（如 HEIC）
}

const pic = ref<Picked | null>(null)
const error = ref('')
const busy = ref(false)
const result = ref<ConvertFileResult | null>(null)

// ── 编辑选项 ──────────────────────────────────────────────
const ROTATIONS: { value: RotateMode; label: string }[] = [
  { value: 'none', label: '不旋转' },
  { value: 'ccw90', label: '逆时针旋转 90°' },
  { value: 'cw90', label: '顺时针旋转 90°' },
  { value: 'r180', label: '旋转 180°' },
  { value: 'exif', label: '根据 EXIF 信息自动旋转' },
]
const rotate = ref<RotateMode>('exif')

const width = ref<number | null>(null)
const height = ref<number | null>(null)
const keepAspect = ref(true)

const cropEnabled = ref(false)
const cropW = ref<number | null>(null)
const cropH = ref<number | null>(null)

const target = ref('png')
const quality = ref(85)

const targetFmt = computed(() => OUTPUT_FORMATS.find(f => f.value === target.value)!)
const isLossy = computed(() => targetFmt.value?.lossy ?? false)
const canConvert = computed(() => !!pic.value && !busy.value)

// ── 输入 ──────────────────────────────────────────────────
function setFile(file: File) {
  error.value = ''
  result.value = null
  if (pic.value) URL.revokeObjectURL(pic.value.srcUrl)
  const ext = extOf(file.name)
  const name = file.name.includes('.') ? file.name.slice(0, file.name.lastIndexOf('.')) : file.name
  pic.value = { file, name, ext, srcUrl: URL.createObjectURL(file), srcBroken: false }
}

function onFilePicked(e: Event) {
  const fs = (e.target as HTMLInputElement).files
  if (fs?.length) setFile(fs[0])
  ;(e.target as HTMLInputElement).value = ''
}

function onDrop(e: DragEvent) {
  const f = e.dataTransfer?.files?.[0]
  if (f) setFile(f)
}

function onPaste(e: ClipboardEvent) {
  const img = [...(e.clipboardData?.items ?? [])]
    .filter(i => i.type.startsWith('image/'))
    .map(i => i.getAsFile())
    .find((f): f is File => !!f)
  if (img) { e.preventDefault(); setFile(img) }
}
onMounted(() => window.addEventListener('paste', onPaste))
onUnmounted(() => {
  window.removeEventListener('paste', onPaste)
  if (pic.value) URL.revokeObjectURL(pic.value.srcUrl)
})

// ── 转换 ──────────────────────────────────────────────────
function buildOptions() {
  const useCrop = cropEnabled.value && (cropW.value ?? 0) > 0 && (cropH.value ?? 0) > 0
  return {
    target: target.value,
    quality: isLossy.value ? quality.value : undefined,
    width: width.value && width.value > 0 ? width.value : undefined,
    height: height.value && height.value > 0 ? height.value : undefined,
    keepAspect: keepAspect.value,
    rotate: rotate.value,
    crop: useCrop ? { width: cropW.value!, height: cropH.value! } : undefined,
  }
}

async function convertOne() {
  if (!pic.value || busy.value) return
  busy.value = true
  error.value = ''
  result.value = null
  try {
    const dir = await open({ directory: true, title: '选择保存目录' })
    if (!dir || typeof dir !== 'string') return
    const base = pic.value.name || 'image'
    const outputPath = await join(dir, `${base}.${targetFmt.value.ext}`)
    const buf = new Uint8Array(await pic.value.file.arrayBuffer())
    result.value = await convertToFile(buf, outputPath, buildOptions())
  } catch (e: any) {
    error.value = String(e?.message || e)
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <div class="layout">
    <!-- 左：预览 -->
    <section class="card col-main">
      <div v-if="!pic" class="dropzone empty" @drop.prevent="onDrop" @dragover.prevent>
        <p class="dz-title">拖拽单张图片到此 · 或 Ctrl+V 粘贴</p>
        <label class="btn btn-outline file-label">
          选择图片
          <input type="file" class="hidden-file"
            accept="image/*,.heic,.heif,.avif,.svg,.svgz,.tga,.qoi,.dds,.exr,.hdr"
            @change="onFilePicked" />
        </label>
      </div>

      <div v-else class="preview-wrap" @drop.prevent="onDrop" @dragover.prevent>
        <div class="preview-box">
          <img v-if="!pic.srcBroken" :src="pic.srcUrl" class="preview-img" alt=""
            @error="pic.srcBroken = true" />
          <div v-else class="preview-fallback">{{ pic.ext.toUpperCase() || '?' }}<br />无法预览</div>
        </div>
        <div class="preview-meta">
          <div class="meta-name" :title="pic.file.name">{{ pic.file.name }}</div>
          <div class="meta-size">{{ formatBytes(pic.file.size) }}</div>
          <label class="btn btn-outline file-label sm">
            更换图片
            <input type="file" class="hidden-file"
              accept="image/*,.heic,.heif,.avif,.svg,.svgz,.tga,.qoi,.dds,.exr,.hdr"
              @change="onFilePicked" />
          </label>
        </div>
      </div>
    </section>

    <!-- 右：编辑选项 -->
    <section class="card col-side">
      <fieldset class="group">
        <legend>旋转</legend>
        <label v-for="r in ROTATIONS" :key="r.value" class="radio">
          <input v-model="rotate" type="radio" :value="r.value" /> {{ r.label }}
        </label>
      </fieldset>

      <div class="field">
        <span class="label">改尺寸（像素，留空=原尺寸）</span>
        <div class="row">
          <input v-model.number="width" type="number" min="1" placeholder="宽" class="input size-input" />
          <span class="times">×</span>
          <input v-model.number="height" type="number" min="1" placeholder="高" class="input size-input" />
        </div>
        <label class="check">
          <input v-model="keepAspect" type="checkbox" /> 保持纵横比
        </label>
      </div>

      <div class="field">
        <label class="check">
          <input v-model="cropEnabled" type="checkbox" /> 裁剪（居中）
        </label>
        <div v-if="cropEnabled" class="row">
          <input v-model.number="cropW" type="number" min="1" placeholder="宽" class="input size-input" />
          <span class="times">×</span>
          <input v-model.number="cropH" type="number" min="1" placeholder="高" class="input size-input" />
        </div>
      </div>

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

      <div class="actions">
        <button class="btn" :disabled="!canConvert" @click="convertOne">
          {{ busy ? '转换中…' : '选择目录并转换' }}
        </button>
        <p v-if="result" class="result">
          已保存 · {{ result.width }}×{{ result.height }} · {{ formatBytes(result.bytes) }}
        </p>
      </div>
    </section>
  </div>

  <p v-if="error" class="error">{{ error }}</p>
</template>

<style scoped>
.layout { display: grid; grid-template-columns: 1fr 280px; gap: 12px; align-items: start; }

.card {
  background: var(--card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 16px;
}

.dropzone { display: flex; flex-direction: column; gap: 12px; align-items: center; }
.dropzone.empty { justify-content: center; min-height: 320px; border: 1px dashed var(--border); border-radius: 6px; }
.dz-title { margin: 0; color: var(--fg-muted); font-size: 13px; text-align: center; }
.file-label { display: inline-flex; align-items: center; cursor: pointer; }
.file-label.sm { font-size: 12px; padding: 4px 10px; }
.hidden-file { display: none; }

.preview-wrap { display: flex; flex-direction: column; gap: 12px; }
.preview-box {
  min-height: 320px; display: flex; align-items: center; justify-content: center;
  background:
    repeating-conic-gradient(#e9e9e9 0% 25%, #fff 0% 50%) 0 / 20px 20px;
  border: 1px solid var(--border); border-radius: 6px; overflow: hidden;
}
.preview-img { max-width: 100%; max-height: 420px; object-fit: contain; }
.preview-fallback { color: var(--fg-muted); font-size: 13px; font-weight: 600; text-align: center; }
.preview-meta { display: flex; align-items: center; gap: 12px; }
.meta-name { flex: 1; min-width: 0; font-size: 13px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.meta-size { font-size: 12px; color: var(--fg-muted); flex-shrink: 0; }

.col-side { display: flex; flex-direction: column; gap: 16px; }
.field { display: flex; flex-direction: column; gap: 6px; }
.label { font-size: 12px; font-weight: 600; color: var(--fg-muted); }
.size-input { width: 80px; }
.times { color: var(--fg-muted); }
.check { font-size: 12px; color: var(--fg-muted); display: flex; align-items: center; gap: 6px; cursor: pointer; }

.group { border: 1px solid var(--border); border-radius: 6px; padding: 8px 12px; display: flex; flex-direction: column; gap: 6px; margin: 0; }
.group legend { font-size: 12px; font-weight: 600; color: var(--fg-muted); padding: 0 4px; }
.radio { font-size: 13px; display: flex; align-items: center; gap: 6px; cursor: pointer; }

.actions { display: flex; flex-direction: column; gap: 8px; margin-top: 4px; }
.result { font-size: 12px; color: var(--primary); margin: 0; }

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

.error { color: var(--danger); margin: 12px 0 0; font-size: 13px; }

@media (max-width: 760px) {
  .layout { grid-template-columns: 1fr; }
}
</style>
