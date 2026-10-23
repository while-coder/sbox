import { invoke } from '@tauri-apps/api/core'
import type { DeviceInfo, CoreInfo, CpuInfo, MemoryModule, MemorySlot, MemoryInfo, GpuInfo, DriveInfo, VolumeInfo, BoardSlotInfo, NetworkInfo, MonitorInfo, BatteryInfo, PeripheralInfo, SummaryInfo, SectionData, SectionKey, PublicIpInfo } from '@sbox/ui/tools/account/system-info/native'

export async function getSummary(): Promise<SummaryInfo> {
  return await invoke('system_info_summary')
}

export async function getSection(section: SectionKey): Promise<SectionData> {
  return await invoke('system_info_section', { section })
}

export async function getPublicIp(): Promise<PublicIpInfo> {
  return await invoke('system_public_ip')
}
