<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted, nextTick } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { PhysicalPosition, PhysicalSize } from '@tauri-apps/api/dpi'
import { writeImage } from '@tauri-apps/plugin-clipboard-manager'
import { save } from '@tauri-apps/plugin-dialog'
import { Image } from '@tauri-apps/api/image'
import jsQR from 'jsqr'
import {
  cropSelectionPixels,
  getLatestCapture,
  getLatestCapturePixels,
  finishScreenshot,
  CAPTURE_READY,
  saveSelection,
  type Capture,
  type CapturePixels,
  type SelectionRect,
} from './screenshot'

const capture = ref<Capture | null>(null)
const shotCanvas = ref<HTMLCanvasElement | null>(null)
const active = computed(() => !!capture.value)

// 选区（视口 CSS 像素）
const sel = reactive({ x: 0, y: 0, w: 0, h: 0 })
type Phase = 'idle' | 'creating' | 'moving' | 'resizing'
const phase = ref<Phase>('idle')
const hasSel = computed(() => sel.w > 3 && sel.h > 3)
// 选区已确定（非操作中）：显示手柄与工具条
const settled = computed(() => phase.value === 'idle' && hasSel.value)
const toast = ref('')

const HANDLES = ['nw', 'n', 'ne', 'e', 'se', 's', 'sw', 'w'] as const
type Handle = typeof HANDLES[number]
let activeHandle: Handle | 'body' | '' = ''
let drag = { mx: 0, my: 0, x: 0, y: 0, w: 0, h: 0 }
let restoreMain = false
let unlisten: UnlistenFn | null = null

function resetSel() {
  sel.x = 0; sel.y = 0; sel.w = 0; sel.h = 0
  phase.value = 'idle'
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
  if (e.key === 'Escape') cancel()
  else if (e.key === 'Enter' && hasSel.value) void doSave()
}

function clamp(v: number, lo: number, hi: number) { return Math.max(lo, Math.min(hi, v)) }

// 在空白处按下 → 新建选区
function onBgMouseDown(e: MouseEvent) {
  if (e.button !== 0) return
  phase.value = 'creating'
  drag = { mx: e.clientX, my: e.clientY, x: e.clientX, y: e.clientY, w: 0, h: 0 }
  sel.x = e.clientX; sel.y = e.clientY; sel.w = 0; sel.h = 0
}

// 在选区内按下 → 移动
function onBodyMouseDown(e: MouseEvent) {
  if (e.button !== 0) return
  e.stopPropagation()
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
  if (phase.value === 'creating' && !hasSel.value) sel.w = 0
  phase.value = 'idle'
  activeHandle = ''
}

// ── 选区 ──────────────────────────────────────────────────
function selectionRect(): SelectionRect | null {
  const cap = capture.value
  if (!cap || !hasSel.value) return null
  const rx = cap.width / window.innerWidth
  const ry = cap.height / window.innerHeight
  const x = Math.round(sel.x * rx)
  const y = Math.round(sel.y * ry)
  const width = Math.max(1, Math.round(sel.w * rx))
  const height = Math.max(1, Math.round(sel.h * ry))
  return { x, y, width, height }
}

async function doSave() {
  const selection = selectionRect()
  if (!selection) return
  try {
    const path = await save({ defaultPath: 'screenshot.png' })
    if (!path) return
    await saveSelection(path, selection)
    await close()
  } catch (e: any) {
    toast.value = `保存失败：${String(e?.message || e)}`
  }
}

async function doCopy() {
  const selection = selectionRect()
  if (!selection) return
  try {
    const pixels = toBytes(await cropSelectionPixels(selection))
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
  const below = sel.y + sel.h + 44 < window.innerHeight
  const top = below ? sel.y + sel.h + 8 : Math.max(8, sel.y - 44)
  const left = clamp(sel.x + sel.w - 268, 8, window.innerWidth - 276)
  return { top: `${top}px`, left: `${left}px` }
})
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
      :style="selStyle"
      @mousedown="onBodyMouseDown"
      @dblclick.stop="doCopy"
      title="拖动可移动，双击复制"
    >
      <!-- 缩放手柄 -->
      <template v-if="settled">
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
    <div v-if="capture && settled" class="toolbar" :style="toolbarStyle" @mousedown.stop>
      <button class="tb" @click="doDecode" title="识别选区中的二维码">识别二维码</button>
      <button class="tb" @click="doCopy" title="复制到剪贴板（双击选区）">复制</button>
      <button class="tb primary" @click="doSave" title="保存为 PNG（Enter）">保存</button>
      <button class="tb ghost" @click="cancel" title="取消（Esc）">取消</button>
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
  display: flex; gap: 4px;
  background: rgba(30, 30, 30, 0.95);
  padding: 5px; border-radius: 6px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
}
.tb {
  border: none; background: #3a3a3a; color: #eee;
  padding: 5px 12px; border-radius: 4px; cursor: pointer; font-size: 13px;
}
.tb:hover { background: #4a4a4a; }
.tb.primary { background: #2d6cdf; color: #fff; }
.tb.primary:hover { background: #1f56b8; }
.tb.ghost { background: transparent; color: #bbb; }
.tb.ghost:hover { color: #fff; }

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
