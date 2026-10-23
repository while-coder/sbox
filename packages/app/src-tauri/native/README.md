# 原生平台资源

`windows/`、`macos/`、`linux/` 保存各平台维护的资源和脚本。Tauri 平台配置仍在 `src-tauri/tauri.<平台>.conf.json`，供 CLI 自动加载。

图标唯一源文件为 `packages/icon/icon.png`。运行根目录 `pnpm sync:icons` 生成 Windows、Linux 和 Web 图标；在 macOS 上同时生成 Dock 和托盘图标。macOS 保留现有的 856/1024 Dock 留白与 18 pt 托盘绘制。

`native/sync-icons.cjs` 负责平台分发、输入指纹缓存和 Tauri CLI 调用，各平台 `sync-icons.cjs` 负责输出处理。在 `src-tauri/` 下可运行 `node native/sync-icons.cjs windows|macos|linux`。Tauri 开发、打包及直接 Cargo 构建都会生成目标平台图标，需要先安装 Node.js 和 pnpm 依赖。

各平台 `icons/` 是生成目录，不纳入 Git。缓存位于根目录 `.tmp/native-icons/`，源图或生成脚本变化、缓存或输出缺失时重新生成。

macOS 的附加配置位于 `macos/Info.plist`；`sign.sh` 统一提供 `--setup`（创建证书）、`--check`（检查身份）、`--dev`（启动签名开发包）、`--import-ci`（CI 导入证书）以及指定路径签名，OpenSSL 配置内嵌于脚本。根目录 `scripts/mac-import-signing-cert.sh` 仅作为共用 CI 的兼容入口转发到 `--import-ci`。首次设置证书运行 `pnpm --filter @sbox/app macos:setup-signing`，启动签名开发包运行 `pnpm dev:app:mac`。
