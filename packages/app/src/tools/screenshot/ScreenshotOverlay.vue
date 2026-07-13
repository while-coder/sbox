<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted, nextTick } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { PhysicalPosition, PhysicalSize } from '@tauri-apps/api/dpi'
import { writeImage } from '@tauri-apps/plugin-clipboard-manager'
import { save } from '@tauri-apps/plugin-dialog'
import { Image } from '@tauri-apps/api/image'
import jsQR from 'jsqr'
import { Check, Download, ScanQrCode, Square, Undo2, X } from 'lucide-vue-next'
import {
  cropSelectionPixels,
  getLatestCapture,
  getLatestCapturePixels,
  finishScreenshot,
  CAPTURE_READY,
  saveSelection,
  type Capture,
  type CapturePixels,
  type RectMark,
  type SelectionRect,
} from './screenshot'

const capture = ref<Capture | null>(null)
const shotCanvas = ref<HTMLCanvasElement | null>(null)
const active = computed(() => !!capture.value)

// 选区（视口 CSS 像素）
const sel = reactive({ x: 0, y: 0, w: 0, h: 0 })
type Phase = 'idle' | 'creating' | 'moving' | 'resizing' | 'marking'
const phase = ref<Phase>('idle')
const hasSel = computed(() => sel.w > 3 && sel.h > 3)
// 选区已确定（非操作中）：显示手柄与工具条
const settled = computed(() => phase.value === 'idle' && hasSel.value)
const toolbarVisible = computed(() => hasSel.value && (phase.value === 'idle' || phase.value === 'marking'))
const toast = ref('')

type Tool = 'select' | 'rect'
interface UiRectMark {
  id: number
  x: number
  y: number
  w: number
  h: number
  color: string
  lineWidth: number
}

const MARK_COLORS = [
  { name: '红色', css: '#ef4444', rgba: [239, 68, 68, 255] },
  { name: '黄色', css: '#facc15', rgba: [250, 204, 21, 255] },
  { name: '绿色', css: '#22c55e', rgba: [34, 197, 94, 255] },
  { name: '蓝色', css: '#3b82f6', rgba: [59, 130, 246, 255] },
  { name: '黑色', css: '#111827', rgba: [17, 24, 39, 255] },
  { name: '白色', css: '#ffffff', rgba: [255, 255, 255, 255] },
] as const
const MARK_WIDTHS = [2, 4, 6] as const
const tool = ref<Tool>('select')
const markColor = ref<string>(MARK_COLORS[0].css)
const markLineWidth = ref<number>(MARK_WIDTHS[1])
const marks = ref<UiRectMark[]>([])
const draftMark = ref<UiRectMark | null>(null)
let nextMarkId = 1
let markStart = { x: 0, y: 0 }

const HANDLES = ['nw', 'n', 'ne', 'e', 'se', 's', 'sw', 'w'] as const
type Handle = typeof HANDLES[number]
let activeHandle: Handle | 'body' | '' = ''
let drag = { mx: 0, my: 0, x: 0, y: 0, w: 0, h: 0 }
let restoreMain = false
let unlisten: UnlistenFn | null = null

function resetSel() {
  sel.x = 0; sel.y = 0; sel.w = 0; sel.h = 0
  phase.value = 'idle'
  tool.value = 'select'
  marks.value = []
  draftMark.value = null
  toast.value = ''
}

function toClampedBytes(payload: CapturePixels): Uint8ClampedArray {
  if (payload instanceof ArrayBuffer) return new Uint8ClampedArray(payload)
  if (payload instanceof Uint8Array) return new Uint8ClampedArray(payload)
  return Uint8ClampedArray.from(payload)
}

function toBytes(payload: CapturePixels): Uint8Array {
  if (payload instanceof ArrayBuffer) return new Uint8Array(payload)
  if (payload instanceof Uint8Array) return payload
  return Uint8Array.from(payload)
}

