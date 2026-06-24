# sbox

sbot 配套桌面工具箱（Tauri + Vue 3）。内置小工具：

- **xiaoai-login** — 小爱登录，获取设备 cookie
- **keystore-gen** — keystore 生成
- **codec** — 编解码（Base64 / URL / Hex / 哈希等）
- **json-convert** — JSON / YAML / TOML 互转与校验
- **jwt** — JWT 解码（header / payload / 时间声明）
- **timestamp** — 时间戳 / 时区转换
- **random-gen** — 密码 / UUID / 随机字符串生成
- **checksum** — 文件校验（计算并比对哈希）

> 新增工具：在 [src/tools/registry.ts](src/tools/registry.ts) 追加一条即可，路由与首页（含搜索/分类）会自动接入。

## 开发

```bash
pnpm install
pnpm dev            # 启动 tauri dev
```

## 构建

```bash
pnpm windows:build  # 或 macos:build / linux:build
```