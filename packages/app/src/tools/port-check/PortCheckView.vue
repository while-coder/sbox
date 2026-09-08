<script setup lang="ts">
import { computed, ref } from 'vue'
import { killProcess, listPort, type PortCheckResult } from './tauri'

const port = ref('')
const checking = ref(false)
const killing = ref<number | null>(null)
const error = ref('')
const result = ref<PortCheckResult | null>(null)

const parsedPort = computed(() => {
  const value = Number(port.value.trim())
  return Number.isInteger(value) && value >= 1 && value <= 65535 ? value : null
})
const canCheck = computed(() => parsedPort.value !== null && !checking.value)

async function check() {
  if (!canCheck.value) return
  checking.value = true
  error.value = ''
  try {
    result.value = await listPort(parsedPort.value!)
  } catch (e: any) {
    error.value = String(e?.message || e)
    result.value = null
  } finally {
    checking.value = false
  }
}

async function kill(pid: number, name: string) {
  if (killing.value !== null) return
  if (!window.confirm(`确定强制结束进程 ${name}（PID ${pid}）吗？未保存的数据会丢失。`)) return
  killing.value = pid
  error.value = ''
  try {
    await killProcess(pid)
    await check()
  } catch (e: any) {
    error.value = String(e?.message || e)
  } finally {
    killing.value = null
  }
}
</script>

<template>
  <div class="port-check">
    <h2>端口占用检查</h2>
    <p class="lead">查看本地端口被哪些进程监听或连接，可一键结束占用进程（进程结束后自动重新检查）。</p>

    <section class="card">
      <label for="port-check-port">端口号（1 - 65535）</label>
      <div class="port-row">
        <input
          id="port-check-port"
          v-model="port"
          class="input port-input"
          type="text"
          inputmode="numeric"
          autocomplete="off"
          spellcheck="false"
          placeholder="如 8080、3000"
          @keydown.enter="check"
        >
        <button type="button" class="btn" :disabled="!canCheck" @click="check">
          {{ checking ? '正在检查…' : '检查占用' }}
        </button>
      </div>
      <p class="helper">同时检查 TCP 与 UDP 的 IPv4 / IPv6 监听及连接；结果不自动刷新，可再次点击检查。</p>
      <p v-if="port.trim() && parsedPort === null" class="error" role="alert">请输入 1 - 65535 之间的整数端口号</p>
      <p v-else-if="error" class="error" role="alert">{{ error }}</p>
    </section>

    <section v-if="result" class="card result" aria-live="polite">
      <div class="result-heading">
        <h3>{{ result.processes.length ? `端口 ${result.port} 被 ${result.processes.length} 个进程占用` : `端口 ${result.port} 空闲` }}</h3>
      </div>

      <div v-if="result.processes.length" class="processes">
        <article v-for="process in result.processes" :key="process.pid" class="process-card">
          <div class="process-title">
            <strong>{{ process.processName }}</strong>
            <code>PID {{ process.pid }}</code>
            <button
              type="button"
              class="kill-btn"
              :disabled="killing !== null"
              @click="kill(process.pid, process.processName)"
            >
              {{ killing === process.pid ? '正在结束…' : '结束进程' }}
            </button>
          </div>
          <p v-if="process.executablePath" class="exe"><code>{{ process.executablePath }}</code></p>
          <ul class="sockets">
            <li v-for="(socket, index) in process.sockets" :key="index">
              <span class="protocol" :class="socket.protocol.toLowerCase()">{{ socket.protocol }}</span>
              <code>{{ socket.localAddress }}</code>
              <span class="state">{{ socket.state }}</span>
            </li>
          </ul>
        </article>
      </div>
      <p v-else class="empty">没有进程占用该端口，可以放心使用。</p>
    </section>
  </div>
</template>

<style scoped>
.port-check { max-width: 900px; margin: 0 auto; }
.lead { margin: 0 0 16px; color: var(--fg-muted); }
.card { padding: 20px; margin-bottom: 16px; border: 1px solid var(--border); border-radius: var(--radius); background: var(--card); }
label { display: block; margin-bottom: 8px; font-size: 13px; color: var(--fg-muted); }
.port-row { display: flex; gap: 8px; align-items: center; }
.port-input { max-width: 220px; min-height: 38px; padding: 7px 10px; border: 1px solid var(--border); border-radius: 5px; background: var(--bg); color: var(--fg); font: 13px ui-monospace, SFMono-Regular, Consolas, monospace; }
.input:focus-visible, button:focus-visible { outline: 2px solid var(--primary); outline-offset: 2px; }
.helper, .error, .empty { margin: 10px 0 0; font-size: 13px; }
.helper { color: var(--fg-muted); }
.error { color: var(--danger); }
.result-heading h3 { margin: 0; font-size: 16px; }
.processes { display: grid; gap: 10px; margin-top: 16px; }
.process-card { padding: 14px; border: 1px solid var(--border); border-radius: 6px; background: var(--bg); }
.process-title { display: flex; gap: 10px; align-items: center; flex-wrap: wrap; }
.process-title code { color: var(--fg-muted); font-size: 12px; }
.exe { margin: 8px 0 0; overflow-wrap: anywhere; }
.sockets { margin: 10px 0 0; padding: 0; list-style: none; }
.sockets li { display: flex; gap: 10px; align-items: center; padding: 4px 0; border-top: 1px solid var(--border); }
.protocol { padding: 1px 7px; border: 1px solid var(--border); border-radius: 999px; font-size: 12px; color: var(--fg-muted); }
.protocol.tcp { color: var(--primary); border-color: var(--primary); }
.state { color: var(--fg-muted); font-size: 12px; }
code { font: 12px ui-monospace, SFMono-Regular, Consolas, monospace; }
.kill-btn { margin-left: auto; min-height: 30px; padding: 4px 12px; border: 1px solid var(--danger); border-radius: 5px; background: transparent; color: var(--danger); cursor: pointer; }
.kill-btn:hover:not(:disabled) { background: var(--danger); color: #fff; }
.kill-btn:disabled { opacity: 0.5; cursor: default; }
.empty { color: var(--fg-muted); }
@media (max-width: 720px) {
  .card { padding: 16px; }
  .port-row { flex-wrap: wrap; }
  .port-input { flex: 1 1 auto; max-width: none; }
}
</style>