function drawCapture(cap: Capture, payload: CapturePixels) {
  const canvas = shotCanvas.value
  if (!canvas) return
  const pixels = toClampedBytes(payload)
  const expected = cap.previewWidth * cap.previewHeight * 4
  if (pixels.byteLength !== expected) {
    throw new Error(`截图像素尺寸不匹配：${pixels.byteLength} != ${expected}`)
  }
  canvas.width = cap.previewWidth
  canvas.height = cap.previewHeight
  const ctx = canvas.getContext('2d', { alpha: false })
  if (!ctx) throw new Error('无法创建截图画布')
  ctx.putImageData(new ImageData(pixels, cap.previewWidth, cap.previewHeight), 0, 0)
}

onMounted(async () => {
  document.documentElement.classList.add('screenshot-overlay-html')
  document.body.classList.add('screenshot-overlay-body')
  window.addEventListener('keydown', onKey)
  // 截图就绪：刷新画面、重置选区、接管鼠标事件并聚焦自身
  unlisten = await listen<{ restoreMain: boolean }>(CAPTURE_READY, async (e) => {
    const t0 = performance.now()
    restoreMain = e.payload?.restoreMain ?? false
    const cap = await getLatestCapture()
    const metaAt = performance.now()
    if (!cap) {
      capture.value = null
      resetSel()
      await nextTick()
      await finishScreenshot(restoreMain)
      return
    }
    capture.value = cap
    resetSel()
    await nextTick()
    const w = getCurrentWindow()
    // 把覆盖层精确定位/铺满“被截的那块屏”（物理像素），多屏才不会错位。
    // 定位失败也要保证 show，否则覆盖层永不显示（表现为按快捷键后毫无反应）。
    try {
      await w.setPosition(new PhysicalPosition(cap.x, cap.y))
      await w.setSize(new PhysicalSize(cap.width, cap.height))
    } catch (err) {
      console.error('覆盖层定位失败：', err)
    }
    try {
      drawCapture(cap, await getLatestCapturePixels())
    } catch (err) {
      toast.value = `截图加载失败：${String((err as Error)?.message || err)}`
      console.error('截图画面加载失败：', err)
    }
    const drawAt = performance.now()
    if (!(await w.isVisible())) await w.show()
    await w.setIgnoreCursorEvents(false)
    await w.setFocus()
    console.debug('[screenshot] overlay', {
      metaMs: Math.round(metaAt - t0),
      drawMs: Math.round(drawAt - metaAt),
      totalMs: Math.round(performance.now() - t0),
      preview: `${cap.previewWidth}x${cap.previewHeight}`,
      source: `${cap.width}x${cap.height}`,
    })
  })
})
onUnmounted(() => {
  document.documentElement.classList.remove('screenshot-overlay-html')
  document.body.classList.remove('screenshot-overlay-body')
  window.removeEventListener('keydown', onKey)
  if (unlisten) unlisten()
})

function onKey(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'z' && marks.value.length) {
    e.preventDefault()
    undoMark()
  } else if (e.key === 'Escape') cancel()
  else if (e.key === 'Enter' && hasSel.value) void doSave()
}

function clamp(v: number, lo: number, hi: number) { return Math.max(lo, Math.min(hi, v)) }

// 在空白处按下 → 新建选区
function onBgMouseDown(e: MouseEvent) {
  if (e.button !== 0) return
  if (tool.value === 'rect') return
  marks.value = []
  draftMark.value = null
  phase.value = 'creating'
  drag = { mx: e.clientX, my: e.clientY, x: e.clientX, y: e.clientY, w: 0, h: 0 }
  sel.x = e.clientX; sel.y = e.clientY; sel.w = 0; sel.h = 0
}

// 在选区内按下 → 移动
function onBodyMouseDown(e: MouseEvent) {
  if (e.button !== 0) return
  e.stopPropagation()
  if (tool.value === 'rect') {
    const x = clamp(e.clientX - sel.x, 0, sel.w)
    const y = clamp(e.clientY - sel.y, 0, sel.h)
    markStart = { x, y }
    draftMark.value = {
      id: nextMarkId,
      x,
      y,
      w: 0,
      h: 0,
      color: markColor.value,
      lineWidth: markLineWidth.value,
    }
    phase.value = 'marking'
    return
  }
  phase.value = 'moving'
  activeHandle = 'body'
  drag = { mx: e.clientX, my: e.clientY, x: sel.x, y: sel.y, w: sel.w, h: sel.h }
}

