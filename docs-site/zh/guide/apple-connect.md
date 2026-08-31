# 用 JWT 调 App Store Connect API 更新后台信息

不打开浏览器、不抓包，直接用 App Store Connect API 改后台信息（App 名称、副标题、版本更新说明、关键词、提审等）。鉴权方式是自签的 ES256 JWT（俗称「苹果 JWT」），本文档覆盖：创建 API 密钥 → 生成 token → 调接口更新信息 → 常见坑。

## 1. 创建 API 密钥（一次性）

1. 打开 [App Store Connect](https://appstoreconnect.apple.com) → **用户和访问** → **集成**（旧版叫「密钥」）→ **App Store Connect API**。
2. 点 **生成 API 密钥**，填一个名字（如 `sbox-api`），角色按需选：
   - 只改元数据/提审：**App Manager**
   - 只读：**Developer** 或 **Finance**（只看销售数据用 Finance）
3. 生成后会得到三个东西：
   - **Issuer ID**：页面顶部显示的 UUID
   - **Key ID**：密钥列表里的 10 位 ID
   - **`AuthKey_<KeyID>.p8`**：私钥文件，**只有这一次下载机会**

::: warning
`.p8` 私钥等于你账号的部分操作权，只存本机（建议放 `secrets/` 一类不入库的目录），不要提交进 git，不要写死在脚本里。泄露后到同一页面**撤销**并重建即可。
:::

## 2. 生成 JWT（ES256）

token 结构与普通 JWT 相同，Apple 的要求是：

| 段 | 字段 | 说明 |
| --- | --- | --- |
| header | `alg` | 固定 `ES256` |
| | `kid` | Key ID |
| | `typ` | `JWT` |
| payload | `iss` | Issuer ID（UUID） |
| | `iat` | 签发时间（秒级时间戳） |
| | `exp` | 过期时间，**最多 iat + 20 分钟** |
| | `aud` | 固定 `appstoreconnect-v1` |

签名用 `.p8`（PKCS#8 格式的 P-256 私钥）做 ES256，签名段是原始 `r||s`（64 字节），不是 DER。

### Node 脚本（零依赖）

保存为 `gen_asc_jwt.mjs`：

```js
// 用 .p8 生成 App Store Connect API token
// 用法: node gen_asc_jwt.mjs <AuthKey.p8> <KeyID> <IssuerID> [分钟数, 默认 20]
import { readFileSync } from 'node:fs'
import { createSign } from 'node:crypto'

const [,, keyPath, keyId, issuerId, minutes = '20'] = process.argv
const key = readFileSync(keyPath, 'utf8')

const b64url = (obj) => Buffer.from(JSON.stringify(obj)).toString('base64url')
const now = Math.floor(Date.now() / 1000)
const signingInput = `${b64url({ alg: 'ES256', kid: keyId, typ: 'JWT' })}.`
  + b64url({ iss: issuerId, iat: now, exp: now + Number(minutes) * 60, aud: 'appstoreconnect-v1' })

const signature = createSign('sha256')
  .update(signingInput)
  .sign({ key, dsaEncoding: 'ieee-p1363' }) // 输出原始 r||s
  .toString('base64url')

console.log(signingInput + '.' + signature)
```

```bash
TOKEN=$(node gen_asc_jwt.mjs ~/secrets/AuthKey_ABC123.p8 ABC123 69a6de12-xxxx-xxxx-xxxx-xxxxxxxxxxxx 20)
```

### Python（PyJWT）

```python
import time, jwt
token = jwt.encode(
    {"iss": ISSUER_ID, "iat": int(time.time()), "exp": int(time.time()) + 1200,
     "aud": "appstoreconnect-v1"},
    open("AuthKey_ABC123.p8").read(),
    algorithm="ES256", headers={"kid": KEY_ID},
)
```

生成后可以直接用本仓库的 **JWT 解码** 工具粘贴检查 `kid` / `iss` / `exp` 是否正确。

## 3. 调接口更新后台信息

所有请求头都一样：`Authorization: Bearer <TOKEN>`，body 为 JSON-API 格式。基地址 `https://api.appstoreconnect.apple.com`。

### 3.1 找到 App ID

```bash
BASE=https://api.appstoreconnect.apple.com
curl -s -H "Authorization: Bearer $TOKEN" \
  "$BASE/v1/apps?filter[bundleId]=com.example.app" | jq '.data[0].id'
```

### 3.2 改 App 名称 / 副标题 / 隐私政策 URL

这些挂在 `appInfoLocalizations` 上（每个语言一条）：

```bash
# 列出本地化，拿到 zh-Hans 那条的 id
curl -s -H "Authorization: Bearer $TOKEN" \
  "$BASE/v1/apps/$APP_ID/appInfos" | jq '.data[].id'
curl -s -H "Authorization: Bearer $TOKEN" \
  "$BASE/v1/appInfos/$APP_INFO_ID/appInfoLocalizations" | jq '.data[] | {id, locale}'

# 更新
curl -s -X PATCH -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  "$BASE/v1/appInfoLocalizations/$LOC_ID" -d '{
    "data": {
      "type": "appInfoLocalizations",
      "id": "'"$LOC_ID"'",
      "attributes": {
        "name": "新的App名",
        "subtitle": "新的副标题",
        "privacyPolicyUrl": "https://example.com/privacy"
      }
    }
  }'
```

### 3.3 改版本「本次更新内容」/ 描述 / 关键词

这些挂在 `appStoreVersionLocalizations` 上：

```bash
# 找到当前编辑中的版本（PREPARE_FOR_SUBMISSION）
curl -s -H "Authorization: Bearer $TOKEN" \
  "$BASE/v1/apps/$APP_ID/appStoreVersions?filter[appStoreState]=PREPARE_FOR_SUBMISSION" \
  | jq '.data[0].id'

# 拿到语言条目后更新
curl -s -X PATCH -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  "$BASE/v1/appStoreVersionLocalizations/$VER_LOC_ID" -d '{
    "data": {
      "type": "appStoreVersionLocalizations",
      "id": "'"$VER_LOC_ID"'",
      "attributes": {
        "whatsNew": "修复了若干问题",
        "keywords": "工具,效率",
        "promotionalText": "无需审核即可更新的推广文本"
      }
    }
  }'
```

> `promotionalText` 是唯一**不需要重新审核**就能生效的字段，适合放活动文案。

### 3.4 关联构建并提审

```bash
# 找到处理完成的构建
curl -s -H "Authorization: Bearer $TOKEN" \
  "$BASE/v1/builds?filter[app]=$APP_ID&filter[processingState]=VALID" | jq '.data[0].id'

# 把构建挂到版本上
curl -s -X PATCH -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  "$BASE/v1/appStoreVersions/$VERSION_ID/relationships/build" -d '{
    "data": { "type": "builds", "id": "'"$BUILD_ID"'" }
  }'

# 提交审核
curl -s -X POST -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  "$BASE/v1/appStoreVersionSubmissions" -d '{
    "data": {
      "type": "appStoreVersionSubmissions",
      "relationships": {
        "appStoreVersion": { "data": { "type": "appStoreVersions", "id": "'"$VERSION_ID"'" } }
      }
    }
  }'
```

## 4. 用 fastlane 省掉手写

如果只是想批量更新元数据，fastlane 的 `deliver` 已经封装好上面全部流程（同样用 `.p8` 鉴权）：

```ruby
# Fastfile
app_store_connect_api_key(
  key_id: "ABC123",
  issuer_id: "69a6de12-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
  key_filepath: ENV["ASC_KEY_PATH"],   # 指向 AuthKey_ABC123.p8
  duration: 1200,                      # 最长 1200 秒 = 20 分钟
)

deliver(
  app_identifier: "com.example.app",
  metadata_path: "./fastlane/metadata", # 改这里面的 txt/json 文件即可
  skip_screenshots: true,
  skip_binary_upload: true,
  automatic_release: false,
)
```

`metadata` 目录里每个语言一个文件夹，`name.txt`、`subtitle.txt`、`release_notes.txt`、`keywords.txt` 等，改完跑 `fastlane deliver` 就会 diff 并更新。

只发 TestFlight 用 `pilot`；建新 App 用 `produce`。

## 5. 常见问题

| 现象 | 原因 |
| --- | --- |
| 401 `INVALID_JWT` | `kid`/`iss` 填错、时钟偏差过大、`exp` 超过 iat+20min、或用了 DER 签名格式 |
| 401 过一会儿才出现 | token 过期了，重新生成（建议脚本里每次请求前现签） |
| 403 | 密钥角色权限不够（如用 Developer 角色想改元数据） |
| 429 | 该密钥的并发请求数超限，看响应头 `Retry-After` 后重试 |

其他注意：

- **`.p8` 只能下载一次**，忘了下载只能撤销重建。
- token 不需要跟 Apple 换取任何东西，签完直接当 `Bearer` 用；不同接口对 token 内密钥的角色做校验。
- 所有写操作都会同步到 App Store Connect 网页后台，网页上能看到改动；被网页端正在编辑的版本锁住时 API 会返回 409。
- 完整接口参考：[App Store Connect API 文档](https://developer.apple.com/app-store-connect/api/)（可下载 OpenAPI spec）。
