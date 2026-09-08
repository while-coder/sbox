use serde::Serialize;
use sysinfo::{Pid, ProcessesToUpdate, System};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortSocket {
    pub protocol: String,
    pub local_address: String,
    pub state: String,
    pub pid: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortProcess {
    pub pid: u32,
    pub process_name: String,
    pub executable_path: Option<String>,
    pub sockets: Vec<PortSocket>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortCheckResult {
    pub port: u16,
    pub processes: Vec<PortProcess>,
}

#[tauri::command]
pub async fn port_check_list(port: u16) -> Result<PortCheckResult, String> {
    tauri::async_runtime::spawn_blocking(move || list_port(port))
        .await
        .map_err(|error| format!("检查任务失败: {error}"))?
}

#[tauri::command]
pub async fn port_check_kill(pid: u32) -> Result<(), String> {
    if pid == 0 {
        return Err("无法结束系统保留进程".into());
    }
    if pid == std::process::id() {
        return Err("不能结束 sbox 自身进程".into());
    }
    tauri::async_runtime::spawn_blocking(move || kill_process(pid))
        .await
        .map_err(|error| format!("结束进程任务失败: {error}"))?
}

fn list_port(port: u16) -> Result<PortCheckResult, String> {
    let sockets = collect_sockets(port)?;

    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::All, true);

    // 保持“先发现的服务端套接字在前”的顺序，同时把同一进程的多个套接字合并到一张卡片。
    let mut order: Vec<u32> = Vec::new();
    let mut grouped: std::collections::HashMap<u32, Vec<PortSocket>> = std::collections::HashMap::new();
    for socket in sockets {
        let group = grouped.entry(socket.pid).or_insert_with(|| {
            order.push(socket.pid);
            Vec::new()
        });
        group.push(socket);
    }

    let processes = order
        .into_iter()
        .map(|pid| {
            let process = system.process(Pid::from_u32(pid));
            PortProcess {
                pid,
                process_name: process
                    .map(|process| process.name().to_string_lossy().into_owned())
                    .filter(|name| !name.is_empty())
                    .unwrap_or_else(|| "未知进程".into()),
                executable_path: process.and_then(|process| process.exe()).map(|path| {
                    path.to_string_lossy().into_owned()
                }),
                sockets: grouped.remove(&pid).unwrap_or_default(),
            }
        })
        .collect();

    Ok(PortCheckResult { port, processes })
}

fn kill_process(pid: u32) -> Result<(), String> {
    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::Some(&[Pid::from_u32(pid)]), true);
    let Some(process) = system.process(Pid::from_u32(pid)) else {
        return Err(format!("进程 {pid} 不存在或已退出"));
    };
    if !process.kill() {
        return Err(format!("结束进程 {pid} 失败，可能需要管理员权限"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_port_finds_own_listener() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let result = list_port(port).unwrap();
        let found = result.processes.iter().any(|process| {
            process.pid == std::process::id()
                && process
                    .sockets
                    .iter()
                    .any(|socket| socket.state == "LISTEN" && socket.protocol == "TCP")
        });
        assert!(found, "应能通过系统表找到本测试进程的监听套接字");
    }
}

#[cfg(windows)]
fn collect_sockets(port: u16) -> Result<Vec<PortSocket>, String> {
    windows_impl::collect(port)
}

#[cfg(windows)]
mod windows_impl {
    use super::PortSocket;
    use std::net::Ipv6Addr;
    use windows::Win32::NetworkManagement::IpHelper::{
        GetExtendedTcpTable, GetExtendedUdpTable, MIB_TCP6ROW_OWNER_PID, MIB_TCPROW_OWNER_PID,
        MIB_UDP6ROW_OWNER_PID, MIB_UDPROW_OWNER_PID, TCP_TABLE_CLASS, TCP_TABLE_OWNER_PID_ALL,
        UDP_TABLE_CLASS, UDP_TABLE_OWNER_PID,
    };

    // GetExtended*Table 先返回需要的缓冲区大小，再填充一次，避免与系统表变化竞态时反复重试。
    fn tcp_table(af: u32, class: TCP_TABLE_CLASS) -> Result<Vec<u8>, String> {
        let mut size = 0_u32;
        let status = unsafe { GetExtendedTcpTable(None, &mut size, false, af, class, 0) };
        ensure_buffer(status, size, "TCP")?;
        let mut buffer = vec![0_u8; size as usize];
        let status = unsafe {
            GetExtendedTcpTable(
                Some(buffer.as_mut_ptr().cast()),
                &mut size,
                false,
                af,
                class,
                0,
            )
        };
        ensure_status(status, "TCP")?;
        buffer.truncate(size as usize);
        Ok(buffer)
    }

    fn udp_table(af: u32, class: UDP_TABLE_CLASS) -> Result<Vec<u8>, String> {
        let mut size = 0_u32;
        let status = unsafe { GetExtendedUdpTable(None, &mut size, false, af, class, 0) };
        ensure_buffer(status, size, "UDP")?;
        let mut buffer = vec![0_u8; size as usize];
        let status = unsafe {
            GetExtendedUdpTable(
                Some(buffer.as_mut_ptr().cast()),
                &mut size,
                false,
                af,
                class,
                0,
            )
        };
        ensure_status(status, "UDP")?;
        buffer.truncate(size as usize);
        Ok(buffer)
    }

    fn ensure_buffer(status: u32, size: u32, name: &str) -> Result<(), String> {
        if status == 122 {
            // ERROR_INSUFFICIENT_BUFFER：首次调用预期结果，size 已填入所需大小。
            if size == 0 {
                return Err(format!("查询 {name} 表失败：系统未返回缓冲区大小"));
            }
            Ok(())
        } else {
            Err(format!("查询 {name} 表失败（Windows 错误 {status}）"))
        }
    }

    fn ensure_status(status: u32, name: &str) -> Result<(), String> {
        if status == 0 {
            Ok(())
        } else {
            Err(format!("查询 {name} 表失败（Windows 错误 {status}）"))
        }
    }

    fn read_entries<T: Copy>(buffer: &[u8]) -> Vec<T> {
        if buffer.len() < 4 {
            return Vec::new();
        }
        let count = u32::from_le_bytes(buffer[0..4].try_into().unwrap()) as usize;
        let base = unsafe { buffer.as_ptr().add(4) }.cast::<T>();
        (0..count)
            .map(|index| unsafe { std::ptr::read_unaligned(base.add(index)) })
            .collect()
    }

    fn local_port(field: u32) -> u16 {
        // MSDN：端口保存在低 16 位且为网络字节序。
        u16::from_be((field & 0xFFFF) as u16)
    }

    fn format_v4(address: u32, port: u16) -> String {
        let [a, b, c, d] = address.to_be_bytes();
        format!("{a}.{b}.{c}.{d}:{port}")
    }

    fn format_v6(address: [u8; 16], port: u16) -> String {
        format!("[{}]:{port}", Ipv6Addr::from(address))
    }

    fn tcp_state_name(state: u32) -> &'static str {
        use windows::Win32::NetworkManagement::IpHelper::*;
        match MIB_TCP_STATE(state as i32) {
            MIB_TCP_STATE_CLOSED => "CLOSED",
            MIB_TCP_STATE_LISTEN => "LISTEN",
            MIB_TCP_STATE_SYN_SENT => "SYN_SENT",
            MIB_TCP_STATE_SYN_RCVD => "SYN_RCVD",
            MIB_TCP_STATE_ESTAB => "ESTABLISHED",
            MIB_TCP_STATE_FIN_WAIT1 => "FIN_WAIT_1",
            MIB_TCP_STATE_FIN_WAIT2 => "FIN_WAIT_2",
            MIB_TCP_STATE_CLOSE_WAIT => "CLOSE_WAIT",
            MIB_TCP_STATE_CLOSING => "CLOSING",
            MIB_TCP_STATE_LAST_ACK => "LAST_ACK",
            MIB_TCP_STATE_TIME_WAIT => "TIME_WAIT",
            MIB_TCP_STATE_DELETE_TCB => "DELETE_TCB",
            _ => "UNKNOWN",
        }
    }

    pub fn collect(port: u16) -> Result<Vec<PortSocket>, String> {
        let mut sockets = Vec::new();
        // 2 = AF_INET，23 = AF_INET6。同一表类配合 AF_INET6 即返回 v6 行（MIB_TCP6TABLE 等）。
        for af in [2_u32, 23_u32] {
            let tcp_rows: Vec<MIB_TCPROW_OWNER_PID> =
                read_entries(&tcp_table(af, TCP_TABLE_OWNER_PID_ALL)?);
            for row in tcp_rows {
                if local_port(row.dwLocalPort) == port {
                    sockets.push(PortSocket {
                        protocol: "TCP".into(),
                        local_address: format_v4(row.dwLocalAddr, port),
                        state: tcp_state_name(row.dwState).into(),
                        pid: row.dwOwningPid,
                    });
                }
            }

            let tcp6_rows: Vec<MIB_TCP6ROW_OWNER_PID> =
                read_entries(&tcp_table(af, TCP_TABLE_OWNER_PID_ALL)?);
            for row in tcp6_rows {
                if local_port(row.dwLocalPort) == port {
                    sockets.push(PortSocket {
                        protocol: "TCP".into(),
                        local_address: format_v6(row.ucLocalAddr, port),
                        state: tcp_state_name(row.dwState).into(),
                        pid: row.dwOwningPid,
                    });
                }
            }

            let udp_rows: Vec<MIB_UDPROW_OWNER_PID> =
                read_entries(&udp_table(af, UDP_TABLE_OWNER_PID)?);
            for row in udp_rows {
                if local_port(row.dwLocalPort) == port {
                    sockets.push(PortSocket {
                        protocol: "UDP".into(),
                        local_address: format_v4(row.dwLocalAddr, port),
                        state: "UDP".into(),
                        pid: row.dwOwningPid,
                    });
                }
            }

            let udp6_rows: Vec<MIB_UDP6ROW_OWNER_PID> =
                read_entries(&udp_table(af, UDP_TABLE_OWNER_PID)?);
            for row in udp6_rows {
                if local_port(row.dwLocalPort) == port {
                    sockets.push(PortSocket {
                        protocol: "UDP".into(),
                        local_address: format_v6(row.ucLocalAddr, port),
                        state: "UDP".into(),
                        pid: row.dwOwningPid,
                    });
                }
            }
        }
        Ok(sockets)
    }

    #[cfg(test)]
    mod tests {
        use super::*;


        #[test]
        fn local_port_decodes_network_byte_order() {
            // 端口存储在网络字节序低 16 位：8080（0x1F90）呈现为 0x0000901F。
            assert_eq!(local_port(0x0000_901F), 8080);
            assert_eq!(local_port(0x0000_5000), 80);
        }

        #[test]
        fn v4_and_v6_formatting() {
            assert_eq!(format_v4(0x7F00_0001, 80), "127.0.0.1:80");
            assert_eq!(format_v6([0_u8; 16], 443), "[::]:443");
        }
    }
}

#[cfg(target_os = "linux")]
fn collect_sockets(port: u16) -> Result<Vec<PortSocket>, String> {
    linux_impl::collect(port)
}

#[cfg(target_os = "linux")]
mod linux_impl {
    use super::PortSocket;
    use std::collections::HashMap;
    use std::net::{Ipv4Addr, Ipv6Addr};
    use std::path::Path;

    struct Entry {
        inode: u64,
        socket: PortSocket,
    }

    pub fn collect(port: u16) -> Result<Vec<PortSocket>, String> {
        let mut entries = Vec::new();
        parse_proc_file("/proc/net/tcp", "TCP", port, &mut entries)?;
        parse_proc_file("/proc/net/tcp6", "TCP", port, &mut entries)?;
        parse_proc_file("/proc/net/udp", "UDP", port, &mut entries)?;
        parse_proc_file("/proc/net/udp6", "UDP", port, &mut entries)?;

        // 套接字只记录 inode，需要扫描各进程的 fd 找到持有者。
        let mut inode_pids: HashMap<u64, u32> = HashMap::new();
        let proc_dir = Path::new("/proc");
        let Ok(processes) = std::fs::read_dir(proc_dir) else {
            return Err("无法读取 /proc".into());
        };
        for process in processes.flatten() {
            let Ok(pid) = process.file_name().to_string_lossy().parse::<u32>() else {
                continue;
            };
            let Ok(fds) = std::fs::read_dir(process.path().join("fd")) else {
                continue;
            };
            for fd in fds.flatten() {
                let Ok(target) = std::fs::read_link(fd.path()) else {
                    continue;
                };
                let target = target.to_string_lossy();
                let Some(inode) = target
                    .strip_prefix("socket:[")
                    .and_then(|value| value.strip_suffix(']'))
                    .and_then(|value| value.parse::<u64>().ok())
                else {
                    continue;
                };
                inode_pids.entry(inode).or_insert(pid);
            }
        }

        Ok(entries
            .into_iter()
            .filter_map(|entry| {
                inode_pids.get(&entry.inode).map(|pid| PortSocket {
                    pid: *pid,
                    ..entry.socket
                })
            })
            .collect())
    }

    fn parse_proc_file(path: &str, protocol: &str, port: u16, entries: &mut Vec<Entry>) -> Result<(), String> {
        let Ok(content) = std::fs::read_to_string(path) else {
            // 文件不存在通常是内核未启用 IPv6 等情况，视为无数据而非错误。
            return Ok(());
        };
        for line in content.lines().skip(1) {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() < 10 {
                continue;
            }
            let Some((address, local_port)) = fields[1].split_once(':') else {
                continue;
            };
            let Ok(local_port) = u16::from_str_radix(local_port, 16) else {
                continue;
            };
            if local_port != port {
                continue;
            }
            let Some(local_address) = format_address(address) else {
                continue;
            };
            let state = match protocol {
                "TCP" => tcp_state_name(fields[3]),
                _ => "UDP",
            };
            let Ok(inode) = fields[9].parse::<u64>() else {
                continue;
            };
            entries.push(Entry {
                inode,
                socket: PortSocket {
                    protocol: protocol.into(),
                    local_address: format!("{local_address}:{local_port}"),
                    state: state.into(),
                    pid: 0,
                },
            });
        }
        Ok(())
    }

    fn format_address(hex: &str) -> Option<String> {
        match hex.len() {
            8 => {
                // /proc 中 IPv4 地址为小端序 u32 的十六进制。
                let value = u32::from_str_radix(hex, 16).ok()?;
                Some(Ipv4Addr::from(value.to_be_bytes()).to_string())
            }
            32 => {
                // IPv6 以 4 个小端序 u32 呈现，逐组还原字节序。
                let mut bytes = [0_u8; 16];
                for (group, chunk) in hex.as_bytes().chunks(8).enumerate() {
                    let hex_group = std::str::from_utf8(chunk).ok()?;
                    let value = u32::from_str_radix(hex_group, 16).ok()?;
                    bytes[group * 4..group * 4 + 4].copy_from_slice(&value.to_be_bytes());
                }
                Some(Ipv6Addr::from(bytes).to_string())
            }
            _ => None,
        }
    }

    fn tcp_state_name(hex: &str) -> &'static str {
        match hex {
            "01" => "ESTABLISHED",
            "02" => "SYN_SENT",
            "03" => "SYN_RECV",
            "04" => "FIN_WAIT_1",
            "05" => "FIN_WAIT_2",
            "06" => "TIME_WAIT",
            "07" => "CLOSE",
            "08" => "CLOSE_WAIT",
            "09" => "LAST_ACK",
            "0A" => "LISTEN",
            "0B" => "CLOSING",
            _ => "UNKNOWN",
        }
    }
}

