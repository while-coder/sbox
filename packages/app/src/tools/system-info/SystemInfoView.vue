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

/** 页签定义：label 显示名，section 对应后端采集段（publicIp 走独立接口）。 */
const TABS: { key: SectionKey | 'public'; label: string }[] = [
  { key: 'device', label: '设备与系统' },
  { key: 'cpu', label: '处理器' },
  { key: 'memory', label: '内存' },
  { key: 'graphics', label: '显卡 · 显示器' },
  { key: 'storage', label: '存储' },
  { key: 'network', label: '网络' },
  { key: 'peripherals', label: '外设' },
  { key: 'public', label: '公网出口' },
]

const summary = ref<SummaryInfo | null>(null)
const summaryError = ref('')
const summaryLoading = ref(false)

/** 各页签数据缓存：首次切到才采集，之后直接复用 */
const cache = reactive(new Map<SectionKey | 'public', SectionData | PublicIpInfo>())
const loadingTabs = reactive(new Set<SectionKey | 'public'>())
const tabErrors = reactive(new Map<SectionKey | 'public', string>())

const activeTab = ref<SectionKey | 'public'>('device')
const copied = ref(false)
/** 浏览器侧的语言（展示用；系统时区由后端给出） */
const browserLanguage = navigator.language

const activeSection = computed(() => cache.get(activeTab.value) ?? null)
const activeError = computed(() => tabErrors.get(activeTab.value) ?? '')
const activeLoading = computed(() => loadingTabs.has(activeTab.value))

const memoryPercent = computed(() => {
  if (!summary.value?.memoryTotalBytes) return 0
  return (summary.value.memoryUsedBytes / summary.value.memoryTotalBytes) * 100
})

/** 首次切到某页签时才采集对应数据，避免进入页面一次性等待全部查询。 */
async function ensureSection(tab: SectionKey | 'public') {
  if (cache.has(tab) || loadingTabs.has(tab)) return
  loadingTabs.add(tab)
  tabErrors.delete(tab)
  try {
    cache.set(tab, tab === 'public' ? await getPublicIp() : await getSection(tab))
  } catch (e: any) {
    tabErrors.set(tab, String(e?.message || e))
  } finally {
    loadingTabs.delete(tab)
  }
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
  ensureSection('device')
})

function switchTab(tab: SectionKey | 'public') {
  activeTab.value = tab
  ensureSection(tab)
}