// 在手柄按下 → 缩放
function onHandleMouseDown(h: Handle, e: MouseEvent) {
  if (e.button !== 0) return
  e.stopPropagation()
  phase.value = 'resizing'
  activeHandle = h
  drag = { mx: e.clientX, my: e.clientY, x: sel.x, y: sel.y, w: sel.w, h: sel.h }
}

function onMouseMove(e: MouseEvent) {
  if (phase.value === 'idle') return
  if (phase.value === 'marking') {
    const mark = draftMark.value
    if (!mark) return
    const x = clamp(e.clientX - sel.x, 0, sel.w)
    const y = clamp(e.clientY - sel.y, 0, sel.h)
    mark.x = Math.min(markStart.x, x)
    mark.y = Math.min(markStart.y, y)
    mark.w = Math.abs(x - markStart.x)
    mark.h = Math.abs(y - markStart.y)
    return
  }
  const dx = e.clientX - drag.mx
  const dy = e.clientY - drag.my
  const W = window.innerWidth, H = window.innerHeight

  if (phase.value === 'creating') {
    sel.x = Math.min(drag.mx, e.clientX)
    sel.y = Math.min(drag.my, e.clientY)
    sel.w = Math.abs(e.clientX - drag.mx)
    sel.h = Math.abs(e.clientY - drag.my)
  } else if (phase.value === 'moving') {
    sel.x = clamp(drag.x + dx, 0, W - drag.w)
    sel.y = clamp(drag.y + dy, 0, H - drag.h)
  } else if (phase.value === 'resizing') {
    const MIN = 10
    let { x, y, w, h } = drag
    const hd = activeHandle as Handle
    if (hd.includes('e')) w = drag.w + dx
    if (hd.includes('s')) h = drag.h + dy
    if (hd.includes('w')) { x = drag.x + dx; w = drag.w - dx }
    if (hd.includes('n')) { y = drag.y + dy; h = drag.h - dy }
    if (w < MIN) { if (hd.includes('w')) x = drag.x + drag.w - MIN; w = MIN }
    if (h < MIN) { if (hd.includes('n')) y = drag.y + drag.h - MIN; h = MIN }
    sel.x = clamp(x, 0, W - 1); sel.y = clamp(y, 0, H - 1)
    sel.w = Math.min(w, W - sel.x); sel.h = Math.min(h, H - sel.y)
  }
}

function onMouseUp() {
  if (phase.value === 'marking') {
    const mark = draftMark.value
    if (mark && mark.w > 3 && mark.h > 3) {
      marks.value.push({ ...mark })
      nextMarkId += 1
    }
    draftMark.value = null
  }
  if (phase.value === 'creating' && !hasSel.value) sel.w = 0
  phase.value = 'idle'
  activeHandle = ''
}

function toggleRectTool() {
  tool.value = tool.value === 'rect' ? 'select' : 'rect'
  draftMark.value = null
  phase.value = 'idle'
}

function undoMark() {
  marks.value.pop()
}

// ── 选区 ──────────────────────────────────────────────────
function selectionRect(): SelectionRect | null {
  const cap = capture.value
  if (!cap || !hasSel.value) return null
  const rx = cap.width / window.innerWidth
  const ry = cap.height / window.innerHeight
  const x = clamp(Math.round(sel.x * rx), 0, cap.width - 1)
  const y = clamp(Math.round(sel.y * ry), 0, cap.height - 1)
  const width = Math.max(1, Math.min(cap.width - x, Math.round(sel.w * rx)))
  const height = Math.max(1, Math.min(cap.height - y, Math.round(sel.h * ry)))
  return { x, y, width, height }
}

