# Updating App Store Connect via JWT

Drive the App Store Connect API directly — no browser, no scraping — to update backend info (app name, subtitle, what's new, keywords, submit for review). Auth is a self-signed ES256 JWT (the "Apple JWT"). This guide covers: creating an API key → generating a token → calling the API → common pitfalls.

## 1. Create an API key (one-time)

1. Open [App Store Connect](https://appstoreconnect.apple.com) → **Users and Access** → **Integrations** → **App Store Connect API**.
2. Click **Generate API Key**, give it a name (e.g. `sbox-api`), and pick a role:
   - Metadata edits / submitting builds: **App Manager**
   - Read-only: **Developer** (or **Finance** for sales data only)
3. You'll get three things:
   - **Issuer ID**: the UUID shown at the top of the page
   - **Key ID**: the 10-character ID in the key list
   - **`AuthKey_<KeyID>.p8`**: the private key — **downloadable only once**

::: warning
The `.p8` key grants partial control over your account. Keep it on your machine only (a non-committed directory like `secrets/` is a good spot), never commit it to git, never hard-code it in scripts. If it leaks, **revoke** it on the same page and generate a new one.
:::

## 2. Generate the JWT (ES256)

The token is a regular JWT with Apple's constraints:

| Segment | Field | Notes |
| --- | --- | --- |
| header | `alg` | always `ES256` |
| | `kid` | Key ID |
| | `typ` | `JWT` |
| payload | `iss` | Issuer ID (UUID) |
| | `iat` | issued-at, seconds since epoch |
| | `exp` | expiry — **at most iat + 20 minutes** |
| | `aud` | always `appstoreconnect-v1` |

Sign with the `.p8` (a PKCS#8 P-256 key) using ES256. The signature segment is raw `r||s` (64 bytes), not DER.

### Node script (zero dependencies)

Save as `gen_asc_jwt.mjs`:

```js
// Generate an App Store Connect API token from a .p8 key
// Usage: node gen_asc_jwt.mjs <AuthKey.p8> <KeyID> <IssuerID> [minutes, default 20]
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
  .sign({ key, dsaEncoding: 'ieee-p1363' }) // raw r||s
  .toString('base64url')

console.log(signingInput + '.' + signature)
```

```bash
TOKEN=$(node gen_asc_jwt.mjs ~/secrets/AuthKey_ABC123.p8 ABC123 69a6de12-xxxx-xxxx-xxxx-xxxxxxxxxxxx 20)
```

### Python (PyJWT)

```python
import time, jwt
token = jwt.encode(
    {"iss": ISSUER_ID, "iat": int(time.time()), "exp": int(time.time()) + 1200,
     "aud": "appstoreconnect-v1"},
    open("AuthKey_ABC123.p8").read(),
    algorithm="ES256", headers={"kid": KEY_ID},
)
```

Paste the result into sbox's **JWT decode** tool to sanity-check `kid` / `iss` / `exp`.

## 3. Update backend info

All requests share one header: `Authorization: Bearer <TOKEN>`, with JSON-API bodies. Base URL: `https://api.appstoreconnect.apple.com`.

### 3.1 Find the app ID

```bash
BASE=https://api.appstoreconnect.apple.com
curl -s -H "Authorization: Bearer $TOKEN" \
  "$BASE/v1/apps?filter[bundleId]=com.example.app" | jq '.data[0].id'
```

### 3.2 App name / subtitle / privacy policy URL

These live on `appInfoLocalizations` (one entry per locale):

```bash
# List localizations and grab the one you need
curl -s -H "Authorization: Bearer $TOKEN" \
  "$BASE/v1/apps/$APP_ID/appInfos" | jq '.data[].id'
curl -s -H "Authorization: Bearer $TOKEN" \
  "$BASE/v1/appInfos/$APP_INFO_ID/appInfoLocalizations" | jq '.data[] | {id, locale}'

# Update
curl -s -X PATCH -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  "$BASE/v1/appInfoLocalizations/$LOC_ID" -d '{
    "data": {
      "type": "appInfoLocalizations",
      "id": "'"$LOC_ID"'",
      "attributes": {
        "name": "New Name",
        "subtitle": "New subtitle",
        "privacyPolicyUrl": "https://example.com/privacy"
      }
    }
  }'
```

### 3.3 What's new / description / keywords

These live on `appStoreVersionLocalizations`:

```bash
# Find the version currently in edit (PREPARE_FOR_SUBMISSION)
curl -s -H "Authorization: Bearer $TOKEN" \
  "$BASE/v1/apps/$APP_ID/appStoreVersions?filter[appStoreState]=PREPARE_FOR_SUBMISSION" \
  | jq '.data[0].id'

# Update a locale entry
curl -s -X PATCH -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  "$BASE/v1/appStoreVersionLocalizations/$VER_LOC_ID" -d '{
    "data": {
      "type": "appStoreVersionLocalizations",
      "id": "'"$VER_LOC_ID"'",
      "attributes": {
        "whatsNew": "Bug fixes",
        "keywords": "tools,productivity",
        "promotionalText": "Text you can update without a new review"
      }
    }
  }'
```

> `promotionalText` is the only field that takes effect **without a new review** — good for limited-time promo copy.

### 3.4 Attach a build and submit for review

```bash
# Find a processed build
curl -s -H "Authorization: Bearer $TOKEN" \
  "$BASE/v1/builds?filter[app]=$APP_ID&filter[processingState]=VALID" | jq '.data[0].id'

# Attach it to the version
curl -s -X PATCH -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  "$BASE/v1/appStoreVersions/$VERSION_ID/relationships/build" -d '{
    "data": { "type": "builds", "id": "'"$BUILD_ID"'" }
  }'

# Submit for review
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

## 4. Skip the hand-rolled code: fastlane

For bulk metadata updates, fastlane's `deliver` wraps all of the above (same `.p8` auth):

```ruby
# Fastfile
app_store_connect_api_key(
  key_id: "ABC123",
  issuer_id: "69a6de12-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
  key_filepath: ENV["ASC_KEY_PATH"],   # points to AuthKey_ABC123.p8
  duration: 1200,                      # max 1200 seconds = 20 minutes
)

deliver(
  app_identifier: "com.example.app",
  metadata_path: "./fastlane/metadata", # edit the txt/json files here
  skip_screenshots: true,
  skip_binary_upload: true,
  automatic_release: false,
)
```

The `metadata` directory has one folder per locale with `name.txt`, `subtitle.txt`, `release_notes.txt`, `keywords.txt`, etc. Edit them and run `fastlane deliver` — it diffs and updates.

Use `pilot` for TestFlight only; `produce` creates new apps.

## 5. Troubleshooting

| Symptom | Cause |
| --- | --- |
| 401 `INVALID_JWT` | wrong `kid`/`iss`, clock skew, `exp` beyond iat+20min, or a DER-encoded signature |
| 401 after a while | token expired — re-sign (best to sign per request) |
| 403 | key role lacks permission (e.g. Developer role trying to edit metadata) |
| 429 | per-key concurrency limit hit; honor the `Retry-After` header |

Other notes:

- **The `.p8` downloads once**; if lost, revoke and recreate.
- The token needs no exchange step — sign it and use it as `Bearer` directly; each endpoint validates the key's role.
- Every write shows up in the App Store Connect web UI; if the web UI holds an edit lock on a version the API returns 409.
- Full reference: [App Store Connect API docs](https://developer.apple.com/app-store-connect/api/) (OpenAPI spec available for download).
