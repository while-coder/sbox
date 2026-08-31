<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import {
  getPublicIp,
  getSection,
  getSummary,
  type PublicIpInfo,
  type SectionData,
  type SectionKey,
  type SummaryInfo,
} from './tauri'

/** 瀑布式段定义：label 为卡片标题，key 对应后端采集段（publicIp 走独立接口）。 */
const SECTIONS: { key: SectionKey | 'public'; label: string }[] = [
  { key: 'device', label: '设备与系统' },
  { key: 'cpu', label: '处理器' },
  { key: 'memory', label: '内存' },
  { key: 'graphics', label: '显卡 · 显示器' },
  { key: 'storage', label: '存储' },
  { key: 'network', label: '网络' },
  { key: 'peripherals', label: '外设' },
  { key: 'public', label: '公网出口' },
]

type SectionKeyAll = SectionKey | 'public'

interface SectionState {
  data?: SectionData | PublicIpInfo
  error?: string
  loading: boolean
}

const summary = ref<SummaryInfo | null>(null)
const summaryError = ref('')
const summaryLoading = ref(false)

/** 各段数据：进入页面即并行采集，每段独立显示加载与错误状态。 */
const state = reactive(
  Object.fromEntries(SECTIONS.map(s => [s.key, { loading: false }])) as Record<SectionKeyAll, SectionState>,
)
/** 进行中的采集任务，复制报告时用于等待全部完成 */
const inflight = new Map<SectionKeyAll, Promise<void>>()

const copied = ref(false)
const copying = ref(false)
/** 浏览器侧的语言（展示用；系统时区由后端给出） */
const browserLanguage = navigator.language

const memoryPercent = computed(() => {
  if (!summary.value?.memoryTotalBytes) return 0
  return (summary.value.memoryUsedBytes / summary.value.memoryTotalBytes) * 100
})

function sectionData(key: SectionKeyAll): SectionData | PublicIpInfo | undefined {
  return state[key]?.data
}

/** 采集单个段落；已在加载中则复用同一任务，完成后自动写入 state。 */
function loadSection(key: SectionKeyAll, force = false): Promise<void> {
  const s = state[key]
  if (force) {
    s.data = undefined
    s.error = undefined
  }
  if (s.data || s.loading) return inflight.get(key) ?? Promise.resolve()
  s.loading = true
  s.error = undefined
  const task = (async () => {
    try {
      s.data = key === 'public' ? await getPublicIp() : await getSection(key)
    } catch (e: any) {
      s.error = String(e?.message || e)
    } finally {
      s.loading = false
      inflight.delete(key)
    }
  })()
  inflight.set(key, task)
  return task
}

async function loadSummary() {
  summaryLoading.value = true
  summaryError.value = ''
  try {
    summary.value = await getSummary()
  } catch (e: any) {
    summaryError.value = String(e?.message || e)
  } finally {
    summaryLoading.value = false
  }
}

onMounted(() => {
  loadSummary()
  for (const s of SECTIONS) loadSection(s.key)
})

/** 刷新：清空全部段落缓存后并行重采。 */
async function refresh() {
  await Promise.all([
    loadSummary(),
    ...SECTIONS.map(s => loadSection(s.key, true)),
  ])
}

function formatBytes(bytes?: number | null): string {
  if (bytes == null || Number.isNaN(bytes) || bytes <= 0) return '—'
  const units = ['B', 'KB', 'MB', 'GB', 'TB', 'PB']
  let value = bytes
  let unit = 0
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024
    unit += 1
  }
  const precision = value >= 100 || unit === 0 ? 0 : 1
  return `${value.toFixed(precision)} ${units[unit]}`
}

function formatFrequency(mhz?: number | null): string {
  if (!mhz) return '—'
  return `${(mhz / 1000).toFixed(2)} GHz`
}

function formatCache(kb?: number | null): string {
  if (!kb) return '—'
  if (kb < 1024) return `${kb} KB`
  return `${(kb / 1024).toFixed(kb % 1024 === 0 ? 0 : 1)} MB`
}