function outputMarks(selection: SelectionRect): RectMark[] {
  const rx = selection.width / sel.w
  const ry = selection.height / sel.h
  const lineScale = (rx + ry) / 2
  return marks.value.map((mark) => {
    const color = MARK_COLORS.find((item) => item.css === mark.color) ?? MARK_COLORS[0]
    const x = clamp(Math.round(mark.x * rx), 0, selection.width - 1)
    const y = clamp(Math.round(mark.y * ry), 0, selection.height - 1)
    return {
      x,
      y,
      width: Math.max(1, Math.min(selection.width - x, Math.round(mark.w * rx))),
      height: Math.max(1, Math.min(selection.height - y, Math.round(mark.h * ry))),
      lineWidth: Math.max(1, Math.round(mark.lineWidth * lineScale)),
      color: [...color.rgba],
    }
  })
}

async function doSave() {
  const selection = selectionRect()
  if (!selection) return
  try {
    const path = await save({ defaultPath: 'screenshot.png' })
    if (!path) return
    await saveSelection(path, selection, outputMarks(selection))
    await close()
  } catch (e: any) {
    toast.value = `保存失败：${String(e?.message || e)}`
  }
}

async function doCopy() {
  const selection = selectionRect()
  if (!selection) return
  try {
    const pixels = toBytes(await cropSelectionPixels(selection, outputMarks(selection)))
    const image = await Image.new(pixels, selection.width, selection.height)
    await writeImage(image)
    await close()
  } catch (e: any) {
    toast.value = `复制失败：${String(e?.message || e)}`
  }
}

async function doDecode() {
  const selection = selectionRect()
  if (!selection) return
  try {
    const pixels = toClampedBytes(await cropSelectionPixels(selection))
    const res = jsQR(pixels, selection.width, selection.height)
    if (res?.data) {
      try { await navigator.clipboard.writeText(res.data) } catch { /* ignore */ }
      toast.value = `识别成功（已复制）：${res.data.slice(0, 80)}`
      setTimeout(() => close(), 1200)
    } else {
      toast.value = '所选区域未识别到二维码'
    }
  } catch (e: any) {
    toast.value = `识别失败：${String(e?.message || e)}`
  }
}

/** 结束：先清空画面并绘制空白，再让覆盖层进入透明穿透状态。 */
async function close() {
  capture.value = null
  await nextTick()
  await new Promise<void>((r) => requestAnimationFrame(() => r()))
  await finishScreenshot(restoreMain)
}

function cancel() { void close() }

// ── 样式计算 ──────────────────────────────────────────────
const selStyle = computed(() => ({
  left: `${sel.x}px`, top: `${sel.y}px`, width: `${sel.w}px`, height: `${sel.h}px`,
}))
const sizeStyle = computed(() => ({
  top: `${Math.max(2, sel.y - 22)}px`, left: `${sel.x}px`,
}))
const toolbarStyle = computed(() => {
  const toolbarHeight = tool.value === 'rect' ? 78 : 38
  const below = sel.y + sel.h + toolbarHeight + 8 < window.innerHeight
  const top = below ? sel.y + sel.h + 8 : Math.max(8, sel.y - toolbarHeight - 8)
  const desiredWidth = tool.value === 'rect' ? 267 : 216
  const toolbarWidth = Math.min(desiredWidth, window.innerWidth - 16)
  const left = clamp(sel.x + sel.w - toolbarWidth, 8, Math.max(8, window.innerWidth - toolbarWidth - 8))
  return { top: `${top}px`, left: `${left}px` }
})
function markStyle(mark: UiRectMark) {
  return {
    left: `${mark.x}px`,
    top: `${mark.y}px`,
    width: `${mark.w}px`,
    height: `${mark.h}px`,
    borderColor: mark.color,
    borderWidth: `${mark.lineWidth}px`,
  }
}
function handleStyle(h: Handle) {
  const C = 'calc(50% - 5px)'
  const map: Record<Handle, Record<string, string>> = {
    nw: { left: '-5px', top: '-5px' }, n: { left: C, top: '-5px' }, ne: { right: '-5px', top: '-5px' },
    e: { right: '-5px', top: C }, se: { right: '-5px', bottom: '-5px' }, s: { left: C, bottom: '-5px' },
    sw: { left: '-5px', bottom: '-5px' }, w: { left: '-5px', top: C },
  }
  return map[h]
}
const handleCursor: Record<Handle, string> = {
  nw: 'nwse-resize', n: 'ns-resize', ne: 'nesw-resize', e: 'ew-resize',
  se: 'nwse-resize', s: 'ns-resize', sw: 'nesw-resize', w: 'ew-resize',
}
</script>