/** 刷新：清空页签缓存后重载概要与当前页签。 */
async function refresh() {
  cache.clear()
  tabErrors.clear()
  await Promise.all([loadSummary(), ensureSection(activeTab.value)])
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

/** 汇总成纯文本报告，方便贴到工单或群聊里排查环境问题。只包含已加载的页签。 */
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
  const device = cache.get('device')
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
  const cpu = cache.get('cpu')
  if (cpu?.kind === 'cpu') {
    const c = cpu.cpu
    lines.push('', '【处理器】')
    lines.push(`${c.name}（${c.vendor || '—'}，插槽 ${c.socket || '—'}）`)
    lines.push(`${c.physicalCores ?? '—'} 核 ${c.logicalCores} 线程，基准 ${formatFrequency(c.baseFrequencyMhz)}，当前 ${formatFrequency(c.currentFrequencyMhz)}`)
    lines.push(`L2 ${formatCache(c.l2CacheKb)}，L3 ${formatCache(c.l3CacheKb)}，使用率 ${c.usagePercent.toFixed(0)}%`)
  }
  const memory = cache.get('memory')
  if (memory?.kind === 'memory') {
    const m = memory.memory
    lines.push('', '【内存】')
    if (m.slotCount) lines.push(`插槽：${m.modules.length}/${m.slotCount} 已用`)
    for (const module of m.modules) {
      lines.push(`  ${module.slot || '—'}：${formatBytes(module.capacityBytes)} ${module.kind || ''} ${module.manufacturer || ''} ${module.partNumber || ''} ${module.configuredSpeedMhz || module.speedMhz ? `${module.configuredSpeedMhz || module.speedMhz} MHz` : ''}`)
    }
  }
  const graphics = cache.get('graphics')
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
  const storage = cache.get('storage')
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
  const network = cache.get('network')
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
  const peripherals = cache.get('peripherals')
  if (peripherals?.kind === 'peripherals' && peripherals.peripherals.length) {
    lines.push('', '【外设】')
    for (const peripheral of peripherals.peripherals) {
      lines.push(`${peripheral.kind}: ${peripheral.name}${peripheral.manufacturer ? `（${peripheral.manufacturer}）` : ''}`)
    }
  }
  const publicIp = cache.get('public')
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

async function copyReport() {
  try {
    await navigator.clipboard.writeText(buildReport())
    copied.value = true
    setTimeout(() => {
      copied.value = false
    }, 1500)
  } catch (e: any) {
    summaryError.value = `复制失败: ${String(e?.message || e)}`
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
        <button type="button" class="btn btn-outline" :disabled="!summary" @click="copyReport">
          {{ copied ? '已复制 ✓' : '复制报告' }}
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

    <!-- 详细信息按页签惰性加载：切到哪个页签才查询哪个，之后缓存复用 -->
    <nav class="tabs" role="tablist">
      <button
        v-for="tab in TABS"
        :key="tab.key"
        type="button"
        role="tab"
        :aria-selected="activeTab === tab.key"
        class="tab"
        :class="{ active: activeTab === tab.key }"
        @click="switchTab(tab.key)"
      >
        {{ tab.label }}
      </button>
    </nav>

    <section class="card detail" role="tabpanel">
      <p v-if="activeError" class="error" role="alert">{{ activeError }}</p>
      <p v-else-if="activeLoading" class="empty">正在采集…</p>
      <p v-else-if="!activeSection" class="empty">暂无数据。</p>

      <!-- 设备与系统 -->
      <template v-else-if="activeTab === 'device' && activeSection.kind === 'device'">
        <dl>
          <div><dt>计算机名</dt><dd><code>{{ activeSection.device.hostname || '—' }}</code></dd></div>
          <div><dt>当前用户</dt><dd>{{ activeSection.device.username || '—' }}</dd></div>
          <div><dt>设备型号</dt><dd>{{ join(activeSection.device.manufacturer, activeSection.device.model) }}</dd></div>
          <div><dt>主板</dt><dd>{{ join(activeSection.device.boardManufacturer, activeSection.device.boardModel, activeSection.device.boardVersion) }}</dd></div>
          <div><dt>BIOS</dt><dd>{{ join(activeSection.device.biosVendor, activeSection.device.biosVersion, activeSection.device.biosDate) }}</dd></div>
          <div><dt>操作系统</dt><dd>{{ join(activeSection.device.osEdition, activeSection.device.cpuArch) }}</dd></div>
          <div><dt>系统版本</dt><dd>{{ join(activeSection.device.osVersion, activeSection.device.osDisplayVersion) }}（Build {{ activeSection.device.osBuild || '—' }}）</dd></div>
          <div><dt>系统安装时间</dt><dd>{{ formatTime(activeSection.device.osInstallTime) }}</dd></div>
          <div><dt>时区 / 语言</dt><dd>{{ activeSection.device.timezone || '—' }} / {{ browserLanguage }}</dd></div>
          <div v-if="activeSection.battery"><dt>电池</dt><dd>
            {{ activeSection.battery.chargePercent != null ? `${activeSection.battery.chargePercent}%` : '—' }}
            {{ activeSection.battery.state || '' }}
            <span v-if="activeSection.battery.runtimeMinutes">，约剩 {{ formatUptime(activeSection.battery.runtimeMinutes * 60) }}</span>
          </dd></div>
          <div><dt>本次开机时间</dt><dd>{{ formatTime(activeSection.device.bootTime) }}</dd></div>
          <div><dt>已运行</dt><dd>{{ formatUptime(activeSection.device.uptimeSecs) }}</dd></div>
        </dl>
      </template>

      <!-- 处理器 -->
      <template v-else-if="activeTab === 'cpu' && activeSection.kind === 'cpu'">
        <dl>
          <div><dt>型号</dt><dd>{{ activeSection.cpu.name || '—' }}</dd></div>
          <div><dt>厂商</dt><dd>{{ activeSection.cpu.vendor || '—' }}</dd></div>
          <div><dt>插槽</dt><dd>{{ activeSection.cpu.socket || '—' }}</dd></div>
          <div><dt>核心 / 线程</dt><dd>{{ activeSection.cpu.physicalCores ?? '—' }} 核 / {{ activeSection.cpu.logicalCores }} 线程</dd></div>
          <div><dt>基准频率</dt><dd>{{ formatFrequency(activeSection.cpu.baseFrequencyMhz) }}</dd></div>
          <div><dt>当前频率</dt><dd>{{ formatFrequency(activeSection.cpu.currentFrequencyMhz) }}</dd></div>
          <div><dt>L2 / L3 缓存</dt><dd>{{ formatCache(activeSection.cpu.l2CacheKb) }} / {{ formatCache(activeSection.cpu.l3CacheKb) }}</dd></div>
          <div><dt>使用率</dt><dd>{{ activeSection.cpu.usagePercent.toFixed(0) }}%</dd></div>
        </dl>
        <div v-if="activeSection.cpu.cores.length" class="cores">
          <div v-for="core in activeSection.cpu.cores" :key="core.name" class="core" :title="`${formatFrequency(core.frequencyMhz)}`">
            <span class="core-name">{{ core.name }}</span>
            <div class="bar"><div class="bar-fill" :style="{ width: `${Math.min(core.usagePercent, 100)}%` }" /></div>
            <span class="core-meta">{{ core.usagePercent.toFixed(0) }}% · {{ formatFrequency(core.frequencyMhz) }}</span>
          </div>
        </div>
      </template>

      <!-- 内存 -->
      <template v-else-if="activeTab === 'memory' && activeSection.kind === 'memory'">
        <div class="usage">
          <div class="usage-text">
            <span>已用 {{ formatBytes(activeSection.memory.usedBytes) }}</span>
            <span>共 {{ formatBytes(activeSection.memory.totalBytes) }}（{{ ((activeSection.memory.usedBytes / activeSection.memory.totalBytes) * 100).toFixed(0) }}%）</span>
          </div>
          <div class="bar"><div class="bar-fill" :style="{ width: `${Math.min((activeSection.memory.usedBytes / activeSection.memory.totalBytes) * 100, 100)}%` }" /></div>
          <p class="helper">可用 {{ formatBytes(activeSection.memory.freeBytes) }}<template v-if="activeSection.memory.swapTotalBytes">；交换分区已用 {{ formatBytes(activeSection.memory.swapUsedBytes) }} / {{ formatBytes(activeSection.memory.swapTotalBytes) }}</template></p>
        </div>
        <dl v-if="activeSection.memory.modules.length || activeSection.memory.slotCount">
          <div>
            <dt>内存条</dt>
            <dd>
              <span v-if="!activeSection.memory.modules.length">未读取到内存条信息</span>
              <span v-else-if="activeSection.memory.slotCount">已安装 {{ activeSection.memory.modules.length }} / {{ activeSection.memory.slotCount }} 条</span>
            </dd>
          </div>
          <div v-for="(module, index) in activeSection.memory.modules" :key="`${module.slot}-${index}`">
            <dt>{{ module.slot || module.bank || '内存条' }}</dt>
            <dd>{{ join(module.kind, formatBytes(module.capacityBytes), module.manufacturer, module.partNumber, module.configuredSpeedMhz || module.speedMhz ? `${module.configuredSpeedMhz || module.speedMhz} MHz` : '') }}</dd>
          </div>
        </dl>
      </template>

      <!-- 显卡 · 显示器 -->
      <template v-else-if="activeTab === 'graphics' && activeSection.kind === 'graphics'">
        <template v-if="activeSection.gpus.length">
          <h3>显卡</h3>
          <article v-for="gpu in activeSection.gpus" :key="gpu.name" class="item">
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
        <template v-if="activeSection.monitors.length">
          <h3>显示器</h3>
          <article v-for="(monitor, index) in activeSection.monitors" :key="`${monitor.name}-${index}`" class="item">
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
      <template v-else-if="activeTab === 'storage' && activeSection.kind === 'storage'">
        <h3>物理磁盘</h3>
        <template v-if="activeSection.drives.length">
          <article v-for="drive in activeSection.drives" :key="drive.serial || drive.model" class="item">
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
        <h3>存储卷</h3>
        <template v-if="activeSection.volumes.length">
          <article v-for="volume in activeSection.volumes" :key="volume.mount" class="item">
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
      <template v-else-if="activeTab === 'network' && activeSection.kind === 'network'">
        <template v-if="activeSection.networks.length">
          <article v-for="network in activeSection.networks" :key="network.mac || network.name" class="item">
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
      <template v-else-if="activeTab === 'peripherals' && activeSection.kind === 'peripherals'">
        <dl v-if="activeSection.peripherals.length">
          <div v-for="(peripheral, index) in activeSection.peripherals" :key="`${peripheral.name}-${index}`">
            <dt>{{ peripheral.kind }}</dt>
            <dd>{{ peripheral.name }}{{ peripheral.manufacturer ? `（${peripheral.manufacturer}）` : '' }}{{ peripheral.status ? ` · ${peripheral.status}` : '' }}</dd>
          </div>
        </dl>
        <p v-else class="empty">未识别到外设信息（台式机未接外设或系统不支持时为空）。</p>
      </template>

      <!-- 公网出口 -->
      <template v-else-if="activeTab === 'public' && activeSection && 'ip' in activeSection">
        <dl>
          <div><dt>公网 IP</dt><dd><code>{{ activeSection.ip }}</code></dd></div>
          <div><dt>归属地</dt><dd>{{ [activeSection.country, activeSection.region, activeSection.city].filter(Boolean).join(' ') || '—' }}</dd></div>
          <div v-if="activeSection.isp"><dt>运营商</dt><dd>{{ activeSection.isp }}</dd></div>
          <div v-if="activeSection.latitude != null && activeSection.longitude != null"><dt>定位</dt><dd>{{ activeSection.latitude.toFixed(4) }}, {{ activeSection.longitude.toFixed(4) }}（IP 估算，非 GPS）</dd></div>
          <div v-if="activeSection.timezone"><dt>网络时区</dt><dd>{{ activeSection.timezone }}</dd></div>
        </dl>
      </template>
    </section>
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
.card h3 { margin: 16px 0 12px; font-size: 15px; }
.card h3:first-child { margin-top: 0; }
.summary { margin-bottom: 12px; }
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
/* 页签栏 */
.tabs { display: flex; gap: 4px; flex-wrap: wrap; margin-bottom: 12px; }
.tab { min-height: 30px; padding: 4px 12px; border: 1px solid var(--border); border-radius: 999px; background: var(--card); color: var(--fg-muted); font-size: 13px; cursor: pointer; }
.tab:hover:not(:disabled) { border-color: var(--primary); color: var(--primary); }
.tab.active { border-color: var(--primary); background: color-mix(in srgb, var(--primary) 12%, transparent); color: var(--primary); }
.detail { min-height: 200px; }
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
