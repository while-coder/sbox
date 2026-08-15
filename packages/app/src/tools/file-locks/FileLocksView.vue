<script setup lang="ts">
import { computed, ref } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { checkFileLocks, type FileLocksCheckResult } from './tauri'

const path = ref('')
const checking = ref(false)
const error = ref('')
const result = ref<FileLocksCheckResult | null>(null)
const canCheck = computed(() => path.value.trim().length > 0 && !checking.value)

async function choosePath(directory: boolean) {
  error.value = ''
  const selection = await open({
    title: directory ? '选择要检查的文件夹' : '选择要检查的文件',
    directory,
    multiple: false,
  })
  if (typeof selection === 'string') {
    path.value = selection
    result.value = null
  }
}

async function check() {
  if (!canCheck.value) return
  checking.value = true
  error.value = ''
  result.value = null
  try {
    result.value = await checkFileLocks(path.value.trim())
    path.value = result.value.path
  } catch (e: any) {
    error.value = String(e?.message || e)
  } finally {
    checking.value = false
  }
}

async function copy(value: string) {
  try {
    await navigator.clipboard.writeText(value)
  } catch (e: any) {
    error.value = `复制失败: ${String(e?.message || e)}`
  }
}
</script>

<template>
  <div class="file-locks">
    <h2>文件占用检查</h2>
    <p class="lead">检查阻止删除、重命名或移动的 Windows 进程。只查询信息，不会结束进程或改动文件。</p>

    <section class="card">
      <label for="file-locks-path">文件或文件夹路径</label>
      <div class="path-row">
        <input
          id="file-locks-path"
          v-model="path"
          class="input"
          autocomplete="off"
          spellcheck="false"
          placeholder="粘贴路径，或从右侧选择"
          @keydown.enter="check"
        >
        <button type="button" class="btn btn-outline" :disabled="checking" @click="choosePath(false)">选择文件</button>
        <button type="button" class="btn btn-outline" :disabled="checking" @click="choosePath(true)">选择文件夹</button>
        <button type="button" class="btn" :disabled="!canCheck" @click="check">
          {{ checking ? '正在检查…' : '检查占用' }}
        </button>
      </div>
      <p class="helper">文件夹会检查其下文件；大型目录扫描到 4,096 个条目时会停止并标记结果不完整。Windows 原生接口无法直接查询仅被文件夹句柄占用的情况。</p>
      <p v-if="error" class="error" role="alert">{{ error }}</p>
    </section>

    <section v-if="result" class="card result" aria-live="polite">
      <div class="result-heading">
        <div>
          <h3>{{ result.processes.length ? `发现 ${result.processes.length} 个占用进程` : '未发现占用进程' }}</h3>
          <p><code>{{ result.path }}</code></p>
        </div>
        <button type="button" class="copy-btn" @click="copy(result.path)">复制路径</button>
      </div>

      <p v-if="result.resourceLimitReached" class="warning">
        此文件夹内容较多，只检查了前 {{ result.registeredResourceCount }} 个路径；请缩小到具体子文件夹后再次检查。
      </p>
      <p v-else class="helper">已检查 {{ result.isDirectory ? '文件夹及其内容' : '文件' }}（{{ result.registeredResourceCount }} 个资源）。</p>

      <div v-if="result.processes.length" class="processes">
        <article v-for="process in result.processes" :key="`${process.processId}-${process.serviceName || ''}`" class="process-card">
          <div class="process-title">
            <strong>{{ process.appName }}</strong>
            <span>{{ process.applicationType }}</span>
          </div>
          <dl>
            <div><dt>进程 ID</dt><dd><code>{{ process.processId }}</code></dd></div>
            <div v-if="process.executablePath"><dt>程序路径</dt><dd><code>{{ process.executablePath }}</code></dd></div>
            <div v-if="process.serviceName"><dt>服务名</dt><dd><code>{{ process.serviceName }}</code></dd></div>
            <div><dt>可重启</dt><dd>{{ process.restartable ? '是' : '否' }}</dd></div>
          </dl>
        </article>
      </div>
      <p v-else class="empty">Windows 没有报告正在占用这个路径的应用。若仍无法操作，可能是检查期间占用已变化，或占用者不在 Windows Restart Manager 可报告范围内。</p>
    </section>
  </div>
</template>

<style scoped>
.file-locks { max-width: 900px; margin: 0 auto; }
.lead { margin: 0 0 16px; color: var(--fg-muted); }
.card { padding: 20px; margin-bottom: 16px; border: 1px solid var(--border); border-radius: var(--radius); background: var(--card); }
label { display: block; margin-bottom: 8px; font-size: 13px; color: var(--fg-muted); }
.path-row { display: flex; gap: 8px; align-items: center; }
.input { min-width: 0; flex: 1 1 auto; min-height: 38px; padding: 7px 10px; border: 1px solid var(--border); border-radius: 5px; background: var(--bg); color: var(--fg); font: 13px ui-monospace, SFMono-Regular, Consolas, monospace; }
.input:focus-visible, button:focus-visible { outline: 2px solid var(--primary); outline-offset: 2px; }
.helper, .error, .warning, .empty { margin: 10px 0 0; font-size: 13px; }
.helper { color: var(--fg-muted); }
.error { color: var(--danger); }
.warning { padding: 10px 12px; border: 1px solid #d97706; border-radius: 5px; color: #a15c06; background: color-mix(in srgb, #d97706 8%, var(--card)); }
.result-heading { display: flex; gap: 16px; align-items: flex-start; justify-content: space-between; }
.result-heading h3 { margin: 0; font-size: 16px; }
.result-heading p { margin: 5px 0 0; color: var(--fg-muted); overflow-wrap: anywhere; }
.processes { display: grid; gap: 10px; margin-top: 16px; }
.process-card { padding: 14px; border: 1px solid var(--border); border-radius: 6px; background: var(--bg); }
.process-title { display: flex; gap: 10px; align-items: center; }
.process-title span { padding: 1px 7px; border: 1px solid var(--border); border-radius: 999px; color: var(--fg-muted); font-size: 12px; }
dl { margin: 10px 0 0; }
dl > div { display: grid; grid-template-columns: 76px minmax(0, 1fr); gap: 10px; padding: 4px 0; }
dt { color: var(--fg-muted); font-size: 12px; }
dd { min-width: 0; margin: 0; overflow-wrap: anywhere; }
code { font: 12px ui-monospace, SFMono-Regular, Consolas, monospace; }
.copy-btn { flex: 0 0 auto; min-height: 32px; padding: 5px 10px; border: 1px solid var(--border); border-radius: 5px; background: transparent; color: var(--fg); cursor: pointer; }
.copy-btn:hover { border-color: var(--primary); color: var(--primary); }
@media (max-width: 720px) {
  .card { padding: 16px; }
  .path-row { flex-wrap: wrap; }
  .input { flex-basis: 100%; }
}
</style>