<template>
  <div
    class="overlay"
    :class="{ active }"
    @mousedown="onBgMouseDown"
    @mousemove="onMouseMove"
    @mouseup="onMouseUp"
  >
    <canvas v-if="capture" ref="shotCanvas" class="shot" />

    <!-- 未选区时整屏压暗 -->
    <div v-if="capture && !hasSel" class="dim-full" />

    <!-- 选区 -->
    <div
      v-if="capture && hasSel"
      class="sel"
      :class="{ marking: tool === 'rect' }"
      :style="selStyle"
      @mousedown="onBodyMouseDown"
      @dblclick.stop="tool === 'select' && doCopy()"
      title="拖动可移动，双击复制"
    >
      <div class="marks-layer">
        <span
          v-for="mark in marks"
          :key="mark.id"
          class="rect-mark"
          :style="markStyle(mark)"
        />
        <span v-if="draftMark" class="rect-mark" :style="markStyle(draftMark)" />
      </div>
      <!-- 缩放手柄 -->
      <template v-if="settled && tool === 'select'">
        <span
          v-for="h in HANDLES"
          :key="h"
          class="handle"
          :style="{ ...handleStyle(h), cursor: handleCursor[h] }"
          @mousedown="onHandleMouseDown(h, $event)"
        />
      </template>
    </div>
    <div v-if="capture && hasSel" class="size-tag" :style="sizeStyle">{{ Math.round(sel.w) }} × {{ Math.round(sel.h) }}</div>

    <!-- 工具条 -->
    <div v-if="capture && toolbarVisible" class="toolbar" :style="toolbarStyle" @mousedown.stop>
      <div class="toolbar-row function-row">
        <button
          class="tb icon-tool"
          :class="{ selected: tool === 'rect' }"
          title="框选标记"
          aria-label="框选标记"
          @click="toggleRectTool"
        ><Square :size="18" /></button>
        <span class="separator" />
        <button class="tb icon-tool" title="识别选区中的二维码" aria-label="识别二维码" @click="doDecode"><ScanQrCode :size="18" /></button>
        <button class="tb icon-tool" :disabled="!marks.length" title="撤销标记（Ctrl+Z）" aria-label="撤销标记" @click="undoMark"><Undo2 :size="18" /></button>
        <span class="separator" />
        <button class="tb icon-tool primary" title="保存为 PNG（Enter）" aria-label="保存" @click="doSave"><Download :size="18" /></button>
        <button class="tb icon-tool danger" title="取消（Esc）" aria-label="取消" @click="cancel"><X :size="18" /></button>
        <button class="tb icon-tool confirm" title="复制到剪贴板（双击选区）" aria-label="复制" @click="doCopy"><Check :size="18" /></button>
      </div>

      <div v-if="tool === 'rect'" class="toolbar-row option-row">
        <button
          v-for="width in MARK_WIDTHS"
          :key="width"
          class="width-option"
          :class="{ selected: markLineWidth === width }"
          :title="`${width} 像素线宽`"
          :aria-label="`${width} 像素线宽`"
          @click="markLineWidth = width"
        ><span :style="{ height: `${width}px` }" /></button>
        <span class="separator" />
        <button
          v-for="color in MARK_COLORS"
          :key="color.css"
          class="swatch"
          :class="{ selected: markColor === color.css }"
          :style="{ backgroundColor: color.css }"
          :title="color.name"
          :aria-label="color.name"
          @click="markColor = color.css"
        />
      </div>
    </div>

    <div v-if="capture && !hasSel" class="hint">拖拽选择区域 · 选区可拖动/缩放 · 双击复制 · Esc 取消</div>
    <div v-if="capture && toast" class="toast" @mousedown.stop>{{ toast }}</div>
  </div>
