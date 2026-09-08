<script setup lang="ts">
import { ref } from 'vue'
import { INPUT_HINT } from './image-convert'
import BatchConvert from './BatchConvert.vue'
import SingleConvert from './SingleConvert.vue'

type Mode = 'batch' | 'single'
const mode = ref<Mode>('batch')
</script>

<template>
  <div class="tool">
    <h2>图片格式转换</h2>
    <p class="lead">常规与非常规格式互转，支持拖拽 / Ctrl+V 粘贴。输入可识别：{{ INPUT_HINT }}。</p>

    <div class="tabs">
      <button class="tab" :class="{ active: mode === 'batch' }" @click="mode = 'batch'">批量转换</button>
      <button class="tab" :class="{ active: mode === 'single' }" @click="mode = 'single'">单图编辑</button>
    </div>

    <BatchConvert v-if="mode === 'batch'" />
    <SingleConvert v-else />
  </div>
</template>

<style scoped>
.tool { max-width: 1000px; margin: 0 auto; }
.lead { color: var(--fg-muted); margin-bottom: 16px; }

.tabs { display: flex; gap: 4px; margin-bottom: 14px; border-bottom: 1px solid var(--border); }
.tab {
  background: none; border: none; border-bottom: 2px solid transparent;
  padding: 8px 14px; font-size: 14px; color: var(--fg-muted); cursor: pointer; margin-bottom: -1px;
}
.tab:hover { color: var(--fg); }
.tab.active { color: var(--primary); border-bottom-color: var(--primary); font-weight: 600; }
</style>
