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

## 发布

推送 `v*` tag 触发 `.github/workflows/release.yml`，产出各平台安装包与 updater manifest。
发布前需在仓库 Secrets 配置 `TAURI_SIGNING_PRIVATE_KEY` 与 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`。

```bash
# 在 CHANGELOG.md 写好对应版本条目后
git tag v0.0.1 && git push origin v0.0.1
```