#[cfg(target_os = "macos")]
fn collect_sockets(port: u16) -> Result<Vec<PortSocket>, String> {
    macos_impl::collect(port)
}

#[cfg(target_os = "macos")]
mod macos_impl {
    use super::PortSocket;
    use std::process::Command;

    pub fn collect(port: u16) -> Result<Vec<PortSocket>, String> {
        // lsof 无匹配时退出码为 1，这是“端口空闲”而非错误。
        let output = Command::new("lsof")
            .args(["-nP", "-i", &format!(":{port}")])
            .output()
            .map_err(|_| "未找到 lsof 命令，无法查询端口占用".to_string())?;
        if !output.stdout.is_empty() && !output.status.success() {
            return Err(format!("lsof 查询失败（退出码 {:?}）", output.status.code()));
        }
        let text = String::from_utf8_lossy(&output.stdout);
        let mut sockets = Vec::new();
        for line in text.lines().skip(1) {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() < 9 {
                continue;
            }
            let Ok(pid) = fields[1].parse::<u32>() else {
                continue;
            };
            let node = fields[7];
            if node != "TCP" && node != "UDP" {
                continue;
            }
            // NAME 列可能带 "(LISTEN)" 等状态后缀，也可能状态在独立一列。
            let mut name = fields[8].to_string();
            let mut state = if node == "UDP" { "UDP" } else { "UNKNOWN" };
            if let Some(rest) = fields.get(9) {
                if let Some(inner) = rest.strip_prefix('(').and_then(|value| value.strip_suffix(')')) {
                    state = inner;
                } else {
                    name.push(' ');
                    name.push_str(rest);
                }
            }
            if node == "UDP" {
                state = "UDP";
            }
            sockets.push(PortSocket {
                protocol: node.into(),
                local_address: name,
                state: state.into(),
                pid,
            });
        }
        Ok(sockets)
    }
}
