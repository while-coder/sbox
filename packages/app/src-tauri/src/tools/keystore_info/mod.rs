use base64::Engine;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::process::{Command, Stdio};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListInput {
    pub path: String,
    /// 可选：只查这个别名。留空则列出所有别名。
    pub alias: Option<String>,
    /// 可选：keystore 密码。留空则不带 -storepass 调用（会因密码校验失败而报错，不会挂起）。
    pub store_password: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeystoreEntry {
    pub alias: String,
    pub entry_type: Option<String>,
    pub creation_date: Option<String>,
    pub owner: Option<String>,
    pub issuer: Option<String>,
    pub serial_number: Option<String>,
    pub valid_from: Option<String>,
    pub valid_until: Option<String>,
    pub signature_algorithm: Option<String>,
    pub key_algorithm: Option<String>,
    pub fingerprint_md5: String,
    pub fingerprint_sha1: String,
    pub fingerprint_sha256: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListResult {
    pub path: String,
    pub store_type: Option<String>,
    pub entries: Vec<KeystoreEntry>,
}

/// 对应 `keytool -list -v -keystore <path> [-alias <alias>]`，解析所有别名的证书指纹。
#[tauri::command(rename_all = "camelCase")]
pub async fn keystore_info_list(input: ListInput) -> Result<ListResult, String> {
    if input.path.trim().is_empty() {
        return Err("请先选择 keystore 文件".into());
    }
    if !std::path::Path::new(&input.path).exists() {
        return Err(format!("文件不存在: {}", input.path));
    }

    let keytool = which::which("keytool")
        .map_err(|_| "未找到 keytool。请安装 JDK 17+ 后重试。".to_string())?;

    let mut cmd = Command::new(&keytool);
    // 强制英文输出：中文 Windows 上 keytool 输出是 GBK 编码，UTF-8 解码会乱码；
    // 英文输出为纯 ASCII，且各平台格式一致
    cmd.args([
        "-J-Duser.language=en",
        "-J-Duser.country=US",
        "-list",
        "-v",
        "-keystore",
        &input.path,
    ]);
    if let Some(alias) = input.alias.as_deref().filter(|s| !s.trim().is_empty()) {
        cmd.args(["-alias", alias.trim()]);
    }
    if let Some(pass) = input.store_password.as_deref().filter(|s| !s.is_empty()) {
        // 密码经环境变量传递，不出现在进程命令行里
        cmd.args(["-storepass:env", "KS_STORE_PASS"])
            .env("KS_STORE_PASS", pass);
    }
    // 无密码时避免 keytool 交互式提问挂起：stdin 直接置空，读到 EOF 即报错退出
    cmd.stdin(Stdio::null());

    let output = cmd
        .output()
        .map_err(|e| format!("调用 keytool 失败: {e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let msg = if stderr.trim().is_empty() { stdout.trim() } else { stderr.trim() };
        if msg.contains("password") || msg.contains("密码") {
            return Err("keystore 密码错误（或未提供密码）".into());
        }
        return Err(format!("keytool 退出码非零:\n{}", msg));
    }

    Ok(parse_keytool_list(&input.path, &stdout))
}

/// 解析 `keytool -list -v` 输出，兼容中文与英文 locale。
fn parse_keytool_list(path: &str, text: &str) -> ListResult {
    let mut entries: Vec<KeystoreEntry> = Vec::new();

    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        if let Some(v) = after_marker(t, &["别名", "Alias name"]) {
            entries.push(KeystoreEntry {
                alias: v,
                entry_type: None,
                creation_date: None,
                owner: None,
                issuer: None,
                serial_number: None,
                valid_from: None,
                valid_until: None,
                signature_algorithm: None,
                key_algorithm: None,
                fingerprint_md5: String::new(),
                fingerprint_sha1: String::new(),
                fingerprint_sha256: String::new(),
            });
            continue;
        }
        let Some(cur) = entries.last_mut() else { continue };
        if cur.entry_type.is_none() {
            if let Some(v) = after_marker(t, &["条目类型", "Entry type"]) {
                cur.entry_type = Some(v);
                continue;
            }
        }
        if cur.creation_date.is_none() {
            if let Some(v) = after_marker(t, &["创建日期", "Creation date"]) {
                cur.creation_date = Some(v);
                continue;
            }
        }
        if cur.owner.is_none() {
            if let Some(v) = after_marker(t, &["所有者", "Owner"]) {
                cur.owner = Some(v);
                continue;
            }
        }
        if cur.issuer.is_none() {
            if let Some(v) = after_marker(t, &["发布者", "Issuer"]) {
                cur.issuer = Some(v);
                continue;
            }
        }
        if cur.serial_number.is_none() {
            if let Some(v) = after_marker(t, &["序列号", "Serial number"]) {
                cur.serial_number = Some(v);
                continue;
            }
        }
        if cur.signature_algorithm.is_none() {
            // 注意：要在 SHA1/SHA256 指纹之后再匹配也没问题，此行值形如 "SHA384withRSA" 不带冒号
            if let Some(v) = after_marker(t, &["签名算法名称", "Signature algorithm name"]) {
                cur.signature_algorithm = Some(v);
                continue;
            }
        }
        if cur.key_algorithm.is_none() {
            if let Some(v) = after_marker(t, &["主体公共密钥算法", "Subject Public Key Algorithm"]) {
                cur.key_algorithm = Some(v);
                continue;
            }
        }
        if let Some(v) = after_marker(t, &["生效时间", "Valid from"]) {
            // keytool 把起止时间放在同一行: "Valid from: X until: Y"，拆开
            let (from, until) = split_validity(&v);
            cur.valid_from = Some(from);
            if let Some(u) = until {
                cur.valid_until = Some(u);
            }
            continue;
        }
        if cur.valid_until.is_none() {
            if let Some(v) = after_marker(t, &["失效时间", "Valid until"]) {
                cur.valid_until = Some(v);
                continue;
            }
        }
        if cur.fingerprint_md5.is_empty() {
            if let Some(v) = after_marker(t, &["MD5"]) {
                cur.fingerprint_md5 = v;
                continue;
            }
        }
        if cur.fingerprint_sha1.is_empty() {
            if let Some(v) = after_marker(t, &["SHA1", "SHA-1"]) {
                cur.fingerprint_sha1 = v;
                continue;
            }
        }
        if cur.fingerprint_sha256.is_empty() {
            if let Some(v) = after_marker(t, &["SHA256", "SHA-256"]) {
                cur.fingerprint_sha256 = v;
                continue;
            }
        }
    }

    // 密钥库类型在文件头部，与别名无关
    let store_type = text
        .lines()
        .find_map(|l| after_marker(l.trim(), &["密钥库类型", "Keystore type"]));

    ListResult {
        path: path.to_string(),
        store_type,
        entries,
    }
}

/// 拆分 "Wed Sep 09 18:14:51 CST 2026, 失效时间: Fri Oct 09 ..." / "X until: Y" 形式的时间段。
fn split_validity(v: &str) -> (String, Option<String>) {
    for sep in ["until:", "until：", "失效时间:", "失效时间："] {
        if let Some((from, until)) = v.split_once(sep) {
            return (from.trim_end_matches(&[',', '，', ' '][..]).to_string(), Some(until.trim().to_string()));
        }
    }
    (v.to_string(), None)
}

/// 若该行以某个标记开头（或包含 "标记:"），返回标记后去空的值。
/// 用前缀匹配而不是 contains，避免把 "SHA1" 误匹配进 "证书指纹 SHA1" 之外的行。
fn after_marker(line: &str, markers: &[&str]) -> Option<String> {
    for m in markers {
        // 中英文输出里标记后跟的冒号可能是半角或全角
        for colon in [":", "："] {
            let pat = format!("{m}{colon}");
            if let Some((_, v)) = line.split_once(&pat) {
                let v = v.trim();
                if !v.is_empty() {
                    return Some(v.to_string());
                }
            }
        }
    }
    None
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyHashInput {
    pub path: String,
    /// 发布密钥散列按别名导出证书，必填。
    pub alias: String,
    /// 可选：keystore 密码。经环境变量传递，不出现在进程命令行里。
    pub store_password: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyHashResult {
    pub alias: String,
    /// 发布密钥散列：base64(SHA1(证书 DER))，对应
    /// `keytool -exportcert | openssl sha1 -binary | openssl base64`（微信开放平台等要求）。
    pub sha1_base64: String,
    /// 冒号分隔的 SHA-1 十六进制指纹，便于与 `keytool -list` 输出核对。
    pub sha1_hex: String,
}

/// 对应 `keytool -exportcert -alias <alias> -keystore <path>`，取证书 DER 并计算散列。
#[tauri::command(rename_all = "camelCase")]
pub async fn keystore_key_hash(input: KeyHashInput) -> Result<KeyHashResult, String> {
    if input.path.trim().is_empty() {
        return Err("请先选择 keystore 文件".into());
    }
    if !std::path::Path::new(&input.path).exists() {
        return Err(format!("文件不存在: {}", input.path));
    }
    let alias = input.alias.trim();
    if alias.is_empty() {
        return Err("请填写要导出证书的别名（alias）".into());
    }

    let keytool = which::which("keytool")
        .map_err(|_| "未找到 keytool。请安装 JDK 17+ 后重试。".to_string())?;

    let mut cmd = Command::new(&keytool);
    // 与 keystore_info_list 一致：强制英文输出，避免中文 Windows 的 GBK 乱码
    cmd.args([
        "-J-Duser.language=en",
        "-J-Duser.country=US",
        "-exportcert",
        "-alias",
        alias,
        "-keystore",
        &input.path,
    ]);
    if let Some(pass) = input.store_password.as_deref().filter(|s| !s.is_empty()) {
        cmd.args(["-storepass:env", "KS_STORE_PASS"]).env("KS_STORE_PASS", pass);
    }
    // stdout 输出证书 DER 二进制；stdin 置空避免无密码时交互式提问挂起
    cmd.stdin(Stdio::null());

    let output = cmd
        .output()
        .map_err(|e| format!("调用 keytool 失败: {e}"))?;
    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let msg = if stderr.trim().is_empty() { stdout.trim() } else { stderr.trim() };
        if msg.contains("password") || msg.contains("密码") {
            return Err("keystore 密码错误（或未提供密码）".into());
        }
        return Err(format!("keytool 退出码非零:\n{}", msg));
    }

    key_hash_from_der(alias, &output.stdout)
}

/// 由证书 DER 计算 base64(SHA1) 与冒号分隔的十六进制指纹。
fn key_hash_from_der(alias: &str, der: &[u8]) -> Result<KeyHashResult, String> {
    if der.is_empty() {
        return Err("keytool 未输出证书内容".into());
    }
    let digest: [u8; 20] = Sha1::digest(der).into();
    let sha1_hex = digest.iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(":");
    Ok(KeyHashResult {
        alias: alias.to_string(),
        sha1_base64: base64::engine::general_purpose::STANDARD.encode(digest),
        sha1_hex,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// keytool -J-Duser.language=en -list -v 的真实输出节选（JDK 21, PKCS12）
    const SAMPLE_EN: &str = r#"Keystore type: PKCS12
Keystore provider: SUN

Your keystore contains 1 entry

Alias name: myapp
Creation date: Sep 9, 2026
Entry type: PrivateKeyEntry
Certificate chain length: 1
Certificate[1]:
Owner: CN=test
Issuer: CN=test
Serial number: b2817dba7b8103ff
Valid from: Wed Sep 09 18:14:51 CST 2026 until: Fri Oct 09 18:14:51 CST 2026
Certificate fingerprints:
	 SHA1: BC:8D:42:65:57:03:74:5D:B2:B7:E2:30:49:C5:2A:47:9B:8F:91:E6
	 SHA256: 24:BC:D7:32:6A:10:21:7C:21:7A:D5:92:C1:7C:12:1C:A3:A8:6A:AB:91:7B:54:46:81:0B:B4:DF:ED:87:BA:85
Signature algorithm name: SHA384withRSA
Subject Public Key Algorithm: 2048-bit RSA key
Version: 3
"#;

    /// 中文 locale 输出节选（macOS/Linux 上为 UTF-8；Windows GBK 情况已被 -J 参数规避）
    const SAMPLE_ZH: &str = r#"密钥库类型: PKCS12
别名: myapp
创建日期: 2026年9月9日
条目类型: PrivateKeyEntry
所有者: CN=test
发布者: CN=test
序列号: b2817dba7b8103ff
生效时间: Wed Sep 09 18:14:51 CST 2026, 失效时间: Fri Oct 09 18:14:51 CST 2026
证书指纹:
	 SHA1: BC:8D:42:65:57:03:74:5D:B2:B7:E2:30:49:C5:2A:47:9B:8F:91:E6
	 SHA256: 24:BC:D7:32:6A:10:21:7C:21:7A:D5:92:C1:7C:12:1C:A3:A8:6A:AB:91:7B:54:46:81:0B:B4:DF:ED:87:BA:85
签名算法名称: SHA384withRSA
主体公共密钥算法: 2048 位 RSA 密钥
"#;

    #[test]
    fn parse_english_output() {
        let r = parse_keytool_list("/tmp/a.jks", SAMPLE_EN);
        assert_eq!(r.store_type.as_deref(), Some("PKCS12"));
        assert_eq!(r.entries.len(), 1);
        let e = &r.entries[0];
        assert_eq!(e.alias, "myapp");
        assert_eq!(e.entry_type.as_deref(), Some("PrivateKeyEntry"));
        assert_eq!(e.creation_date.as_deref(), Some("Sep 9, 2026"));
        assert_eq!(e.owner.as_deref(), Some("CN=test"));
        assert_eq!(e.issuer.as_deref(), Some("CN=test"));
        assert_eq!(e.serial_number.as_deref(), Some("b2817dba7b8103ff"));
        assert_eq!(e.valid_from.as_deref(), Some("Wed Sep 09 18:14:51 CST 2026"));
        assert_eq!(e.valid_until.as_deref(), Some("Fri Oct 09 18:14:51 CST 2026"));
        assert_eq!(e.signature_algorithm.as_deref(), Some("SHA384withRSA"));
        assert_eq!(e.key_algorithm.as_deref(), Some("2048-bit RSA key"));
        assert_eq!(e.fingerprint_sha1, "BC:8D:42:65:57:03:74:5D:B2:B7:E2:30:49:C5:2A:47:9B:8F:91:E6");
        assert_eq!(e.fingerprint_sha256, "24:BC:D7:32:6A:10:21:7C:21:7A:D5:92:C1:7C:12:1C:A3:A8:6A:AB:91:7B:54:46:81:0B:B4:DF:ED:87:BA:85");
        assert!(e.fingerprint_md5.is_empty()); // JDK 21 默认不再显示 MD5
    }

    #[test]
    fn parse_chinese_output() {
        let r = parse_keytool_list("/tmp/a.jks", SAMPLE_ZH);
        assert_eq!(r.store_type.as_deref(), Some("PKCS12"));
        assert_eq!(r.entries.len(), 1);
        let e = &r.entries[0];
        assert_eq!(e.alias, "myapp");
        assert_eq!(e.creation_date.as_deref(), Some("2026年9月9日"));
        assert_eq!(e.issuer.as_deref(), Some("CN=test"));
        assert_eq!(e.serial_number.as_deref(), Some("b2817dba7b8103ff"));
        assert_eq!(e.valid_from.as_deref(), Some("Wed Sep 09 18:14:51 CST 2026"));
        assert_eq!(e.valid_until.as_deref(), Some("Fri Oct 09 18:14:51 CST 2026"));
        assert_eq!(e.signature_algorithm.as_deref(), Some("SHA384withRSA"));
        assert_eq!(e.key_algorithm.as_deref(), Some("2048 位 RSA 密钥"));
        assert_eq!(e.fingerprint_sha1, "BC:8D:42:65:57:03:74:5D:B2:B7:E2:30:49:C5:2A:47:9B:8F:91:E6");
    }

    #[test]
    fn key_hash_matches_expected_digest() {
        // SHA1(空输入) 的标准值，校验 base64 与十六进制两条输出
        let r = key_hash_from_der("myapp", b"").unwrap_err();
        assert_eq!(r, "keytool 未输出证书内容");

        let r = key_hash_from_der("myapp", b"abc").unwrap();
        assert_eq!(r.sha1_base64, "qZk+NkcGgWq6PiVxeFDCbJzQ2J0=");
        assert_eq!(r.sha1_hex, "A9:99:3E:36:47:06:81:6A:BA:3E:25:71:78:50:C2:6C:9C:D0:D8:9D");
        assert_eq!(r.alias, "myapp");
    }
}