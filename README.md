# sbox

sbox 是一个桌面工具箱，提供常用的格式处理、生成校验、图片媒体和账号辅助功能。

共用界面位于 [`packages/ui`](packages/ui/README.md)，桌面宿主 `packages/app` 注入 `NativeRust` 连接 Tauri；浏览器预览注入 `NativeWeb`。运行 `pnpm dev:web` 预览浏览器支持的工具，`pnpm build:web` 生成 Web 版本；`pnpm check:ui`、`pnpm check:app` 分别检查共用页面和桌面宿主。

## 版本与原生资源

根 `package.json` 是版本来源，修改后运行 `pnpm sync:versions`，同步应用包与 Rust 应用版本（含 Cargo.lock）。Tauri 配置直接读取根版本。

图标源文件为 `packages/icon/icon.png`，运行 `pnpm sync:icons` 刷新桌面和 Web 图标；macOS 图标需在 macOS 上生成。原生配置、图标生成和签名脚本按平台放在 [src-tauri/native](packages/app/src-tauri/native/README.md)。开发和构建会自动生成对应平台图标。

## 功能

### 编解码 / 数据

- 编解码工具：支持 Base64、URL、Hex、HTML、Unicode、JSON 等常用编码转换，并可计算文本或文件哈希。
- JSON 格式化 / 查看器：支持美化、压缩、排序、树状查看，以及复制属性值或路径。
- JSON / YAML / TOML 互转：支持三种配置格式互相转换、格式化和校验。

### 开发辅助

- JWT 解码：解析 JWT 的 header 和 payload，并把时间声明转换为可读时间。
- 时间戳 / 时区转换：支持 Unix 时间戳、日期、相对时间和时区换算。
- 网页翻译：在应用内使用 Google、百度、Bing、有道、DeepL 或混元翻译，并可通过 Tab 随时切换服务。

### 生成 / 校验

- 随机生成器：生成密码、UUID 和随机字符串。
- 文件校验：计算文件或文本哈希，并与期望值比对。
- Keystore 生成：生成 Android APK 签名所需的 keystore，并给出常用发布配置所需的信息；也可读取现有 keystore 的证书指纹、计算发布密钥散列（base64(SHA1)）。
- SSH 密钥生成：可选择 Ed25519 或 RSA 4096，支持密码保护、复制公钥和查看 SHA-256 指纹。

### 图像 / 媒体

- 二维码生成 / 识别：生成二维码，或从图片、粘贴内容、拖拽文件、剪贴板中识别二维码。
- 截图：支持全屏框选截图、保存、复制到剪贴板、识别二维码，并可使用全局快捷键。

### 账号 / 设备

- 小爱登录：登录小米账号，导出小爱相关接入信息。
- Google Drive 登录：完成浏览器授权，导出 Google Drive 接入所需信息。
