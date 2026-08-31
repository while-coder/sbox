//! 本机基础信息：操作系统、设备、主板/BIOS、CPU、内存、显卡、显示器、存储与网络适配器。
//! 静态规格用系统自带工具采集（Windows: PowerShell CIM，macOS: system_profiler，
//! Linux: DMI/lspci/lsblk/xrandr），实时用量（内存 / CPU 使用率）用 sysinfo。
//!
//! 采集按 section 拆分（summary / device / cpu / memory / graphics / storage / network /
//! peripherals），前端概要秒开，详细数据切到对应页签时才查询，避免一次性等待。

use serde::{Deserialize, Serialize};
use sysinfo::System;

/// 页面顶部概要：全部来自 sysinfo，秒级返回。
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SummaryInfo {
    pub hostname: Option<String>,
    pub username: Option<String>,
    /// 系统长版本名，如 "Windows 11 专业版"
    pub os_edition: Option<String>,
    pub cpu_arch: Option<String>,
    pub cpu_name: String,
    pub cpu_logical_cores: usize,
    pub cpu_usage_percent: f32,
    pub memory_total_bytes: u64,
    pub memory_used_bytes: u64,
    /// 本机 IPv4（带子网掩码）
    pub ipv4: Vec<String>,
    pub boot_time: u64,
    pub uptime_secs: u64,
}

