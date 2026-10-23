# @sbox/ui

sbox 的共用 Vue 界面、路由、工具逻辑与主题。桌面宿主 `packages/app` 通过 workspace 依赖复用本包。

## 包边界与扩展

本包仅保存浏览器和桌面共用的页面、工具与能力接口。桌面专属的 Vue 页面、设置、快捷键与初始化逻辑位于 `packages/app/src/`，不由 UI 包导入。

- `src/platform/native.ts`：文件保存、版本信息和外部链接等共用能力接口。
- `src/platform/native_web.ts`：共用能力的浏览器实现。
- `packages/app/src/platform/native.ts`：扩展共用接口，定义窗口、快捷键及更新契约。各桌面工具的类型与 Rust 命令调用放在 `packages/app/src/tools/<分类>/<工具>/native.ts`。
- `packages/app/src/platform/native_rust.ts`：桌面实现，`configureDesktopNative()` 同时向 app 和共用 UI 注入这一实现。
- `packages/app/src/shell/registry.ts`：注册桌面独有工具、设置页及截图覆盖层路由。

宿主通过 `<SboxApp :tools="tools" />` 注入导航列表，通过 `createSboxRouter({ tools, routes })` 注册页面。`routes` 中同路径的路由会替换共用默认页面；`meta.bare: true` 表示该页面使用无外壳布局。`packages/app/src/main.ts` 包含完整示例。

更新弹窗通过共用 App 的 `updater` 插槽挂载。共用的 `@sbox/ui/about` 提供应用信息与 GitHub 入口，桌面通过它的 `update` 插槽添加更新面板。

共用工具包括编码转换、JSON、JSON/YAML/TOML 互转、JWT、时间戳、随机生成、文件校验和二维码。系统信息、批量重命名、截图、证书/SSH 生成和原生登录等页面与配套代码全部由 app 持有；Web 构建不包含这些页面。浏览器剪贴板遵循浏览器的安全上下文及用户权限限制。

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