</template>

<style scoped>
:global(.screenshot-overlay-html),
:global(.screenshot-overlay-body),
:global(.screenshot-overlay-body #app) {
  background: transparent;
}

.overlay {
  position: fixed; inset: 0; width: 100vw; height: 100vh;
  overflow: hidden; cursor: default; user-select: none;
  pointer-events: none;
}
.overlay.active {
  cursor: crosshair;
  pointer-events: auto;
}
.shot { position: absolute; inset: 0; width: 100vw; height: 100vh; object-fit: fill; }
.dim-full { position: absolute; inset: 0; background: rgba(0, 0, 0, 0.45); }

.sel {
  position: absolute;
  border: 1px solid #2d6cdf;
  box-shadow: 0 0 0 100vmax rgba(0, 0, 0, 0.45);
  cursor: move;
}
.sel.marking { cursor: crosshair; }
.marks-layer { position: absolute; inset: 0; overflow: hidden; pointer-events: none; }
.rect-mark {
  position: absolute; display: block; border-style: solid;
  background: transparent; pointer-events: none;
}
.handle {
  position: absolute; width: 10px; height: 10px;
  background: #fff; border: 1px solid #2d6cdf; border-radius: 50%;
}

.size-tag {
  position: absolute;
  background: rgba(0, 0, 0, 0.75); color: #fff;
  font: 12px ui-monospace, Consolas, monospace;
  padding: 1px 6px; border-radius: 3px; pointer-events: none;
}

.toolbar {
  position: absolute;
  z-index: 4; display: flex; flex-direction: column; align-items: flex-start; gap: 6px;
  max-width: calc(100vw - 16px);
}
.toolbar-row {
  display: flex; align-items: center; gap: 4px;
  min-width: max-content; padding: 5px; border-radius: 6px;
  background: rgba(30, 30, 30, 0.95);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
}
.function-row { width: max-content; }
.option-row { height: 34px; }
.tb {
  flex: none;
  border: none; background: #3a3a3a; color: #eee;
  padding: 5px 12px; border-radius: 4px; cursor: pointer; font-size: 13px;
}
.tb:hover { background: #4a4a4a; }
.tb:disabled { opacity: 0.4; cursor: default; }
.tb.icon-tool {
  width: 28px; height: 28px; padding: 0;
  display: grid; place-items: center; line-height: 1;
}
.tb.selected { background: #2563eb; color: #fff; }
.separator { flex: none; width: 1px; height: 20px; margin: 0 2px; background: #555; }
.swatch, .width-option {
  flex: none; display: grid; place-items: center;
  width: 24px; height: 24px; padding: 0; border: 2px solid transparent;
  border-radius: 4px; cursor: pointer;
}
.swatch { box-shadow: inset 0 0 0 1px rgba(0, 0, 0, 0.28); }
.swatch.selected, .width-option.selected { border-color: #fff; }
.width-option { background: #3a3a3a; }
.width-option > span { display: block; width: 14px; border-radius: 2px; background: #eee; }
.tb.primary { background: #2d6cdf; color: #fff; }
.tb.primary:hover { background: #1f56b8; }
.tb.danger { background: transparent; color: #fb7185; }
.tb.danger:hover { background: #4b2028; color: #fff; }
.tb.confirm { background: transparent; color: #4ade80; }
.tb.confirm:hover { background: #174a2b; color: #fff; }

.hint {
  position: absolute; top: 16px; left: 50%; transform: translateX(-50%);
  background: rgba(0, 0, 0, 0.6); color: #fff;
  padding: 6px 14px; border-radius: 16px; font-size: 13px; pointer-events: none;
}
.toast {
  position: absolute; bottom: 32px; left: 50%; transform: translateX(-50%);
  max-width: 80vw; background: rgba(0, 0, 0, 0.85); color: #fff;
  padding: 8px 16px; border-radius: 6px; font-size: 13px; word-break: break-all;
}
</style>
