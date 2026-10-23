# @sbox/ui

sbox 的共用 Vue 界面、路由、工具逻辑与主题。桌面宿主 `packages/app` 通过 workspace 依赖复用本包。

## Native 对接

`src/platform/native.ts` 定义 `NativePlatform` 和 `configureNative()`。界面仅使用应用级操作，不导入 Tauri SDK，也不包含 Rust command 名称。

- `packages/app/src/platform/native_rust.ts`：桌面实现，连接 Tauri 的文件、对话框、剪贴板、窗口、快捷键、更新和各工具 Rust 命令。截图窗口控制器与工具命令实现在同目录下。
- `src/platform/native_web.ts`：浏览器实现，提供文件下载、剪贴板、版本信息和外部链接；桌面专属操作明确返回错误。
- `src/tools/**/native.ts`：共用工具的数据类型和服务入口，转发到注入的 native 实现。

宿主应先调用 `configureNative(new NativeRust())` 或 `configureNative(new NativeWeb())`，再动态导入 `@sbox/ui/app`、`@sbox/ui/router` 并挂载。`packages/app/src/main.ts` 包含完整示例。桌面更新弹窗通过共用 App 的 `updater` 插槽挂载。

浏览器导航提供编码转换、JSON、JSON/YAML/TOML 互转、JWT、时间戳、随机生成、文件校验、二维码。系统信息、系统文件修改、截图、证书/SSH 生成和原生登录等工具保留在桌面端；浏览器直接访问这些路由会返回首页。浏览器剪贴板遵循浏览器的安全上下文及用户权限限制。

## 开发与检查

在仓库根目录运行：

```sh
pnpm dev:web       # 浏览器预览，同一宿主自动注入 NativeWeb
pnpm build:web     # Web 模式构建，移除 Rust 对接代码
pnpm dev          # Tauri 开发，注入 NativeRust
pnpm check:ui     # 共用 Vue 页面与 TypeScript 检查
pnpm check:app    # 桌面宿主 TypeScript 与 native 契约检查
```

UI 包以源码方式导出，由宿主 Vite 编译，无需单独构建。`pnpm sync:versions` 同步本包版本。
