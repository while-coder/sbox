# sbox

sbot 配套桌面工具箱（Tauri + Vue 3）。内置小工具：

- **xiaoai-login** — 小爱登录，获取设备 cookie
- **keystore-gen** — keystore 生成
- **codec** — 编解码

## 开发

```bash
pnpm install
pnpm dev            # 启动 tauri dev
```

## 构建

```bash
pnpm windows:build  # 或 macos:build / linux:build
```