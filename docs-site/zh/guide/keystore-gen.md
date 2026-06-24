# Keystore 生成

生成 Android APK 签名 keystore（PKCS12），并自动给出在 CI 中签名 release 包所需的 4 个 GitHub Actions secret。

## 前置条件

- 需要安装 **Java (JDK)** —— 工具底层调用 `keytool`。sbox 启动时会检测 Java 并显示其路径。若未检测到 Java，请先安装 JDK 再重新打开工具。

## 操作步骤

1. 打开 sbox，选择 **Keystore 生成**。
2. 填写表单：
   - **别名 (Alias)** —— keystore 内的密钥别名。
   - **store 密码** / **key 密码** —— 各至少 6 位，需输入两次确认。
   - **有效期** —— 默认 *永不过期*（100 年），也可自定义年数。
   - **证书信息（高级）** —— Common Name 必填；OU / 组织 / 城市 / 省份 / 国家（默认 `CN`）可选。
3. 点击生成，选择 `.keystore` 文件的保存位置。
4. sbox 会展示 keystore 详情，以及一段可直接粘贴的 GitHub Actions secret。

## GitHub Actions secret

生成的 secret 块包含 release 工作流所需的 4 个值：

| Secret | 含义 |
| --- | --- |
| `KEYSTORE_BASE64` | keystore 文件的 Base64 编码 |
| `KEYSTORE_PASSWORD` | store 密码 |
| `KEY_ALIAS` | 密钥别名 |
| `KEY_PASSWORD` | key 密码 |

在仓库的 **Settings → Secrets and variables → Actions** 中添加它们。每个值都有独立的 **复制** 按钮。

## 说明

- 妥善保管 `.keystore` 文件及其密码 —— 一旦丢失，就无法再发布与原签名一致的更新。
- 应用签名密钥建议使用较长有效期（默认 100 年），让它比应用本身活得更久。
