# 快速开始

**sbox** 是 [sbot](https://github.com/while-coder/sbot) 的配套桌面工具箱。它是一个基于 [Tauri](https://tauri.app/) + Vue 3 的小应用，把配置 sbot 时常遇到的杂活集中起来，省得你手动写脚本：

- [**小爱登录**](./xiaoai-login) — 登录小米账号，导出 `channel.xiaoai` 所需的设备凭据。
- [**Keystore 生成**](./keystore-gen) — 生成 Android 签名 keystore（PKCS12）以及配套的 GitHub Actions secret。
- [**编解码工具**](./codec) — Base64 / URL / Hex / HTML / Unicode / JSON 转换，MD5 与 SHA 哈希，UUID、时间戳与字节大小。

每个工具都会产出一个值（通常是一段 JSON），复制后粘贴到 sbot admin 的 **「粘贴凭据 JSON」** 入口，或填进仓库 secret 即可。

## 安装

到 [Releases 页面](https://github.com/while-coder/sbox/releases) 下载对应平台的安装包并运行。

- **Windows** — `.msi` / `.exe`
- **macOS** — `.dmg`
- **Linux** — `.AppImage` / `.deb`

应用内置自动更新，有新版本时会提示你升级。

## 从源码运行

```bash
pnpm install
pnpm dev            # 启动 tauri dev
```

构建桌面安装包：

```bash
pnpm windows:build  # 或 macos:build / linux:build
```

> 构建需要先准备好 [Tauri 环境](https://tauri.app/start/prerequisites/)（Rust 工具链 + 对应平台的 webview 依赖）。

## 使用流程

1. 启动 sbox，在首页卡片中挑一个工具。
2. 按界面提示操作。
3. 复制输出，粘到需要的地方 —— sbot admin、`.env` 或 GitHub secret。

从 [小爱登录](./xiaoai-login) 看一个完整示例。