/// 详细页签数据。tag "kind" 用于前端区分分支。
#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SectionData {
    Device {
        device: Box<DeviceInfo>,
        battery: Option<BatteryInfo>,
    },
    Cpu {
        cpu: Box<CpuInfo>,
    },
    Memory {
        memory: Box<MemoryInfo>,
    },
    Graphics {
        gpus: Vec<GpuInfo>,
        monitors: Vec<MonitorInfo>,
    },
    Storage {
        drives: Vec<DriveInfo>,
        volumes: Vec<VolumeInfo>,
    },
    Network {
        networks: Vec<NetworkInfo>,
    },
    Peripherals {
        peripherals: Vec<PeripheralInfo>,
    },
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Section {
    Device,
    Cpu,
    Memory,
    Graphics,
    Storage,
    Network,
    Peripherals,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub hostname: Option<String>,
    pub username: Option<String>,
    /// 整机厂商与型号（组装机为空，主板信息可参考）
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub board_manufacturer: Option<String>,
    pub board_model: Option<String>,
    pub board_version: Option<String>,
    pub bios_vendor: Option<String>,
    pub bios_version: Option<String>,
    pub bios_date: Option<String>,
    /// 系统名称，如 Windows / macOS
    pub os_name: Option<String>,
    /// 完整版本名，如 "Windows 11 专业版" / "macOS 15.5"
    pub os_edition: Option<String>,
    /// 系统大版本，如 "11" / "15.5"
    pub os_version: Option<String>,
    /// 功能更新代号，如 "24H2"（仅 Windows）
    pub os_display_version: Option<String>,
    /// 完整 build 号，如 "10.0.26200.835"
    pub os_build: Option<String>,
    /// 系统安装时间（Unix 秒，仅 Windows）
    pub os_install_time: Option<u64>,
    /// 系统时区，如 "China Standard Time" / "Asia/Shanghai"
    pub timezone: Option<String>,
    /// 系统架构，如 x86_64 / aarch64
    pub cpu_arch: Option<String>,
    /// 开机时间（Unix 秒）
    pub boot_time: u64,
    /// 已运行时长（秒）
    pub uptime_secs: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuInfo {
    /// 完整型号名，如 "12th Gen Intel(R) Core(TM) i7-12700"
    pub name: String,
    pub vendor: Option<String>,
    /// 插槽标识，如 "U3E1" / "AM4"
    pub socket: Option<String>,
    pub physical_cores: Option<usize>,
    pub logical_cores: usize,
    /// 标称频率（MHz）
    pub base_frequency_mhz: u64,
    /// 采样到的实时频率（MHz）
    pub current_frequency_mhz: u64,
    /// 二级缓存（KB）
    pub l2_cache_kb: Option<u64>,
    /// 三级缓存（KB）
    pub l3_cache_kb: Option<u64>,
    /// 采样得到的使用率（%）
    pub usage_percent: f32,
    /// 每个逻辑核心的实时频率与使用率
    pub cores: Vec<CoreInfo>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreInfo {
    /// 核心名，如 "CPU 0"
    pub name: String,
    pub usage_percent: f32,
    pub frequency_mhz: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
    /// 主板内存插槽总数
    pub slot_count: Option<u32>,
    /// 已安装的内存条
    pub modules: Vec<MemoryModule>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryModule {
    /// 插槽位置，如 "ChannelA-DIMM0"
    pub slot: Option<String>,
    pub bank: Option<String>,
    pub manufacturer: Option<String>,
    pub part_number: Option<String>,
    pub capacity_bytes: Option<u64>,
    /// 额定频率（MHz）
    pub speed_mhz: Option<u64>,
    /// 实际运行频率（MHz）
    pub configured_speed_mhz: Option<u64>,
    /// 内存代际，如 DDR4 / DDR5
    pub kind: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuInfo {
    pub name: String,
    pub vendor: Option<String>,
    pub driver_version: Option<String>,
    /// 显存大小；Windows 的 WMI 字段为 32 位，超过 4GB 时可能不准
    pub memory_bytes: Option<u64>,
    /// 当前输出分辨率，如 "2560 x 1440"
    pub resolution: Option<String>,
    pub refresh_rate_hz: Option<u32>,
    /// 设备状态，如 "OK"
    pub status: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveInfo {
    /// 磁盘型号
    pub model: String,
    /// 接口类型，如 NVMe / SATA / USB
    pub interface: Option<String>,
    pub media_type: Option<String>,
    pub size_bytes: Option<u64>,
    pub serial: Option<String>,
    pub partition_count: Option<u32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeInfo {
    /// 挂载点，如 "C:"
    pub mount: String,
    pub label: Option<String>,
    pub file_system: Option<String>,
    pub total_bytes: Option<u64>,
    pub free_bytes: Option<u64>,
    /// 卷类型：本地磁盘 / 可移动磁盘 / 网络磁盘
    pub kind: Option<String>,
    /// 所属物理磁盘型号
    pub disk_model: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInfo {
    pub name: String,
    pub mac: String,
    pub ipv4: Vec<String>,
    pub ipv6: Vec<String>,
    pub subnet: Vec<String>,
    pub gateway: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorInfo {
    /// 显示器型号，如 "DELL U2723QE"
    pub name: Option<String>,
    /// 对角线尺寸（英寸），由 EDID 物理尺寸换算
    pub size_inches: Option<f32>,
    /// 当前分辨率，如 "2560 x 1440"
    pub resolution: Option<String>,
    pub refresh_rate_hz: Option<u32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatteryInfo {
    /// 电量百分比
    pub charge_percent: Option<u32>,
    /// 充电状态，如 "已接电源" / "放电中" / "充电中"
    pub state: Option<String>,
    /// 预计剩余可用时长（分钟）
    pub runtime_minutes: Option<u64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PeripheralInfo {
    /// 设备类别：鼠标 / 键盘 / 蓝牙
    pub kind: String,
    pub name: String,
    pub manufacturer: Option<String>,
    pub status: Option<String>,
}

/// 公网出口信息，IP 定位服务返回（位置为 IP 估算，非真实 GPS）。
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicIpInfo {
    pub ip: String,
    pub country: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub timezone: Option<String>,
    /// 运营商 / 出口组织
    pub isp: Option<String>,
}

// ===== 命令 =====

#[tauri::command(rename_all = "camelCase")]
pub async fn system_info_summary() -> Result<SummaryInfo, String> {
    tauri::async_runtime::spawn_blocking(collect_summary)
        .await
        .map_err(|error| format!("采集概要信息失败: {error}"))?
}

#[tauri::command(rename_all = "camelCase")]
pub async fn system_info_section(section: Section) -> Result<SectionData, String> {
    tauri::async_runtime::spawn_blocking(move || collect_section(section))
        .await
        .map_err(|error| format!("采集本机信息失败: {error}"))?
}

/// 公网出口 IP 与归属地。位置为 IP 库估算，不是设备 GPS。
#[tauri::command(rename_all = "camelCase")]
pub async fn system_public_ip() -> Result<PublicIpInfo, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|error| format!("创建 HTTP 客户端失败: {error}"))?;
    match fetch_public_ip(&client, "https://ipwho.is/").await {
        Ok(info) => Ok(info),
        Err(_) => fetch_public_ip(&client, "https://ipapi.co/json/").await,
    }
}

async fn fetch_public_ip(client: &reqwest::Client, url: &str) -> Result<PublicIpInfo, String> {
    let value: serde_json::Value = client
        .get(url)
        .send()
        .await
        .map_err(|error| format!("请求公网 IP 失败: {error}"))?
        .json()
        .await
        .map_err(|error| format!("解析公网 IP 响应失败: {error}"))?;

    // ipwho.is 查询失败时仍返回 200，靠 success 字段区分，走备用服务
    if value.get("success") == Some(&serde_json::Value::Bool(false)) {
        return Err("定位服务未命中".into());
    }

    let text = |key: &str| {
        value
            .get(key)
            .and_then(serde_json::Value::as_str)
            .map(|text| text.trim().to_string())
            .filter(|text| !text.is_empty())
    };
    let number = |key: &str| value.get(key).and_then(serde_json::Value::as_f64);
    let ip = text("ip").filter(|ip| !ip.is_empty()).ok_or("响应缺少 IP 字段")?;
    let timezone = text("timezone").or_else(|| {
        value
            .get("timezone")
            .and_then(|tz| tz.get("id"))
            .and_then(serde_json::Value::as_str)
            .map(|text| text.to_string())
    });
    // ipwho.is 的运营商在 connection.isp / connection.org，ipapi.co 在 org
    let isp = value
        .get("connection")
        .and_then(|conn| conn.get("isp").or_else(|| conn.get("org")))
        .and_then(serde_json::Value::as_str)
        .map(|text| text.to_string())
        .or_else(|| text("org"));

    Ok(PublicIpInfo {
        ip,
        country: text("country").or_else(|| text("country_name")),
        region: text("region"),
        city: text("city"),
        latitude: number("latitude"),
        longitude: number("longitude"),
        timezone,
        isp,
    })
}

// ===== sysinfo 快速路径（概要与各 section 的实时数据） =====

/// sysinfo 需要两次采样间隔才能得到真实 CPU 使用率，统一在这里处理。
fn sample_system() -> System {
    let mut system = System::new();
    system.refresh_all();
    std::thread::sleep(std::time::Duration::from_millis(200));
    system.refresh_cpu_usage();
    system
}

fn collect_summary() -> Result<SummaryInfo, String> {
    let system = sample_system();
    let cpus = system.cpus();
    let ipv4 = sysinfo_networks()
        .iter()
        .flat_map(|network| network.ipv4.iter().cloned())
        .collect();
    Ok(SummaryInfo {
        hostname: System::host_name(),
        username: current_username(),
        os_edition: System::long_os_version(),
        cpu_arch: Some(System::cpu_arch()),
        cpu_name: cpus
            .first()
            .map(|cpu| cpu.brand().trim().to_string())
            .unwrap_or_default(),
        cpu_logical_cores: cpus.len(),
        cpu_usage_percent: system.global_cpu_usage(),
        memory_total_bytes: system.total_memory(),
        memory_used_bytes: system.used_memory(),
        ipv4,
        boot_time: System::boot_time(),
        uptime_secs: System::uptime(),
    })
}

fn current_username() -> Option<String> {
    std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .ok()
}

fn sysinfo_device() -> DeviceInfo {
    DeviceInfo {
        hostname: System::host_name(),
        username: current_username(),
        manufacturer: None,
        model: None,
        board_manufacturer: None,
        board_model: None,
        board_version: None,
        bios_vendor: None,
        bios_version: None,
        bios_date: None,
        os_name: System::name(),
        os_edition: System::long_os_version(),
        os_version: System::os_version(),
        os_display_version: None,
        os_build: System::kernel_version(),
        os_install_time: None,
        timezone: None,
        cpu_arch: Some(System::cpu_arch()),
        boot_time: System::boot_time(),
        uptime_secs: System::uptime(),
    }
}

fn sysinfo_cpu() -> CpuInfo {
    let system = sample_system();
    let cpus = system.cpus();
    CpuInfo {
        name: cpus
            .first()
            .map(|cpu| cpu.brand().trim().to_string())
            .unwrap_or_default(),
        vendor: None,
        socket: None,
        physical_cores: System::physical_core_count(),
        logical_cores: cpus.len(),
        base_frequency_mhz: cpus.first().map(|cpu| cpu.frequency()).unwrap_or(0),
        current_frequency_mhz: cpus.first().map(|cpu| cpu.frequency()).unwrap_or(0),
        l2_cache_kb: None,
        l3_cache_kb: None,
        usage_percent: system.global_cpu_usage(),
        cores: cpus
            .iter()
            .map(|cpu| CoreInfo {
                name: cpu.name().to_string(),
                usage_percent: cpu.cpu_usage(),
                frequency_mhz: cpu.frequency(),
            })
            .collect(),
    }
}

fn sysinfo_memory() -> MemoryInfo {
    let system = sample_system();
    MemoryInfo {
        total_bytes: system.total_memory(),
        used_bytes: system.used_memory(),
        free_bytes: system.free_memory(),
        swap_total_bytes: system.total_swap(),
        swap_used_bytes: system.used_swap(),
        slot_count: None,
        modules: Vec::new(),
    }
}

fn sysinfo_networks() -> Vec<NetworkInfo> {
    sysinfo::Networks::new_with_refreshed_list()
        .list()
        .iter()
        .map(|(name, data)| {
            let mut info = NetworkInfo {
                name: name.clone(),
                mac: data.mac_address().to_string(),
                ipv4: Vec::new(),
                ipv6: Vec::new(),
                subnet: Vec::new(),
                gateway: None,
            };
            for network in data.ip_networks() {
                if network.addr.is_ipv4() {
                    info.ipv4.push(network.to_string());
                } else {
                    info.ipv6.push(network.to_string());
                    info.subnet.push(format!("/{}", network.prefix));
                }
            }
            info
        })
        .collect()
}

fn sysinfo_volumes() -> Vec<VolumeInfo> {
    sysinfo::Disks::new_with_refreshed_list()
        .list()
        .iter()
        .map(|disk| VolumeInfo {
            mount: disk.mount_point().to_string_lossy().into_owned(),
            label: Some(disk.name().to_string_lossy().into_owned()),
            file_system: Some(disk.file_system().to_string_lossy().into_owned()),
            total_bytes: Some(disk.total_space()),
            free_bytes: Some(disk.available_space()),
            kind: Some(if disk.is_removable() { "可移动磁盘" } else { "本地磁盘" }.into()),
            disk_model: None,
        })
        .collect()
}

/// Unix 平台读取时区：/etc/localtime 软链里带 IANA 时区名（如 zoneinfo/Asia/Shanghai），失败时回退 TZ 环境变量。
#[cfg(any(target_os = "macos", target_os = "linux"))]
fn unix_timezone() -> Option<String> {
    if let Ok(target) = std::fs::read_link("/etc/localtime") {
        let target = target.to_string_lossy();
        if let Some(index) = target.find("zoneinfo/") {
            let zone = &target[index + "zoneinfo/".len()..];
            if !zone.is_empty() {
                return Some(zone.to_string());
            }
        }
    }
    std::env::var("TZ")
        .ok()
        .map(|tz| tz.trim_start_matches(':').to_string())
        .filter(|tz| !tz.is_empty())
}

// ===== section 分发 =====

#[cfg(windows)]
use windows_impl as platform;
#[cfg(target_os = "macos")]
use macos_impl as platform;
#[cfg(target_os = "linux")]
use linux_impl as platform;
#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
use other_impl as platform;

fn collect_section(section: Section) -> Result<SectionData, String> {
    let data = match section {
        Section::Device => {
            let (device, battery) = platform::device_section();
            SectionData::Device {
                device: Box::new(device),
                battery,
            }
        }
        Section::Cpu => SectionData::Cpu {
            cpu: Box::new(platform::cpu_section()),
        },
        Section::Memory => SectionData::Memory {
            memory: Box::new(platform::memory_section()),
        },
        Section::Graphics => {
            let (gpus, monitors) = platform::graphics_section();
            SectionData::Graphics { gpus, monitors }
        }
        Section::Storage => {
            let (drives, volumes) = platform::storage_section();
            SectionData::Storage { drives, volumes }
        }
        Section::Network => SectionData::Network {
            networks: platform::network_section(),
        },
        Section::Peripherals => SectionData::Peripherals {
            peripherals: platform::peripherals_section(),
        },
    };
    Ok(data)
}

#[cfg(windows)]
mod windows_impl {
    use super::{
        BatteryInfo, CpuInfo, DeviceInfo, DriveInfo, GpuInfo, MemoryInfo, MonitorInfo,
        NetworkInfo, PeripheralInfo, VolumeInfo,
    };
    use serde_json::Value;
    use std::os::windows::process::CommandExt;

    /// CIM 公共头：关闭错误即停（缺字段用空值兜底）、UTF-8 输出避免中文乱码。
    const HEADER: &str = "\
$ErrorActionPreference = 'SilentlyContinue'
[Console]::OutputEncoding = [Text.Encoding]::UTF8
";

    const DEVICE_SCRIPT: &str = "\
$os = Get-CimInstance Win32_OperatingSystem
$cs = Get-CimInstance Win32_ComputerSystem
$bb = @(Get-CimInstance Win32_BaseBoard)[0]
$bi = @(Get-CimInstance Win32_BIOS)[0]
$reg = Get-ItemProperty 'HKLM:\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion'
[pscustomobject]@{
  os = $os | Select-Object Caption, OSArchitecture, InstallDate
  displayVersion = $reg.DisplayVersion
  buildLabel = if ($reg.UBR) { '{0}.{1}' -f $reg.CurrentBuildNumber, $reg.UBR } else { [string]$reg.CurrentBuildNumber }
  timezone = (Get-TimeZone).Id
  machine = $cs | Select-Object Manufacturer, Model, SystemFamily
  board = $bb | Select-Object Manufacturer, Product, Version, SerialNumber
  bios = $bi | Select-Object Manufacturer, SMBIOSBIOSVersion, @{n='Date';e={ if ($_.ReleaseDate) { $_.ReleaseDate.ToString('yyyy-MM-dd') } }}
  battery = @(Get-CimInstance Win32_Battery)[0] | Select-Object EstimatedChargeRemaining, BatteryStatus, EstimatedRunTime
} | ConvertTo-Json -Compress -Depth 3";

    const CPU_SCRIPT: &str = "\
$cpu = @(Get-CimInstance Win32_Processor)[0]
[pscustomobject]@{ cpu = $cpu | Select-Object Name, Manufacturer, SocketDesignation, NumberOfCores, NumberOfLogicalProcessors, MaxClockSpeed, L2CacheSize, L3CacheSize } | ConvertTo-Json -Compress -Depth 3";

    const MEMORY_SCRIPT: &str = "\
[pscustomobject]@{
  memoryModules = @(Get-CimInstance Win32_PhysicalMemory) | Select-Object DeviceLocator, BankLabel, Manufacturer, PartNumber, Capacity, Speed, ConfiguredClockSpeed, SMBIOSMemoryType
  memorySlots = @(Get-CimInstance Win32_PhysicalMemoryArray | Measure-Object -Property MemoryDevices -Sum).Sum
} | ConvertTo-Json -Compress -Depth 3";

    const GRAPHICS_SCRIPT: &str = "\
# WMI 的 AdapterRAM 是 32 位字段，显存超过 4GB 会截断；注册表里的 HardwareInformation 是准确显存。
# 不同驱动类型不一：NVIDIA 存 Int64，Intel/AMD 多存 4 字节 Byte[]，需分别解析。
$gpuReg = @(Get-ChildItem 'HKLM:\\SYSTEM\\CurrentControlSet\\Control\\Class\\{4d36e968-e325-11ce-bfc1-08002be10318}') | ForEach-Object {
  $p = Get-ItemProperty $_.PSPath
  $raw = $p.'HardwareInformation.qwMemorySize'
  if (-not $raw) { $raw = $p.'HardwareInformation.MemorySize' }
  # ToUInt64 要求 8 字节，核显多为 4 字节二进制，需补零后转换
  $vram = if ($raw -is [byte[]]) {
    $bytes = [byte[]]$raw
    while ($bytes.Count -lt 8) { $bytes += [byte]0 }
    [BitConverter]::ToUInt64($bytes, 0)
  } elseif ($raw) { [uint64]$raw } else { $null }
  [pscustomobject]@{
    Name = $p.DriverDesc
    Vram = $vram
  }
}
# 显示器：WmiMonitorID 给型号（UInt16 数组需转字符），BasicDisplayParams 给物理尺寸（厘米）→ 换算英寸
$monitors = @(Get-CimInstance -Namespace root\\wmi -ClassName WmiMonitorID) | Where-Object { $_.Active } | ForEach-Object {
  $m = $_
  $disp = @(Get-CimInstance -Namespace root\\wmi -ClassName WmiMonitorBasicDisplayParams) | Where-Object { $_.InstanceName -eq $m.InstanceName } | Select-Object -First 1
  $friendly = -join ($m.UserFriendlyName | Where-Object { $_ -and $_ -gt 32 } | ForEach-Object { [char]$_ })
  $vendor = -join ($m.ManufacturerName | Where-Object { $_ -and $_ -gt 32 } | ForEach-Object { [char]$_ })
  $inches = if ($disp -and $disp.MaxHorizontalImageSize) { [math]::Round(([math]::Sqrt([math]::Pow($disp.MaxHorizontalImageSize, 2) + [math]::Pow($disp.MaxVerticalImageSize, 2))) / 2.54, 1) }
  [pscustomobject]@{ Name = if ($friendly) { $friendly } else { $vendor }; Inches = $inches }
}
[pscustomobject]@{
  gpus = @(Get-CimInstance Win32_VideoController) | ForEach-Object {
    $vram = ($gpuReg | Where-Object Name -eq $_.Name | Select-Object -First 1).Vram
    $_ | Select-Object Name, VideoProcessor, DriverVersion, @{n='Vram';e={ $vram }}, CurrentHorizontalResolution, CurrentVerticalResolution, CurrentRefreshRate, Status
  }
  monitors = $monitors
} | ConvertTo-Json -Compress -Depth 3";

    const STORAGE_SCRIPT: &str = "\
# 逻辑卷 → 分区 → 物理磁盘 的关联链，标明每个盘符属于哪块物理磁盘。
# 关联对象里只带键属性（DeviceID），型号要用完整查询建好映射再查。
$ldToPart = @(Get-CimInstance Win32_LogicalDiskToPartition)
$driveModels = @{}
foreach ($d in @(Get-CimInstance Win32_DiskDrive)) { $driveModels[$d.DeviceID] = $d.Model }
$partToDrive = @{}
foreach ($a in @(Get-CimInstance Win32_DiskDriveToDiskPartition)) { $partToDrive[$a.Dependent.DeviceID] = $driveModels[$a.Antecedent.DeviceID] }
$volumes = @(Get-CimInstance Win32_LogicalDisk) | ForEach-Object {
  $vol = $_
  $assoc = $ldToPart | Where-Object { $_.Dependent.DeviceID -eq $vol.DeviceID } | Select-Object -First 1
  $diskModel = if ($assoc) { $partToDrive[$assoc.Antecedent.DeviceID] }
  $vol | Select-Object DeviceID, VolumeName, FileSystem, Size, FreeSpace, DriveType, @{n='DiskModel';e={ $diskModel }}
}
[pscustomobject]@{
  drives = @(Get-CimInstance Win32_DiskDrive) | Select-Object Model, InterfaceType, MediaType, Size, SerialNumber, Partitions
  volumes = $volumes
} | ConvertTo-Json -Compress -Depth 3";

    const NETWORK_SCRIPT: &str = "\
[pscustomobject]@{ nics = @(Get-CimInstance Win32_NetworkAdapterConfiguration -Filter 'IPEnabled = True') | Select-Object Description, MACAddress, IPAddress, IPSubnet, DefaultIPGateway } | ConvertTo-Json -Compress -Depth 3";

    const PERIPHERALS_SCRIPT: &str = "\
[pscustomobject]@{ peripherals = @(Get-CimInstance Win32_PnPEntity -Filter \"PNPClass='Mouse' OR PNPClass='Keyboard' OR PNPClass='Bluetooth'\") | Select-Object Name, PNPClass, Manufacturer, Status } | ConvertTo-Json -Compress -Depth 3";

    /// 运行一段 CIM 查询脚本并解析 JSON，失败返回 None（页面相应字段显示为空，不报错）。
    fn run_cim(script: &str) -> Option<Value> {
        let full = format!("{HEADER}{script}");
        let output = std::process::Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &full])
            .creation_flags(0x0800_0000) // CREATE_NO_WINDOW，避免弹出控制台
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let text = String::from_utf8_lossy(&output.stdout);
        match serde_json::from_str::<Value>(text.trim()) {
            Ok(value) => Some(value),
            Err(error) => {
                log::warn!("解析 PowerShell 本机信息输出失败: {error}");
                None
            }
        }
    }

    pub fn device_section() -> (DeviceInfo, Option<BatteryInfo>) {
        let mut device = super::sysinfo_device();
        let Some(value) = run_cim(DEVICE_SCRIPT) else {
            return (device, None);
        };
        // 操作系统：CIM 的 Caption 带版本名（如 "Microsoft Windows 11 企业版"），比 sysinfo 更具体
        if let Some(os) = value.get("os") {
            if let Some(caption) = str_field(os, "Caption") {
                // 大版本号取 Caption 里第一段纯数字的词（"11"）
                device.os_version = caption
                    .split_whitespace()
                    .find(|word| word.chars().next().is_some_and(|c| c.is_ascii_digit()))
                    .map(|version| version.to_string());
                device.os_edition = Some(caption);
            }
            if let Some(arch) = str_field(os, "OSArchitecture") {
                device.cpu_arch = Some(arch);
            }
            device.os_install_time = str_field(os, "InstallDate").and_then(parse_cim_date);
        }
        device.os_display_version = str_field(&value, "displayVersion");
        // sysinfo 的 kernel_version 在 Windows 上是 build 号（如 26200），拼上 UBR 更完整
        if let Some(build) = str_field(&value, "buildLabel") {
            device.os_build = Some(format!("10.0.{build}"));
        }
        device.timezone = str_field(&value, "timezone");
        if let Some(machine) = value.get("machine") {
            device.manufacturer = str_field(machine, "Manufacturer");
            device.model = str_field(machine, "Model");
        }
        if let Some(board) = value.get("board") {
            device.board_manufacturer = str_field(board, "Manufacturer");
            device.board_model = str_field(board, "Product");
            device.board_version = str_field(board, "Version");
        }
        if let Some(bios) = value.get("bios") {
            device.bios_vendor = str_field(bios, "Manufacturer");
            device.bios_version = str_field(bios, "SMBIOSBIOSVersion");
            device.bios_date = str_field(bios, "Date");
        }
        let battery = value.get("battery").and_then(parse_battery);
        (device, battery)
    }

    pub fn cpu_section() -> CpuInfo {
        let mut cpu = super::sysinfo_cpu();
        if let Some(value) = run_cim(CPU_SCRIPT) {
            if let Some(cpu_value) = value.get("cpu") {
                if let Some(name) = str_field(cpu_value, "Name") {
                    cpu.name = name;
                }
                cpu.vendor = str_field(cpu_value, "Manufacturer");
                cpu.socket = str_field(cpu_value, "SocketDesignation");
                if let Some(cores) = u64_field(cpu_value, "NumberOfCores") {
                    cpu.physical_cores = Some(cores as usize);
                }
                if let Some(threads) = u64_field(cpu_value, "NumberOfLogicalProcessors") {
                    cpu.logical_cores = threads as usize;
                }
                if let Some(frequency) = u64_field(cpu_value, "MaxClockSpeed") {
                    cpu.base_frequency_mhz = frequency;
                }
                cpu.l2_cache_kb = u64_field(cpu_value, "L2CacheSize");
                cpu.l3_cache_kb = u64_field(cpu_value, "L3CacheSize");
            }
        }
        cpu
    }

    pub fn memory_section() -> MemoryInfo {
        let mut memory = super::sysinfo_memory();
        let Some(value) = run_cim(MEMORY_SCRIPT) else {
            return memory;
        };
        if let Some(modules) = as_items(value.get("memoryModules")) {
            memory.modules = modules
                .iter()
                .filter_map(|module| {
                    // Capacity 为 0 或缺失的插槽没有插内存条
                    let capacity = u64_field(module, "Capacity").filter(|capacity| *capacity > 0)?;
                    Some(super::MemoryModule {
                        slot: str_field(module, "DeviceLocator"),
                        bank: str_field(module, "BankLabel"),
                        manufacturer: str_field(module, "Manufacturer"),
                        part_number: str_field(module, "PartNumber"),
                        capacity_bytes: Some(capacity),
                        speed_mhz: u64_field(module, "Speed"),
                        configured_speed_mhz: u64_field(module, "ConfiguredClockSpeed"),
                        kind: memory_kind(u64_field(module, "SMBIOSMemoryType").unwrap_or(0)),
                    })
                })
                .collect();
        }
        memory.slot_count = u64_field(&value, "memorySlots").map(|slots| slots as u32);
        memory
    }

    pub fn graphics_section() -> (Vec<GpuInfo>, Vec<MonitorInfo>) {
        let Some(value) = run_cim(GRAPHICS_SCRIPT) else {
            return (Vec::new(), Vec::new());
        };
        let gpus: Vec<GpuInfo> = as_items(value.get("gpus"))
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| {
                        Some(GpuInfo {
                            name: str_field(item, "Name")?,
                            vendor: str_field(item, "VideoProcessor"),
                            driver_version: str_field(item, "DriverVersion"),
                            // Vram 来自注册表，比 WMI 的 32 位 AdapterRAM 准确
                            memory_bytes: u64_field(item, "Vram"),
                            resolution: match (
                                u64_field(item, "CurrentHorizontalResolution"),
                                u64_field(item, "CurrentVerticalResolution"),
                            ) {
                                (Some(width), Some(height)) => Some(format!("{width} × {height}")),
                                _ => None,
                            },
                            refresh_rate_hz: u64_field(item, "CurrentRefreshRate").map(|rate| rate as u32),
                            status: str_field(item, "Status"),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        let mut monitors = as_items(value.get("monitors"))
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| {
                        Some(MonitorInfo {
                            name: str_field(item, "Name"),
                            size_inches: item.get("Inches").and_then(Value::as_f64).map(|inches| inches as f32),
                            resolution: None,
                            refresh_rate_hz: None,
                        })
                    })
                    .collect::<Vec<MonitorInfo>>()
            })
            .unwrap_or_default();
        // 只有单一有效输出时，把 GPU 的当前分辨率/刷新率挂到显示器上；
        // 多屏多卡的对应关系 CIM 拿不到，宁可留空也不猜。
        let active_outputs: Vec<&GpuInfo> = gpus.iter().filter(|gpu| gpu.resolution.is_some()).collect();
        if active_outputs.len() == 1 {
            let output = active_outputs[0];
            for monitor in monitors.iter_mut() {
                monitor.resolution = output.resolution.clone();
                monitor.refresh_rate_hz = output.refresh_rate_hz;
            }
        }
        (gpus, monitors)
    }

    pub fn storage_section() -> (Vec<DriveInfo>, Vec<VolumeInfo>) {
        let Some(value) = run_cim(STORAGE_SCRIPT) else {
            return (Vec::new(), super::sysinfo_volumes());
        };
        let drives = as_items(value.get("drives"))
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| {
                        Some(DriveInfo {
                            model: str_field(item, "Model")?,
                            interface: str_field(item, "InterfaceType"),
                            media_type: str_field(item, "MediaType"),
                            size_bytes: u64_field(item, "Size"),
                            serial: str_field(item, "SerialNumber"),
                            partition_count: u64_field(item, "Partitions").map(|count| count as u32),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        let volumes = as_items(value.get("volumes"))
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| {
                        let mount = str_field(item, "DeviceID")?.trim_end_matches('\\').to_string();
                        Some(VolumeInfo {
                            mount,
                            label: str_field(item, "VolumeName"),
                            file_system: str_field(item, "FileSystem"),
                            total_bytes: u64_field(item, "Size"),
                            free_bytes: u64_field(item, "FreeSpace"),
                            kind: u64_field(item, "DriveType").map(drive_type_name).map(|kind| kind.to_string()),
                            disk_model: str_field(item, "DiskModel"),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        (drives, volumes)
    }

    pub fn network_section() -> Vec<NetworkInfo> {
        let Some(value) = run_cim(NETWORK_SCRIPT) else {
            return super::sysinfo_networks();
        };
        as_items(value.get("nics"))
            .map(|items| {
                // CIM 查询按适配器顺序返回；IP 是 {"地址","子网"} 并列数组，按下标配对
                items
                    .iter()
                    .map(|item| {
                        let (ipv4, ipv6, subnet) = pair_addresses(item);
                        NetworkInfo {
                            name: str_field(item, "Description").unwrap_or_default(),
                            mac: str_field(item, "MACAddress").unwrap_or_default(),
                            ipv4,
                            ipv6,
                            subnet,
                            gateway: arr_field(item, "DefaultIPGateway").into_iter().next(),
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn peripherals_section() -> Vec<PeripheralInfo> {
        let Some(value) = run_cim(PERIPHERALS_SCRIPT) else {
            return Vec::new();
        };
        as_items(value.get("peripherals"))
            .map(|items| {
                // 一个外设常会注册多个 HID 端点，按 类别+名称 去重
                let mut seen = std::collections::HashSet::new();
                items
                    .iter()
                    .filter_map(|item| {
                        let class = str_field(item, "PNPClass")?;
                        Some(PeripheralInfo {
                            kind: peripheral_kind(&class).to_string(),
                            name: str_field(item, "Name")?,
                            manufacturer: str_field(item, "Manufacturer"),
                            status: str_field(item, "Status"),
                        })
                    })
                    .filter(|peripheral| seen.insert((peripheral.kind.clone(), peripheral.name.clone())))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// SMBIOS 内存类型代码 → 代际名。
    fn memory_kind(kind: u64) -> Option<String> {
        let name = match kind {
            20 => "DDR",
            21 => "DDR2",
            24 => "DDR3",
            26 => "DDR4",
            34 => "DDR5",
            35 => "LPDDR5",
            _ => return None,
        };
        Some(name.into())
    }

    fn drive_type_name(kind: u64) -> &'static str {
        match kind {
            2 => "可移动磁盘",
            3 => "本地磁盘",
            4 => "网络磁盘",
            5 => "光驱",
            6 => "RAM 磁盘",
            _ => "其他",
        }
    }

    /// Win32_Battery.BatteryStatus 代码 → 状态名。
    fn battery_status_name(code: u64) -> &'static str {
        match code {
            1 => "放电中",
            2 => "已接电源",
            3 => "已充满",
            4 => "电量低",
            5 => "电量严重不足",
            6..=9 => "充电中",
            11 => "部分充电",
            _ => "未知",
        }
    }

    fn parse_battery(value: &Value) -> Option<BatteryInfo> {
        let charge_percent = u64_field(value, "EstimatedChargeRemaining").map(|percent| percent as u32);
        let state = u64_field(value, "BatteryStatus").map(battery_status_name).map(|state| state.to_string());
        // EstimatedRunTime 的 71582788 表示"未知"
        let runtime_minutes = u64_field(value, "EstimatedRunTime").filter(|minutes| *minutes > 0 && *minutes < 71582788);
        (charge_percent.is_some() || state.is_some()).then(|| BatteryInfo {
            charge_percent,
            state,
            runtime_minutes,
        })
    }

    fn peripheral_kind(class: &str) -> &'static str {
        match class {
            "Mouse" => "鼠标",
            "Keyboard" => "键盘",
            "Bluetooth" => "蓝牙",
            _ => "其他",
        }
    }

    /// Win32_NetworkAdapterConfiguration 的 IPAddress 与 IPSubnet 是同长度的并列数组，
    /// 需要按下标配对才能区分 IPv4 / IPv6 与各自的掩码。
    fn pair_addresses(item: &Value) -> (Vec<String>, Vec<String>, Vec<String>) {
        let addresses = arr_field(item, "IPAddress");
        let subnets = arr_field(item, "IPSubnet");
        let mut ipv4 = Vec::new();
        let mut ipv6 = Vec::new();
        let mut subnet = Vec::new();
        for (index, address) in addresses.iter().enumerate() {
            if address.contains(':') {
                ipv6.push(address.clone());
                subnet.push(subnets.get(index).cloned().unwrap_or_default());
            } else {
                ipv4.push(format!("{address}/{}", subnets.get(index).cloned().unwrap_or_default()));
            }
        }
        (ipv4, ipv6, subnet)
    }

    fn str_field(value: &Value, key: &str) -> Option<String> {
        let text = value.get(key)?.as_str()?.trim();
        (!text.is_empty()).then(|| text.to_string())
    }

    fn u64_field(value: &Value, key: &str) -> Option<u64> {
        value.get(key)?.as_u64()
    }

    fn arr_field(value: &Value, key: &str) -> Vec<String> {
        match value.get(key) {
            Some(Value::Array(items)) => items
                .iter()
                .filter_map(Value::as_str)
                .map(|text| text.to_string())
                .collect(),
            Some(Value::String(text)) => vec![text.clone()],
            _ => Vec::new(),
        }
    }

    /// ConvertTo-Json 对单元素集合输出对象而非数组，这里统一成数组。
    fn as_items(value: Option<&Value>) -> Option<Vec<&Value>> {
        match value? {
            Value::Array(items) => Some(items.iter().collect()),
            Value::Object(_) => Some(vec![value?]),
            _ => None,
        }
    }

    /// CIM 的日期序列化成形如 "/Date(1765176243000)/" 的毫秒时间戳。
    fn parse_cim_date(text: String) -> Option<u64> {
        let digits: String = text.chars().filter(|c| c.is_ascii_digit()).collect();
        digits.parse::<u64>().ok().map(|millis| millis / 1_000)
    }

    #[cfg(test)]
    mod tests {
        use super::super::{collect_section, collect_summary, Section, SectionData};

        #[test]
        fn summary_and_device_section_return_platform_detail() {
            let summary = collect_summary().expect("概要采集应成功");
            assert!(!summary.cpu_name.is_empty(), "CPU 型号不应为空");
            assert!(summary.memory_total_bytes > 0);

            let data = collect_section(Section::Device).expect("设备段采集应成功");
            let SectionData::Device { device, .. } = data else {
                panic!("设备段应返回 Device 分支");
            };
            let edition = device.os_edition.as_deref().unwrap_or_default();
            assert!(edition.contains("Windows"), "系统版本应来自 CIM Caption，实际: {edition}");
            assert!(device.os_build.is_some());
            assert!(device.timezone.is_some());
        }

        #[test]
        fn graphics_section_reports_vram() {
            let data = collect_section(Section::Graphics).expect("显卡段采集应成功");
            let SectionData::Graphics { gpus, .. } = data else {
                panic!("显卡段应返回 Graphics 分支");
            };
            assert!(!gpus.is_empty(), "至少应识别到一块显卡");
            // 本机 NVIDIA 独显的注册表显存应能读到（Int64 类型）
            assert!(
                gpus.iter().any(|gpu| gpu.memory_bytes.unwrap_or(0) > 4_000_000_000),
                "应至少有一块显卡显存超过 4GB（验证注册表显存读取）"
            );
        }
    }
}

#[cfg(target_os = "macos")]
mod macos_impl {
    use super::{
        BatteryInfo, CpuInfo, DeviceInfo, DriveInfo, GpuInfo, MemoryInfo, MonitorInfo, NetworkInfo,
        PeripheralInfo, VolumeInfo,
    };
    use serde_json::Value;

    pub fn device_section() -> (DeviceInfo, Option<BatteryInfo>) {
        let mut device = super::sysinfo_device();
        let value = run_profiler("SPHardwareDataType");
        if let Some(hardware) = value
            .get("SPHardwareDataType")
            .and_then(Value::as_array)
            .and_then(|items| items.first())
        {
            device.manufacturer = Some("Apple".into());
            device.model = str_field(hardware, "machine_model").or_else(|| str_field(hardware, "machine_name"));
            device.board_model = str_field(hardware, "machine_identifier");
            device.bios_version = str_field(hardware, "boot_rom_version");
            device.os_edition = str_field(hardware, "os_version");
        }
        device.timezone = super::unix_timezone();
        (device, parse_pmset_battery())
    }

    pub fn cpu_section() -> CpuInfo {
        let mut cpu = super::sysinfo_cpu();
        if let Some(hardware) = run_profiler("SPHardwareDataType")
            .get("SPHardwareDataType")
            .and_then(Value::as_array)
            .and_then(|items| items.first())
        {
            // Apple Silicon 用 chip_type（如 "Apple M1 Pro"），Intel Mac 用 cpu_type
            if let Some(name) = str_field(hardware, "chip_type").or_else(|| str_field(hardware, "cpu_type")) {
                cpu.name = name;
            }
            if let Some(cores) = u64_field(hardware, "physical_cpus") {
                cpu.physical_cores = Some(cores as usize);
            }
            if let Some(threads) = u64_field(hardware, "logical_cpus") {
                cpu.logical_cores = threads as usize;
            }
        }
        cpu
    }

    pub fn memory_section() -> MemoryInfo {
        let mut memory = super::sysinfo_memory();
        if let Some(modules) = run_profiler("SPMemoryDataType")
            .get("SPMemoryDataType")
            .and_then(Value::as_array)
        {
            memory.modules = modules
                .iter()
                .map(|module| super::MemoryModule {
                    slot: str_field(module, "_name").or_else(|| str_field(module, "device_locator")),
                    bank: str_field(module, "bank_label"),
                    manufacturer: str_field(module, "manufacturer"),
                    part_number: str_field(module, "part_number"),
                    capacity_bytes: u64_field(module, "size"),
                    speed_mhz: u64_field(module, "speed"),
                    configured_speed_mhz: None,
                    kind: str_field(module, "ddr_type").or_else(|| str_field(module, "memory_type")),
                })
                .collect();
        }
        memory
    }

    pub fn graphics_section() -> (Vec<GpuInfo>, Vec<MonitorInfo>) {
        let value = run_profiler("SPDisplaysDataType");
        let Some(items) = value.get("SPDisplaysDataType").and_then(Value::as_array) else {
            return (Vec::new(), Vec::new());
        };
        let gpus = items
            .iter()
            .filter_map(|item| {
                let name = str_field(item, "sppci_model").or_else(|| str_field(item, "_name"))?;
                Some(GpuInfo {
                    name,
                    vendor: str_field(item, "sppci_vendor"),
                    driver_version: None,
                    memory_bytes: item
                        .get("spdisplays_vram_shared")
                        .or_else(|| item.get("spdisplays_vram"))
                        .and_then(parse_vram),
                    resolution: item
                        .get("spdisplays_ndrvs")
                        .and_then(Value::as_array)
                        .and_then(|displays| displays.first())
                        .and_then(|display| str_field(display, "_resolution")),
                    refresh_rate_hz: None,
                    status: None,
                })
            })
            .collect();
        // 显示器：spdisplays_ndrvs 里每块屏给名称与分辨率（"5120 x 2880 @ 60Hz" 或 "5120 x 2880"）
        let monitors = items
            .iter()
            .filter_map(|item| item.get("spdisplays_ndrvs").and_then(Value::as_array))
            .flat_map(|displays| displays.iter())
            .filter_map(|display| {
                let name = str_field(display, "_name")?;
                let (resolution, refresh_rate_hz) = parse_display_resolution(display);
                Some(MonitorInfo {
                    name: Some(name),
                    // system_profiler 不提供物理对角尺寸
                    size_inches: None,
                    resolution,
                    refresh_rate_hz,
                })
            })
            .collect();
        (gpus, monitors)
    }

    pub fn storage_section() -> (Vec<DriveInfo>, Vec<VolumeInfo>) {
        let storage = run_profiler("SPStorageDataType SPNVMeDataType");
        let volumes = storage
            .get("SPStorageDataType")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .map(|item| VolumeInfo {
                        mount: str_field(item, "mount_point").unwrap_or_else(|| "/".into()),
                        label: str_field(item, "_name"),
                        file_system: str_field(item, "file_system"),
                        total_bytes: u64_field(item, "capacity_in_bytes"),
                        free_bytes: u64_field(item, "free_space_in_bytes"),
                        kind: Some("本地磁盘".into()),
                        disk_model: None,
                    })
                    .collect()
            })
            .unwrap_or_default();
        // NVMe 盘字段为小写（model / serial / capacity_in_bytes）
        let drives = storage
            .get("SPNVMeDataType")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| {
                        let model = str_field(item, "model")
                            .or_else(|| str_field(item, "device_name"))
                            .or_else(|| str_field(item, "_name"))?;
                        Some(DriveInfo {
                            model,
                            interface: Some("NVMe".into()),
                            media_type: Some("SSD".into()),
                            size_bytes: u64_field(item, "capacity_in_bytes").or_else(|| u64_field(item, "capacity")),
                            serial: str_field(item, "serial"),
                            partition_count: None,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        (drives, volumes)
    }

    pub fn network_section() -> Vec<NetworkInfo> {
        super::sysinfo_networks()
    }

    pub fn peripherals_section() -> Vec<PeripheralInfo> {
        // 蓝牙外设：device_connected 数组里的设备，类别取 minorClassOfDevice
        run_profiler("SPBluetoothDataType")
            .get("SPBluetoothDataType")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.get("device_connected").and_then(Value::as_array))
                    .flat_map(|devices| devices.iter())
                    .filter_map(|device| {
                        let name = str_field(device, "_name")?;
                        let kind = str_field(device, "device_minorClassOfDevice").unwrap_or_else(|| "蓝牙设备".into());
                        Some(PeripheralInfo {
                            kind,
                            name,
                            manufacturer: str_field(device, "device_manufacturer"),
                            status: Some("已连接".into()),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 运行 system_profiler 并解析 JSON。
    fn run_profiler(datatypes: &str) -> Value {
        let Ok(output) = std::process::Command::new("system_profiler")
            .args([datatypes, "-json"])
            .output()
        else {
            return Value::Null;
        };
        if !output.status.success() {
            return Value::Null;
        }
        serde_json::from_str::<Value>(String::from_utf8_lossy(&output.stdout).trim()).unwrap_or(Value::Null)
    }

    /// 解析 "5120 x 2880 @ 60Hz" / "5120 x 2880" 形式的显示器分辨率与刷新率。
    fn parse_display_resolution(display: &Value) -> (Option<String>, Option<u32>) {
        let raw = display
            .get("spdisplays_resolution")
            .or_else(|| display.get("_resolution"))
            .and_then(Value::as_str)
            .unwrap_or_default();
        if let Some((resolution, rate)) = raw.split_once('@') {
            let rate: u32 = rate
                .trim()
                .trim_end_matches("Hz")
                .trim_end_matches('.')
                .trim()
                .parse()
                .unwrap_or(0);
            (Some(resolution.trim().to_string()), (rate > 0).then_some(rate))
        } else if raw.is_empty() {
            // 兜底：_pixelsx / _pixelsy 给像素数
            match (u64_field(display, "_pixelsx"), u64_field(display, "_pixelsy")) {
                (Some(width), Some(height)) => (Some(format!("{width} × {height}")), None),
                _ => (None, None),
            }
        } else {
            (Some(raw.trim().to_string()), None)
        }
    }

    /// 解析 `pmset -g batt` 输出，如 "InternalBattery-0 (id=..)\t87%; discharging; 4:13 remaining"。
    fn parse_pmset_battery() -> Option<BatteryInfo> {
        let output = std::process::Command::new("pmset").args(["-g", "batt"]).output().ok()?;
        let text = String::from_utf8_lossy(&output.stdout);
        let line = text.lines().find(|line| line.contains('%'))?;
        let percent_index = line.find('%')?;
        // 取 % 前的连续数字
        let charge_percent: u32 = line[..percent_index]
            .trim_end_matches(|c: char| !c.is_ascii_digit())
            .rsplit(|c: char| !c.is_ascii_digit())
            .next()?
            .parse()
            .ok()?;
        // 第一段分号后的词是状态：discharging / charging / charged / "AC Power"
        let state = line.split(';').nth(1).map(|state| match state.trim().to_ascii_lowercase().as_str() {
            "discharging" => "放电中".to_string(),
            "charging" => "充电中".to_string(),
            "charged" | "ac power" => "已接电源".to_string(),
            other => other.to_string(),
        });
        // 剩余时长形如 "4:13 remaining"
        let runtime_minutes = line.split(';').nth(2).and_then(|part| {
            let part = part.trim().strip_suffix("remaining")?.trim();
            let (hours, minutes) = part.split_once(':')?;
            let hours: u64 = hours.trim().parse().ok()?;
            let minutes: u64 = minutes.trim().parse().ok()?;
            let total = hours * 60 + minutes;
            (total > 0).then_some(total)
        });
        Some(BatteryInfo {
            charge_percent: Some(charge_percent),
            state,
            runtime_minutes,
        })
    }

    /// system_profiler 的显存形如 "16 GB" / "512 MB"。
    fn parse_vram(value: &Value) -> Option<u64> {
        let text = value.as_str()?.trim();
        let (number, unit) = text.split_once(' ')?;
        let number: f64 = number.parse().ok()?;
        let multiplier = match unit.trim_start().trim_end_matches('.').to_ascii_uppercase().as_str() {
            "KB" => 1_024,
            "MB" => 1_024 * 1_024,
            "GB" => 1_024 * 1_024 * 1_024,
            _ => return None,
        };
        Some((number * multiplier as f64) as u64)
    }

    fn str_field(value: &Value, key: &str) -> Option<String> {
        let text = value.get(key)?.as_str()?.trim();
        (!text.is_empty()).then(|| text.to_string())
    }

    fn u64_field(value: &Value, key: &str) -> Option<u64> {
        value.get(key)?.as_u64()
    }
}

#[cfg(target_os = "linux")]
mod linux_impl {
    use super::{
        BatteryInfo, CpuInfo, DeviceInfo, DriveInfo, GpuInfo, MemoryInfo, MonitorInfo, NetworkInfo,
        PeripheralInfo, VolumeInfo,
    };
    use serde_json::Value;
    use std::path::{Path, PathBuf};

    /// 读取 DMI（/sys/class/dmi/id）拿整机 / 主板 / BIOS 信息，普通用户也可读。
    fn read_dmi(key: &str) -> Option<String> {
        let text = std::fs::read_to_string(Path::new("/sys/class/dmi/id").join(key)).ok()?;
        let text = text.trim().to_string();
        // DMI 里常见 "System Product Name"、"To Be Filled By O.E.M." 之类的占位值，视为未提供
        let placeholder = text.is_empty()
            || text.eq_ignore_ascii_case("System Product Name")
            || text.eq_ignore_ascii_case("System Version")
            || text.to_ascii_lowercase().starts_with("to be filled");
        (!placeholder).then_some(text)
    }

    pub fn device_section() -> (DeviceInfo, Option<BatteryInfo>) {
        let mut device = super::sysinfo_device();
        device.manufacturer = read_dmi("sys_vendor").or(device.manufacturer);
        device.model = read_dmi("product_name").or(device.model);
        device.board_manufacturer = read_dmi("board_vendor");
        device.board_model = read_dmi("board_name");
        device.board_version = read_dmi("board_version");
        device.bios_vendor = read_dmi("bios_vendor");
        device.bios_version = read_dmi("bios_version");
        device.bios_date = read_dmi("bios_date");
        device.timezone = super::unix_timezone();
        (device, read_battery())
    }

    pub fn cpu_section() -> CpuInfo {
        super::sysinfo_cpu()
    }

    pub fn memory_section() -> MemoryInfo {
        super::sysinfo_memory()
    }

    pub fn graphics_section() -> (Vec<GpuInfo>, Vec<MonitorInfo>) {
        (list_gpus_from_lspci(), list_monitors_from_xrandr())
    }

    pub fn storage_section() -> (Vec<DriveInfo>, Vec<VolumeInfo>) {
        (list_drives_from_lsblk(), filter_real_volumes(super::sysinfo_volumes()))
    }

    pub fn network_section() -> Vec<NetworkInfo> {
        super::sysinfo_networks()
    }

    pub fn peripherals_section() -> Vec<PeripheralInfo> {
        Vec::new()
    }

    /// 显卡：lspci 过滤 VGA / 3D / Display 控制器（未安装 lspci 时留空，不报错）。
    fn list_gpus_from_lspci() -> Vec<GpuInfo> {
        let Ok(output) = std::process::Command::new("lspci").arg("-nn").output() else {
            return Vec::new();
        };
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|line| {
                line.contains("VGA compatible controller")
                    || line.contains("3D controller")
                    || line.contains("Display controller")
            })
            .filter_map(|line| {
                // 形如 "00:02.0 VGA compatible controller [0300]: Intel UHD Graphics [8086:46a6]"
                let name = line.split("]: ").nth(1)?;
                // 去掉末尾的 [厂商:设备] ID
                let name = name
                    .trim_end_matches(']')
                    .rsplit_once(" [")
                    .map(|(before, _)| before)
                    .unwrap_or(name)
                    .trim()
                    .to_string();
                (!name.is_empty()).then(|| GpuInfo {
                    name,
                    vendor: None,
                    driver_version: None,
                    memory_bytes: None,
                    resolution: None,
                    refresh_rate_hz: None,
                    status: None,
                })
            })
            .collect()
    }

    /// 物理磁盘：lsblk JSON 输出，过滤 type=disk 的顶层设备。
    fn list_drives_from_lsblk() -> Vec<DriveInfo> {
        let Ok(output) = std::process::Command::new("lsblk")
            .args(["--json", "-b", "-o", "NAME,MODEL,SIZE,SERIAL,TRAN,TYPE,ROTA"])
            .output()
        else {
            return Vec::new();
        };
        if !output.status.success() {
            return Vec::new();
        }
        let Ok(value) = serde_json::from_str::<Value>(String::from_utf8_lossy(&output.stdout).trim()) else {
            return Vec::new();
        };
        value
            .get("blockdevices")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter(|item| item.get("type").and_then(Value::as_str) == Some("disk"))
                    .map(|item| DriveInfo {
                        model: item
                            .get("model")
                            .and_then(Value::as_str)
                            .filter(|model| !model.trim().is_empty())
                            .unwrap_or(item.get("name").and_then(Value::as_str).unwrap_or("unknown disk"))
                            .trim()
                            .to_string(),
                        interface: item
                            .get("tran")
                            .and_then(Value::as_str)
                            .map(|tran| tran.to_ascii_uppercase()),
                        media_type: item
                            .get("rota")
                            .and_then(Value::as_bool)
                            .map(|rotational| if rotational { "HDD" } else { "SSD" })
                            .map(|kind| kind.to_string()),
                        size_bytes: u64_field(item, "size"),
                        serial: item
                            .get("serial")
                            .and_then(Value::as_str)
                            .map(|serial| serial.trim().to_string())
                            .filter(|serial| !serial.is_empty()),
                        partition_count: None,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 显示器：xrandr 解析（仅 X11 会话可用，Wayland 下留空）。
    /// 形如 "eDP-1 connected primary 1920x1080+0+0 ... 344mm x 194mm"，模式行里 "*" 标记当前刷新率。
    fn list_monitors_from_xrandr() -> Vec<MonitorInfo> {
        let Ok(output) = std::process::Command::new("xrandr").arg("--current").output() else {
            return Vec::new();
        };
        let mut monitors = Vec::new();
        let mut current: Option<MonitorInfo> = None;
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            if !line.starts_with(' ') {
                // 新的输出口行
                if let Some(monitor) = current.take() {
                    monitors.push(monitor);
                }
                let Some((name_part, rest)) = line.split_once(' ') else { continue };
                if !rest.contains("connected") {
                    continue;
                }
                // 物理尺寸在行尾 "344mm x 194mm"
                let size_inches = rest
                    .rsplit_once("mm x ")
                    .and_then(|(width_part, height_part)| {
                        let height: f64 = height_part.trim_end_matches("mm").trim().parse().ok()?;
                        let width: f64 = width_part.rsplit(' ').next()?.trim().parse().ok()?;
                        Some(((width * width + height * height).sqrt() / 25.4 * 10.0).round() / 10.0)
                    });
                let resolution = rest
                    .split_whitespace()
                    .find(|part| part.matches('x').count() == 1 && part.split('x').all(|n| n.parse::<u32>().is_ok()))
                    .map(|part| part.replace('x', " × "));
                current = Some(MonitorInfo {
                    name: Some(name_part.trim().to_string()),
                    size_inches: size_inches.map(|inches| inches as f32),
                    resolution,
                    refresh_rate_hz: None,
                });
            } else if line.contains('*') {
                // 当前模式行，如 "   1920x1080     60.05*+"
                if let Some(monitor) = current.as_mut() {
                    monitor.refresh_rate_hz = line
                        .split_whitespace()
                        .find(|mode| mode.contains('*'))
                        .and_then(|mode| mode.split('*').next())
                        .and_then(|rate| rate.parse::<f64>().ok())
                        .map(|rate| rate.round() as u32);
                }
            }
        }
        if let Some(monitor) = current.take() {
            monitors.push(monitor);
        }
        monitors
    }

    /// 电池：/sys/class/power_supply/BAT*/，无电池的台式机目录不存在 → None。
    fn read_battery() -> Option<BatteryInfo> {
        let base = PathBuf::from("/sys/class/power_supply");
        let mut battery_dir = None;
        let entries = std::fs::read_dir(&base).ok()?;
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_ascii_uppercase();
            if name.starts_with("BAT") {
                battery_dir = Some(entry.path());
                break;
            }
        }
        let dir = battery_dir?;
        let read = |name: &str| std::fs::read_to_string(dir.join(name)).ok().map(|text| text.trim().to_string());
        let charge_percent = read("capacity").and_then(|text| text.parse().ok());
        let state = read("status").map(|status| match status.as_str() {
            "Discharging" => "放电中".into(),
            "Charging" => "充电中".into(),
            "Full" => "已充满".into(),
            "Not charging" => "已接电源".into(),
            other => other.to_string(),
        });
        (charge_percent.is_some() || state.is_some()).then(|| BatteryInfo {
            charge_percent,
            state,
            runtime_minutes: None,
        })
    }

    /// sysinfo 的卷来自挂载点，会把 squashfs / bind mount / docker overlay 也列进来，
    /// 这里过滤掉虚拟文件系统与重复挂载点，只保留真实块设备卷。
    fn filter_real_volumes(volumes: Vec<VolumeInfo>) -> Vec<VolumeInfo> {
        let mut seen_mounts = std::collections::HashSet::new();
        volumes
            .into_iter()
            .filter(|volume| {
                let is_virtual = volume.file_system.as_deref().is_some_and(|fs| {
                    matches!(fs, "squashfs" | "tmpfs" | "overlay" | "proc" | "sysfs" | "devtmpfs" | "devfs" | "iso9660")
                });
                !is_virtual && seen_mounts.insert(volume.mount.clone())
            })
            .collect()
    }

    fn u64_field(value: &Value, key: &str) -> Option<u64> {
        value.get(key)?.as_u64()
    }
}

/// 未支持的平台（如移动端）：只提供 sysinfo 能拿到的部分。
#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
mod other_impl {
    use super::{CpuInfo, DeviceInfo, DriveInfo, GpuInfo, MemoryInfo, MonitorInfo, NetworkInfo, PeripheralInfo, VolumeInfo};

    pub fn device_section() -> (DeviceInfo, Option<super::BatteryInfo>) {
        (super::sysinfo_device(), None)
    }

    pub fn cpu_section() -> CpuInfo {
        super::sysinfo_cpu()
    }

    pub fn memory_section() -> MemoryInfo {
        super::sysinfo_memory()
    }

    pub fn graphics_section() -> (Vec<GpuInfo>, Vec<MonitorInfo>) {
        (Vec::new(), Vec::new())
    }

    pub fn storage_section() -> (Vec<DriveInfo>, Vec<VolumeInfo>) {
        (Vec::new(), super::sysinfo_volumes())
    }

    pub fn network_section() -> Vec<NetworkInfo> {
        super::sysinfo_networks()
    }

    pub fn peripherals_section() -> Vec<PeripheralInfo> {
        Vec::new()
    }
}
