import { invoke } from '@tauri-apps/api/core'

export interface DeviceInfo {
  hostname?: string
  username?: string
  manufacturer?: string
  model?: string
  boardManufacturer?: string
  boardModel?: string
  boardVersion?: string
  biosVendor?: string
  biosVersion?: string
  biosDate?: string
  osName?: string
  osEdition?: string
  osVersion?: string
  osDisplayVersion?: string
  osBuild?: string
  osInstallTime?: number
  timezone?: string
  cpuArch?: string
  bootTime: number
  uptimeSecs: number
}

export interface CoreInfo {
  name: string
  usagePercent: number
  frequencyMhz: number
}

export interface CpuInfo {
  name: string
  vendor?: string
  socket?: string
  physicalCores?: number
  logicalCores: number
  baseFrequencyMhz: number
  currentFrequencyMhz: number
  l2CacheKb?: number
  l3CacheKb?: number
  usagePercent: number
  /** 每个逻辑核心的实时频率与使用率 */
  cores: CoreInfo[]
}

export interface MemoryModule {
  slot?: string
  bank?: string
  manufacturer?: string
  partNumber?: string
  capacityBytes?: number
  speedMhz?: number
  configuredSpeedMhz?: number
  kind?: string
}

export interface MemoryInfo {
  totalBytes: number
  usedBytes: number
  freeBytes: number
  swapTotalBytes: number
  swapUsedBytes: number
  slotCount?: number
  modules: MemoryModule[]
}

export interface GpuInfo {
  name: string
  vendor?: string
  driverVersion?: string
  memoryBytes?: number
  resolution?: string
  refreshRateHz?: number
  status?: string
}

export interface DriveInfo {
  model: string
  interface?: string
  mediaType?: string
  sizeBytes?: number
  serial?: string
  partitionCount?: number
}

export interface VolumeInfo {
  mount: string
  label?: string
  fileSystem?: string
  totalBytes?: number
  freeBytes?: number
  kind?: string
  /** 所属物理磁盘型号 */
  diskModel?: string
}

export interface NetworkInfo {
  name: string
  mac: string
  ipv4: string[]
  ipv6: string[]
  subnet: string[]
  gateway?: string
}

export interface MonitorInfo {
  name?: string
  /** 对角线尺寸（英寸） */
  sizeInches?: number
  resolution?: string
  refreshRateHz?: number
}

export interface BatteryInfo {
  chargePercent?: number
  state?: string
  /** 预计剩余可用时长（分钟） */
  runtimeMinutes?: number
}

export interface PeripheralInfo {
  kind: string
  name: string
  manufacturer?: string
  status?: string
}

export interface SummaryInfo {
  hostname?: string
  username?: string
  osEdition?: string
  cpuArch?: string
  cpuName: string
  cpuLogicalCores: number
  cpuUsagePercent: number
  memoryTotalBytes: number
  memoryUsedBytes: number
  ipv4: string[]
  bootTime: number
  uptimeSecs: number
}

/** 详细页签数据，kind 区分分支，各页签首次切到时才按需采集。 */
export type SectionData =
  | { kind: 'device'; device: DeviceInfo; battery?: BatteryInfo | null }
  | { kind: 'cpu'; cpu: CpuInfo }
  | { kind: 'memory'; memory: MemoryInfo }
  | { kind: 'graphics'; gpus: GpuInfo[]; monitors: MonitorInfo[] }
  | { kind: 'storage'; drives: DriveInfo[]; volumes: VolumeInfo[] }
  | { kind: 'network'; networks: NetworkInfo[] }
  | { kind: 'peripherals'; peripherals: PeripheralInfo[] }

export type SectionKey = 'device' | 'cpu' | 'memory' | 'graphics' | 'storage' | 'network' | 'peripherals'

export async function getSummary(): Promise<SummaryInfo> {
  return await invoke('system_info_summary')
}

export async function getSection(section: SectionKey): Promise<SectionData> {
  return await invoke('system_info_section', { section })
}

/** 公网出口信息，位置为 IP 库估算，非真实 GPS。 */
export interface PublicIpInfo {
  ip: string
  country?: string
  region?: string
  city?: string
  latitude?: number
  longitude?: number
  timezone?: string
  isp?: string
}

export async function getPublicIp(): Promise<PublicIpInfo> {
  return await invoke('system_public_ip')
}
