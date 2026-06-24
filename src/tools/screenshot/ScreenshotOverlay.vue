<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted, nextTick } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { PhysicalPosition, PhysicalSize } from '@tauri-apps/api/dpi'
import { writeImage } from '@tauri-apps/plugin-clipboard-manager'
import { Image } from '@tauri-apps/api/image'
import jsQR from 'jsqr'
import { getLatestCapture, finishScreenshot, CAPTURE_READY, type Capture } from './screenshot'
import { saveBase64File } from '../../save'

const capture = ref<Capture | null>(null)
const dataUrl = computed(() => capture.value ? `data:image/png;base64,${capture.value.base64}` : '')
const imgEl = ref<HTMLImageElement | null>(null)

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

onMounted(async () => {
  window.addEventListener('keydown', onKey)
  // 截图就绪：刷新画面、重置选区、显示并聚焦自身
  unlisten = await listen<{ restoreMain: boolean }>(CAPTURE_READY, async (e) => {
    restoreMain = e.payload?.restoreMain ?? false
    const cap = await getLatestCapture()
    // 先把新图解码完成再显示，避免 show 后仍在解码而露出空白/上一帧
    if (cap) {
      try {
        const pre = document.createElement('img')
        pre.src = `data:image/png;base64,${cap.base64}`
        await pre.decode()
      } catch { /* 解码失败也继续 */ }
    }
    capture.value = cap
    resetSel()
    await nextTick()
    const w = getCurrentWindow()
    // 把覆盖层精确定位/铺满“被截的那块屏”（物理像素），多屏才不会错位
    if (cap) {
      await w.setPosition(new PhysicalPosition(cap.x, cap.y))
      await w.setSize(new PhysicalSize(cap.width, cap.height))
    }
    await w.show()
    await w.setFocus()
  })
})
onUnmounted(() => {
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

// ── 裁剪 ──────────────────────────────────────────────────
function cropCanvas(): HTMLCanvasElement | null {
  const img = imgEl.value
  const cap = capture.value
  if (!img || !cap || !hasSel.value) return null
  const rx = cap.width / window.innerWidth
  const ry = cap.height / window.innerHeight
  const sx = Math.round(sel.x * rx), sy = Math.round(sel.y * ry)
  const sw = Math.round(sel.w * rx), sh = Math.round(sel.h * ry)
  const canvas = document.createElement('canvas')
  canvas.width = sw; canvas.height = sh
  const ctx = canvas.getContext('2d', { willReadFrequently: true })
  if (!ctx) return null
  ctx.drawImage(img, sx, sy, sw, sh, 0, 0, sw, sh)
  return canvas
}

function b64ToBytes(b64: string): Uint8Array {
  const bin = atob(b64)
  const bytes = new Uint8Array(bin.length)
  for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i)
  return bytes
}

async function doSave() {
  const canvas = cropCanvas()
  if (!canvas) return
  try {
    const b64 = canvas.toDataURL('image/png').split(',', 2)[1]
    const ok = await saveBase64File(b64, 'screenshot.png')
    if (ok) await close()
  } catch (e: any) {
    toast.value = `保存失败：${String(e?.message || e)}`
  }
}

async function doCopy() {
  const canvas = cropCanvas()
  if (!canvas) return
  try {
    const b64 = canvas.toDataURL('image/png').split(',', 2)[1]
    const image = await Image.fromBytes(b64ToBytes(b64))
    await writeImage(image)
    await close()
  } catch (e: any) {
    toast.value = `复制失败：${String(e?.message || e)}`
  }
}

async function doDecode() {
  const canvas = cropCanvas()
  if (!canvas) return
  const ctx = canvas.getContext('2d', { willReadFrequently: true })
  if (!ctx) return
  const data = ctx.getImageData(0, 0, canvas.width, canvas.height)
  const res = jsQR(data.data, data.width, data.height)
  if (res?.data) {
    try { await navigator.clipboard.writeText(res.data) } catch { /* ignore */ }
    toast.value = `识别成功（已复制）：${res.data.slice(0, 80)}`
    setTimeout(() => close(), 1200)
  } else {
    toast.value = '所选区域未识别到二维码'
  }
}

/** 结束：先清空画面并绘制空白（避免复用窗口残留本次截图），再隐藏覆盖层。 */
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
  <div class="overlay" @mousedown="onBgMouseDown" @mousemove="onMouseMove" @mouseup="onMouseUp">
    <img v-if="dataUrl" ref="imgEl" :src="dataUrl" class="shot" alt="" draggable="false" />

    <!-- 未选区时整屏压暗 -->
    <div v-if="!hasSel" class="dim-full" />

    <!-- 选区 -->
    <div
      v-if="hasSel"
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
    <div v-if="hasSel" class="size-tag" :style="sizeStyle">{{ Math.round(sel.w) }} × {{ Math.round(sel.h) }}</div>

    <!-- 工具条 -->
    <div v-if="settled" class="toolbar" :style="toolbarStyle" @mousedown.stop>
      <button class="tb" @click="doDecode" title="识别选区中的二维码">识别二维码</button>
      <button class="tb" @click="doCopy" title="复制到剪贴板（双击选区）">复制</button>
      <button class="tb primary" @click="doSave" title="保存为 PNG（Enter）">保存</button>
      <button class="tb ghost" @click="cancel" title="取消（Esc）">取消</button>
    </div>

    <div v-if="!hasSel" class="hint">拖拽选择区域 · 选区可拖动/缩放 · 双击复制 · Esc 取消</div>
    <div v-if="toast" class="toast" @mousedown.stop>{{ toast }}</div>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed; inset: 0; width: 100vw; height: 100vh;
  overflow: hidden; cursor: crosshair; user-select: none;
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
