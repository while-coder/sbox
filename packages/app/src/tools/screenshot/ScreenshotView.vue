<script setup lang="ts">
import { ref } from 'vue'
import { startScreenshot } from './screenshot'
import { settings } from '../../settings'

const error = ref('')

async function shoot() {
  error.value = ''
  try {
    await startScreenshot()
  } catch (e: any) {
    error.value = `截图失败：${String(e?.message || e)}`
  }
}
</script>

<template>
  <div class="tool">
    <h2>截图</h2>
    <p class="lead">捕获主显示器后在全屏覆盖层上框选区域，可保存、复制到剪贴板或识别其中的二维码。</p>

    <section class="card">
      <button class="btn big" @click="shoot">开始截图</button>
      <p class="tip">点击后窗口会自动隐藏并冻结屏幕，拖拽鼠标框选区域；Esc 取消，Enter 保存。</p>
      <p class="tip" v-if="settings.screenshotEnabled && settings.screenshotKey">
        全局快捷键：<kbd>{{ settings.screenshotKey }}</kbd>（可在设置中修改）
      </p>
      <p class="tip muted">注：当前版本捕获主显示器；多显示器框选将在后续完善。</p>
    </section>

    <p v-if="error" class="error">{{ error }}</p>
  </div>
</template>

<style scoped>
.tool { max-width: 720px; margin: 0 auto; }
.lead { color: var(--fg-muted); margin-bottom: 16px; }
.card {
  background: var(--card); border: 1px solid var(--border);
  border-radius: var(--radius); padding: 24px; text-align: center;
}
.btn.big {
  background: var(--primary); color: #fff; border: none;
  padding: 12px 32px; border-radius: var(--radius); cursor: pointer;
  font-size: 15px; font-weight: 600;
}
.btn.big:hover { background: var(--primary-hover); }
.tip { color: var(--fg-muted); font-size: 13px; margin: 12px 0 0; }
.tip.muted { opacity: 0.75; font-size: 12px; }
kbd {
  background: var(--bg); border: 1px solid var(--border); border-radius: 4px;
  padding: 1px 6px; font: 12px ui-monospace, Consolas, monospace;
}
.error { color: var(--danger); margin: 12px 0 0; font-size: 13px; }
</style>