function formatUptime(secs: number): string {
  const days = Math.floor(secs / 86400)
  const hours = Math.floor((secs % 86400) / 3600)
  const minutes = Math.floor((secs % 3600) / 60)
  if (days > 0) return `${days} 天 ${hours} 小时`
  if (hours > 0) return `${hours} 小时 ${minutes} 分钟`
  return `${minutes} 分钟`
}

function formatTime(unixSecs?: number | null): string {
  if (!unixSecs) return '—'
  return new Date(unixSecs * 1000).toLocaleString()
}

function join(...values: (string | undefined | null)[]): string {
  return values.filter(value => value && value.trim()).join(' ') || '—'
}

/** 汇总成纯文本报告，方便贴到工单或群聊里排查环境问题。 */
function buildReport(): string {
  const lines: string[] = []
  if (summary.value) {
    const s = summary.value
    lines.push('【概要】')
    lines.push(`计算机名: ${s.hostname || '—'}`, `用户: ${s.username || '—'}`)
    lines.push(`系统: ${join(s.osEdition, s.cpuArch)}`)
    lines.push(`CPU: ${s.cpuName}（${s.cpuLogicalCores} 线程，使用率 ${s.cpuUsagePercent.toFixed(0)}%）`)
    lines.push(`内存: ${formatBytes(s.memoryUsedBytes)} / ${formatBytes(s.memoryTotalBytes)}`)
    if (s.ipv4.length) lines.push(`本机 IPv4: ${s.ipv4.join(', ')}`)
    lines.push(`已运行: ${formatUptime(s.uptimeSecs)}`)
  }
  const device = sectionData('device')
  if (device?.kind === 'device') {
    const d = device.device
    lines.push('', '【设备与系统】')
    lines.push(`设备: ${join(d.manufacturer, d.model)}`)
    lines.push(`主板: ${join(d.boardManufacturer, d.boardModel, d.boardVersion)}`)
    lines.push(`BIOS: ${join(d.biosVendor, d.biosVersion, d.biosDate)}`)
    lines.push(`系统版本: ${join(d.osVersion, d.osDisplayVersion)}（Build ${d.osBuild || '—'}）`)
    lines.push(`时区: ${d.timezone || '—'}`, `语言: ${browserLanguage}`)
    lines.push(`安装时间: ${formatTime(d.osInstallTime)}`, `开机时间: ${formatTime(d.bootTime)}`)
    if (device.battery) {
      const battery = device.battery
      lines.push(`电池: ${battery.chargePercent != null ? `${battery.chargePercent}%` : '—'} ${battery.state || ''}${battery.runtimeMinutes ? `（约剩 ${formatUptime(battery.runtimeMinutes * 60)}）` : ''}`)
    }
  }
  const cpu = sectionData('cpu')
  if (cpu?.kind === 'cpu') {
    const c = cpu.cpu
    lines.push('', '【处理器】')
    lines.push(`${c.name}（${c.vendor || '—'}，插槽 ${c.socket || '—'}）`)
    lines.push(`${c.physicalCores ?? '—'} 核 ${c.logicalCores} 线程，基准 ${formatFrequency(c.baseFrequencyMhz)}，当前 ${formatFrequency(c.currentFrequencyMhz)}`)
    lines.push(`L2 ${formatCache(c.l2CacheKb)}，L3 ${formatCache(c.l3CacheKb)}，使用率 ${c.usagePercent.toFixed(0)}%`)
  }
  const memory = sectionData('memory')
  if (memory?.kind === 'memory') {
    const m = memory.memory
    lines.push('', '【内存】')
    if (m.slotCount) lines.push(`插槽：${m.modules.length}/${m.slotCount} 已用`)
    for (const module of m.modules) {
      lines.push(`  ${module.slot || '—'}：${formatBytes(module.capacityBytes)} ${module.kind || ''} ${module.manufacturer || ''} ${module.partNumber || ''} ${module.configuredSpeedMhz || module.speedMhz ? `${module.configuredSpeedMhz || module.speedMhz} MHz` : ''}`)
    }
  }
  const graphics = sectionData('graphics')
  if (graphics?.kind === 'graphics') {
    lines.push('', '【显卡】')
    if (!graphics.gpus.length) lines.push('未识别到显卡')
    for (const gpu of graphics.gpus) {
      lines.push(`${gpu.name}${gpu.driverVersion ? `，驱动 ${gpu.driverVersion}` : ''}${gpu.memoryBytes ? `，显存 ${formatBytes(gpu.memoryBytes)}` : ''}${gpu.resolution ? `，当前分辨率 ${gpu.resolution}${gpu.refreshRateHz ? ` @ ${gpu.refreshRateHz}Hz` : ''}` : ''}`)
    }
    lines.push('', '【显示器】')
    if (!graphics.monitors.length) lines.push('未识别到显示器')
    for (const monitor of graphics.monitors) {
      lines.push(`${monitor.name || '未知显示器'}${monitor.sizeInches ? `，${monitor.sizeInches}"` : ''}${monitor.resolution ? `，${monitor.resolution}` : ''}${monitor.refreshRateHz ? ` @ ${monitor.refreshRateHz}Hz` : ''}`)
    }
  }
  const storage = sectionData('storage')
  if (storage?.kind === 'storage') {
    lines.push('', '【物理磁盘】')
    if (!storage.drives.length) lines.push('未识别到物理磁盘')
    for (const drive of storage.drives) {
      lines.push(`${drive.model}：${formatBytes(drive.sizeBytes)}，${join(drive.interface, drive.mediaType)}${drive.partitionCount ? `，${drive.partitionCount} 个分区` : ''}`)
    }
    lines.push('', '【存储卷】')
    for (const volume of storage.volumes) {
      lines.push(`${volume.mount}（${volume.label || '本地磁盘'}，${volume.fileSystem || '—'}${volume.diskModel ? `，位于 ${volume.diskModel}` : ''}）：${formatBytes(volume.freeBytes)} 可用 / 共 ${formatBytes(volume.totalBytes)}`)
    }
  }
  const network = sectionData('network')
  if (network?.kind === 'network') {
    lines.push('', '【网络适配器】')
    if (!network.networks.length) lines.push('未识别到网络适配器')
    for (const adapter of network.networks) {
      lines.push(`${adapter.name}：MAC ${adapter.mac || '—'}`)
      if (adapter.ipv4.length) lines.push(`  IPv4: ${adapter.ipv4.join(', ')}`)
      if (adapter.ipv6.length) lines.push(`  IPv6: ${adapter.ipv6.join(', ')}`)
      if (adapter.gateway) lines.push(`  网关: ${adapter.gateway}`)
    }
  }
  const peripherals = sectionData('peripherals')
  if (peripherals?.kind === 'peripherals' && peripherals.peripherals.length) {
    lines.push('', '【外设】')
    for (const peripheral of peripherals.peripherals) {
      lines.push(`${peripheral.kind}: ${peripheral.name}${peripheral.manufacturer ? `（${peripheral.manufacturer}）` : ''}`)
    }
  }
  const publicIp = sectionData('public')
  if (publicIp && 'ip' in publicIp) {
    lines.push('', '【公网出口】')
    lines.push(`公网 IP: ${publicIp.ip}`)
    if (join(publicIp.country, publicIp.region, publicIp.city) !== '—') {
      lines.push(`归属地: ${[publicIp.country, publicIp.region, publicIp.city].filter(Boolean).join(' ')}`)
    }
    if (publicIp.isp) lines.push(`运营商: ${publicIp.isp}`)
    if (publicIp.latitude != null && publicIp.longitude != null) {
      lines.push(`定位: ${publicIp.latitude.toFixed(4)}, ${publicIp.longitude.toFixed(4)}（IP 估算）`)
    }
  }
  return lines.join('\n')
}

/** 复制前先等全部段落采集完成，保证报告完整；失败的段落会跳过。 */
async function copyReport() {
  copying.value = true
  try {
    await Promise.all(SECTIONS.map(s => loadSection(s.key)))
    await navigator.clipboard.writeText(buildReport())
    copied.value = true
    setTimeout(() => {
      copied.value = false
    }, 1500)
  } catch (e: any) {
    summaryError.value = `复制失败: ${String(e?.message || e)}`
  } finally {
    copying.value = false
  }
}
</script>

<template>
  <div class="system-info">
    <div class="heading">
      <div>
        <h2>本机信息</h2>
        <p class="lead">操作系统、硬件规格与网络环境一览，可复制完整报告用于环境排查。</p>
      </div>
      <div class="actions">
        <button type="button" class="btn btn-outline" :disabled="copying || summaryLoading" @click="copyReport">
          {{ copying ? '正在采集…' : copied ? '已复制 ✓' : '复制报告' }}
        </button>
        <button type="button" class="btn" :disabled="summaryLoading" @click="refresh">
          {{ summaryLoading ? '正在读取…' : '刷新' }}
        </button>
      </div>
    </div>

    <!-- 概要：全部来自 sysinfo，秒级返回，置于页面最上方 -->
    <section class="card summary">
      <p v-if="summaryError" class="error" role="alert">{{ summaryError }}</p>
      <p v-else-if="!summary" class="empty">正在读取概要…</p>
      <template v-else>
        <dl class="stats">
          <div><dt>系统</dt><dd>{{ join(summary.osEdition, summary.cpuArch) }}</dd></div>
          <div><dt>计算机名</dt><dd><code>{{ summary.hostname || '—' }}</code></dd></div>
          <div><dt>处理器</dt><dd>{{ summary.cpuName || '—' }}（{{ summary.cpuLogicalCores }} 线程，{{ summary.cpuUsagePercent.toFixed(0) }}%）</dd></div>
          <div><dt>内存</dt><dd>{{ formatBytes(summary.memoryUsedBytes) }} / {{ formatBytes(summary.memoryTotalBytes) }}（{{ memoryPercent.toFixed(0) }}%）</dd></div>
          <div><dt>本机 IPv4</dt><dd><code>{{ summary.ipv4.join('，') || '—' }}</code></dd></div>
          <div><dt>当前用户</dt><dd>{{ summary.username || '—' }}</dd></div>
          <div><dt>已运行</dt><dd>{{ formatUptime(summary.uptimeSecs) }}</dd></div>
        </dl>
        <div class="bar"><div class="bar-fill" :style="{ width: `${Math.min(memoryPercent, 100)}%` }" /></div>
      </template>
    </section>

    <!-- 详细信息按段落瀑布排列：进入页面即并行采集，每段独立显示加载与错误状态 -->
    <template v-for="sec in SECTIONS" :key="sec.key">
      <section class="card section">
        <h3>{{ sec.label }}</h3>
        <p v-if="state[sec.key].error" class="error" role="alert">{{ state[sec.key].error }}</p>
        <p v-else-if="state[sec.key].loading" class="empty">正在采集…</p>
        <p v-else-if="!state[sec.key].data" class="empty">暂无数据。</p>

        <!-- 设备与系统 -->
        <template v-else-if="sec.key === 'device' && state[sec.key].data.kind === 'device'">
          <dl>
            <div><dt>计算机名</dt><dd><code>{{ state[sec.key].data.device.hostname || '—' }}</code></dd></div>
            <div><dt>当前用户</dt><dd>{{ state[sec.key].data.device.username || '—' }}</dd></div>
            <div><dt>设备型号</dt><dd>{{ join(state[sec.key].data.device.manufacturer, state[sec.key].data.device.model) }}</dd></div>
            <div><dt>主板</dt><dd>{{ join(state[sec.key].data.device.boardManufacturer, state[sec.key].data.device.boardModel, state[sec.key].data.device.boardVersion) }}</dd></div>
            <div><dt>BIOS</dt><dd>{{ join(state[sec.key].data.device.biosVendor, state[sec.key].data.device.biosVersion, state[sec.key].data.device.biosDate) }}</dd></div>
            <div><dt>操作系统</dt><dd>{{ join(state[sec.key].data.device.osEdition, state[sec.key].data.device.cpuArch) }}</dd></div>
            <div><dt>系统版本</dt><dd>{{ join(state[sec.key].data.device.osVersion, state[sec.key].data.device.osDisplayVersion) }}（Build {{ state[sec.key].data.device.osBuild || '—' }}）</dd></div>
            <div><dt>系统安装时间</dt><dd>{{ formatTime(state[sec.key].data.device.osInstallTime) }}</dd></div>
            <div><dt>时区 / 语言</dt><dd>{{ state[sec.key].data.device.timezone || '—' }} / {{ browserLanguage }}</dd></div>
            <div v-if="state[sec.key].data.battery"><dt>电池</dt><dd>
              {{ state[sec.key].data.battery.chargePercent != null ? `${state[sec.key].data.battery.chargePercent}%` : '—' }}
              {{ state[sec.key].data.battery.state || '' }}
              <span v-if="state[sec.key].data.battery.runtimeMinutes">，约剩 {{ formatUptime(state[sec.key].data.battery.runtimeMinutes * 60) }}</span>
            </dd></div>
            <div><dt>本次开机时间</dt><dd>{{ formatTime(state[sec.key].data.device.bootTime) }}</dd></div>
            <div><dt>已运行</dt><dd>{{ formatUptime(state[sec.key].data.device.uptimeSecs) }}</dd></div>
          </dl>
        </template>

        <!-- 处理器 -->
        <template v-else-if="sec.key === 'cpu' && state[sec.key].data.kind === 'cpu'">
          <dl>
            <div><dt>型号</dt><dd>{{ state[sec.key].data.cpu.name || '—' }}</dd></div>
            <div><dt>厂商</dt><dd>{{ state[sec.key].data.cpu.vendor || '—' }}</dd></div>
            <div><dt>插槽</dt><dd>{{ state[sec.key].data.cpu.socket || '—' }}</dd></div>
            <div><dt>核心 / 线程</dt><dd>{{ state[sec.key].data.cpu.physicalCores ?? '—' }} 核 / {{ state[sec.key].data.cpu.logicalCores }} 线程</dd></div>
            <div><dt>基准频率</dt><dd>{{ formatFrequency(state[sec.key].data.cpu.baseFrequencyMhz) }}</dd></div>
            <div><dt>当前频率</dt><dd>{{ formatFrequency(state[sec.key].data.cpu.currentFrequencyMhz) }}</dd></div>
            <div><dt>L2 / L3 缓存</dt><dd>{{ formatCache(state[sec.key].data.cpu.l2CacheKb) }} / {{ formatCache(state[sec.key].data.cpu.l3CacheKb) }}</dd></div>
            <div><dt>使用率</dt><dd>{{ state[sec.key].data.cpu.usagePercent.toFixed(0) }}%</dd></div>
          </dl>
          <div v-if="state[sec.key].data.cpu.cores.length" class="cores">
            <div v-for="core in state[sec.key].data.cpu.cores" :key="core.name" class="core" :title="`${formatFrequency(core.frequencyMhz)}`">
              <span class="core-name">{{ core.name }}</span>
              <div class="bar"><div class="bar-fill" :style="{ width: `${Math.min(core.usagePercent, 100)}%` }" /></div>
              <span class="core-meta">{{ core.usagePercent.toFixed(0) }}% · {{ formatFrequency(core.frequencyMhz) }}</span>
            </div>
          </div>
        </template>

        <!-- 内存 -->
        <template v-else-if="sec.key === 'memory' && state[sec.key].data.kind === 'memory'">
          <div class="usage">
            <div class="usage-text">
              <span>已用 {{ formatBytes(state[sec.key].data.memory.usedBytes) }}</span>
              <span>共 {{ formatBytes(state[sec.key].data.memory.totalBytes) }}（{{ ((state[sec.key].data.memory.usedBytes / state[sec.key].data.memory.totalBytes) * 100).toFixed(0) }}%）</span>
            </div>
            <div class="bar"><div class="bar-fill" :style="{ width: `${Math.min((state[sec.key].data.memory.usedBytes / state[sec.key].data.memory.totalBytes) * 100, 100)}%` }" /></div>
            <p class="helper">可用 {{ formatBytes(state[sec.key].data.memory.freeBytes) }}<template v-if="state[sec.key].data.memory.swapTotalBytes">；交换分区已用 {{ formatBytes(state[sec.key].data.memory.swapUsedBytes) }} / {{ formatBytes(state[sec.key].data.memory.swapTotalBytes) }}</template></p>
          </div>
          <dl v-if="state[sec.key].data.memory.modules.length || state[sec.key].data.memory.slotCount">
            <div>
              <dt>内存条</dt>
              <dd>
                <span v-if="!state[sec.key].data.memory.modules.length">未读取到内存条信息</span>
                <span v-else-if="state[sec.key].data.memory.slotCount">已安装 {{ state[sec.key].data.memory.modules.length }} / {{ state[sec.key].data.memory.slotCount }} 条</span>
              </dd>
            </div>
            <div v-for="(module, index) in state[sec.key].data.memory.modules" :key="`${module.slot}-${index}`">
              <dt>{{ module.slot || module.bank || '内存条' }}</dt>
              <dd>{{ join(module.kind, formatBytes(module.capacityBytes), module.manufacturer, module.partNumber, module.configuredSpeedMhz || module.speedMhz ? `${module.configuredSpeedMhz || module.speedMhz} MHz` : '') }}</dd>
            </div>
          </dl>
        </template>

        <!-- 显卡 · 显示器 -->
        <template v-else-if="sec.key === 'graphics' && state[sec.key].data.kind === 'graphics'">
          <template v-if="state[sec.key].data.gpus.length">
            <h4>显卡</h4>
            <article v-for="gpu in state[sec.key].data.gpus" :key="gpu.name" class="item">
              <strong>{{ gpu.name }}</strong>
              <dl>
                <div><dt>厂商</dt><dd>{{ gpu.vendor || '—' }}</dd></div>
                <div><dt>驱动版本</dt><dd><code>{{ gpu.driverVersion || '—' }}</code></dd></div>
                <div><dt>显存</dt><dd>{{ formatBytes(gpu.memoryBytes) }}</dd></div>
                <div><dt>当前输出</dt><dd>{{ gpu.resolution ? `${gpu.resolution}${gpu.refreshRateHz ? ` @ ${gpu.refreshRateHz} Hz` : ''}` : '—' }}</dd></div>
                <div><dt>设备状态</dt><dd>{{ gpu.status || '—' }}</dd></div>
              </dl>
            </article>
          </template>
          <p v-else class="empty">未识别到显卡信息。</p>
          <template v-if="state[sec.key].data.monitors.length">
            <h4>显示器</h4>
            <article v-for="(monitor, index) in state[sec.key].data.monitors" :key="`${monitor.name}-${index}`" class="item">
              <strong>{{ monitor.name || '未知显示器' }}</strong>
              <dl>
                <div v-if="monitor.sizeInches"><dt>尺寸</dt><dd>{{ monitor.sizeInches }}"</dd></div>
                <div><dt>分辨率 / 刷新率</dt><dd>{{ monitor.resolution || '—' }}{{ monitor.refreshRateHz ? ` @ ${monitor.refreshRateHz} Hz` : '' }}</dd></div>
              </dl>
            </article>
          </template>
          <p v-else class="empty">未识别到显示器信息。</p>
        </template>

        <!-- 存储 -->
        <template v-else-if="sec.key === 'storage' && state[sec.key].data.kind === 'storage'">
          <h4>物理磁盘</h4>
          <template v-if="state[sec.key].data.drives.length">
            <article v-for="drive in state[sec.key].data.drives" :key="drive.serial || drive.model" class="item">
              <strong>{{ drive.model }}</strong>
              <dl>
                <div><dt>容量</dt><dd>{{ formatBytes(drive.sizeBytes) }}</dd></div>
                <div><dt>接口 / 类型</dt><dd>{{ join(drive.interface, drive.mediaType) }}</dd></div>
                <div v-if="drive.partitionCount"><dt>分区数</dt><dd>{{ drive.partitionCount }}</dd></div>
                <div><dt>序列号</dt><dd><code>{{ drive.serial || '—' }}</code></dd></div>
              </dl>
            </article>
          </template>
          <p v-else class="empty">未识别到物理磁盘信息。</p>
          <h4>存储卷</h4>
          <template v-if="state[sec.key].data.volumes.length">
            <article v-for="volume in state[sec.key].data.volumes" :key="volume.mount" class="item">
              <div class="usage">
                <div class="usage-text">
                  <span><strong>{{ volume.mount }}</strong>（{{ join(volume.label, volume.fileSystem) }}{{ volume.kind ? `，${volume.kind}` : '' }}）</span>
                  <span>{{ formatBytes(volume.freeBytes) }} 可用 / 共 {{ formatBytes(volume.totalBytes) }}</span>
                </div>
                <p v-if="volume.diskModel" class="helper">位于 {{ volume.diskModel }}</p>
                <div v-if="volume.totalBytes" class="bar">
                  <div class="bar-fill" :style="{ width: `${Math.min(((volume.totalBytes - (volume.freeBytes || 0)) / volume.totalBytes) * 100, 100)}%` }" />
                </div>
              </div>
            </article>
          </template>
          <p v-else class="empty">未识别到存储卷。</p>
        </template>

        <!-- 网络 -->
        <template v-else-if="sec.key === 'network' && state[sec.key].data.kind === 'network'">
          <template v-if="state[sec.key].data.networks.length">
            <article v-for="network in state[sec.key].data.networks" :key="network.mac || network.name" class="item">
              <strong>{{ network.name }}</strong>
              <dl>
                <div><dt>IPv4</dt><dd><code>{{ network.ipv4.join('，') || '—' }}</code></dd></div>
                <div v-if="network.ipv6.length"><dt>IPv6</dt><dd><code>{{ network.ipv6.join('，') }}</code></dd></div>
                <div><dt>MAC 地址</dt><dd><code>{{ network.mac || '—' }}</code></dd></div>
                <div v-if="network.gateway"><dt>默认网关</dt><dd><code>{{ network.gateway }}</code></dd></div>
              </dl>
            </article>
          </template>
          <p v-else class="empty">未识别到已联网的网络适配器。</p>
        </template>

        <!-- 外设 -->
        <template v-else-if="sec.key === 'peripherals' && state[sec.key].data.kind === 'peripherals'">
          <dl v-if="state[sec.key].data.peripherals.length">
            <div v-for="(peripheral, index) in state[sec.key].data.peripherals" :key="`${peripheral.name}-${index}`">
              <dt>{{ peripheral.kind }}</dt>
              <dd>{{ peripheral.name }}{{ peripheral.manufacturer ? `（${peripheral.manufacturer}）` : '' }}{{ peripheral.status ? ` · ${peripheral.status}` : '' }}</dd>
            </div>
          </dl>
          <p v-else class="empty">未识别到外设信息（台式机未接外设或系统不支持时为空）。</p>
        </template>

        <!-- 公网出口 -->
        <template v-else-if="sec.key === 'public' && state[sec.key].data && 'ip' in state[sec.key].data">
          <dl>
            <div><dt>公网 IP</dt><dd><code>{{ state[sec.key].data.ip }}</code></dd></div>
            <div><dt>归属地</dt><dd>{{ [state[sec.key].data.country, state[sec.key].data.region, state[sec.key].data.city].filter(Boolean).join(' ') || '—' }}</dd></div>
            <div v-if="state[sec.key].data.isp"><dt>运营商</dt><dd>{{ state[sec.key].data.isp }}</dd></div>
            <div v-if="state[sec.key].data.latitude != null && state[sec.key].data.longitude != null"><dt>定位</dt><dd>{{ state[sec.key].data.latitude.toFixed(4) }}, {{ state[sec.key].data.longitude.toFixed(4) }}（IP 估算，非 GPS）</dd></div>
            <div v-if="state[sec.key].data.timezone"><dt>网络时区</dt><dd>{{ state[sec.key].data.timezone }}</dd></div>
          </dl>
        </template>
      </section>
    </template>
  </div>
</template>

<style scoped>
.system-info { max-width: 900px; margin: 0 auto; }
.heading { display: flex; gap: 16px; align-items: flex-start; justify-content: space-between; margin-bottom: 16px; }
.heading h2 { margin: 0; }
.lead { margin: 6px 0 0; color: var(--fg-muted); }
.actions { display: flex; flex: 0 0 auto; gap: 8px; }
.error { margin: 0 0 12px; color: var(--danger); }
.card { padding: 20px; border: 1px solid var(--border); border-radius: var(--radius); background: var(--card); }
.summary { margin-bottom: 12px; }
.section { margin-bottom: 12px; }
.section > h3 { margin: 0 0 12px; font-size: 15px; }
.card h4 { margin: 16px 0 12px; font-size: 15px; }
dl { margin: 0; }
dl > div { display: grid; grid-template-columns: 110px minmax(0, 1fr); gap: 10px; padding: 5px 0; border-bottom: 1px dashed var(--border); }
dl > div:last-child { border-bottom: none; }
dt { color: var(--fg-muted); font-size: 12px; padding-top: 2px; }
dd { min-width: 0; margin: 0; overflow-wrap: anywhere; font-size: 13px; }
code { font: 12px ui-monospace, SFMono-Regular, Consolas, monospace; }
.item { padding: 12px 14px; margin-bottom: 10px; border: 1px solid var(--border); border-radius: 6px; background: var(--bg); }
.item:last-child { margin-bottom: 0; }
.item > strong { display: block; margin-bottom: 6px; font-size: 13px; }
.usage { margin-bottom: 10px; }
.usage-text { display: flex; gap: 12px; align-items: baseline; justify-content: space-between; flex-wrap: wrap; font-size: 13px; }
.helper { margin: 6px 0 0; font-size: 12px; color: var(--fg-muted); }
.empty { margin: 0; font-size: 13px; color: var(--fg-muted); }
.bar { height: 6px; margin-top: 6px; border-radius: 999px; background: color-mix(in srgb, var(--fg) 10%, transparent); overflow: hidden; }
.bar-fill { height: 100%; border-radius: inherit; background: var(--primary); transition: width 0.3s ease; }
/* 每核心频率与使用率 */
.cores { display: grid; grid-template-columns: repeat(auto-fill, minmax(190px, 1fr)); gap: 8px 14px; margin-top: 14px; }
.core { display: grid; grid-template-columns: 44px minmax(0, 1fr); gap: 4px 8px; align-items: center; }
.core-name { font-size: 11px; color: var(--fg-muted); }
.core-meta { grid-column: 2; font-size: 11px; color: var(--fg-muted); }
.core .bar { margin-top: 0; }
button { min-height: 32px; padding: 5px 12px; border: 1px solid var(--border); border-radius: 5px; background: transparent; color: var(--fg); cursor: pointer; }
button:hover:not(:disabled) { border-color: var(--primary); color: var(--primary); }
button:disabled { opacity: 0.55; cursor: default; }
button:focus-visible { outline: 2px solid var(--primary); outline-offset: 2px; }
.btn { background: var(--primary); border-color: var(--primary); color: #fff; }
.btn:hover:not(:disabled) { opacity: 0.9; color: #fff; }
@media (max-width: 720px) {
  .card { padding: 16px; }
  .heading { flex-direction: column; }
  dl > div { grid-template-columns: 92px minmax(0, 1fr); }
}
</style>
