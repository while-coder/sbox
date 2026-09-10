//! 证书元信息与指纹（纯 Rust，无需 keytool/JDK）。
//!
//! - DER 证书元信息用 x509-parser 解析；指纹直接对证书原始字节计算
//!   （证书 SHA-1 指纹的定义就是 SHA1(DER 编码)，与 keytool -list 输出一致）
//! - 文件读取支持：PEM（含多证书链）、PKCS#7（.p7b/.p7c，含 PEM 封装的 PKCS7）、裸 DER

use base64::Engine;
use digest::Digest as _;
use md5::Md5;
use sha1::Sha1;
use sha2::Sha256;
use x509_parser::der_parser::oid::Oid;
use x509_parser::prelude::*;
use x509_parser::public_key::PublicKey;

use super::KeystoreEntry;

/// 由证书 DER 构建完整的展示条目（元信息 + 三个指纹 + 密钥散列）。
pub fn der_to_entry(alias: &str, entry_type: &str, der: &[u8]) -> Result<KeystoreEntry, String> {
    let (rem, cert) = X509Certificate::from_der(der)
        .map_err(|e| format!("解析 X.509 证书失败: {e}"))?;
    if !rem.is_empty() {
        return Err("证书 DER 后有多余数据，格式异常".into());
    }
    let (md5, sha1, sha256) = fingerprints(der);
    Ok(KeystoreEntry {
        alias: alias.to_string(),
        entry_type: Some(entry_type.to_string()),
        creation_date: None,
        owner: Some(cert.subject().to_string()),
        issuer: Some(cert.issuer().to_string()),
        serial_number: Some(cert.serial.to_str_radix(16)),
        valid_from: Some(cert.validity().not_before.to_string()),
        valid_until: Some(cert.validity().not_after.to_string()),
        signature_algorithm: Some(sig_alg_name(&cert.signature_algorithm.algorithm)),
        key_algorithm: Some(key_alg_name(&cert.tbs_certificate.subject_pki)),
        fingerprint_md5: md5,
        fingerprint_sha1: sha1,
        fingerprint_sha256: sha256,
        sha1_base64: Some(sha1_base64(der)),
    })
}

/// (MD5, SHA-1, SHA-256) 三个指纹，大写冒号分隔，对齐 keytool -list 输出格式。
pub fn fingerprints(der: &[u8]) -> (String, String, String) {
    let md5: [u8; 16] = Md5::digest(der).into();
    let sha1: [u8; 20] = Sha1::digest(der).into();
    let sha256: [u8; 32] = Sha256::digest(der).into();
    (hex_colon(&md5), hex_colon(&sha1), hex_colon(&sha256))
}

/// base64(SHA1(der))，即发布密钥散列（Facebook「Android Key Hashes」/微信「应用签名」）。
pub fn sha1_base64(der: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(Sha1::digest(der))
}

/// 从「查看指纹」解析出的冒号分隔 hex 指纹反推密钥散列（keystore 分支免二次调用 keytool）。
pub fn sha1_base64_from_hex(hex_colon: &str) -> Option<String> {
    let compact: String = hex_colon.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    if compact.len() != 40 {
        return None;
    }
    let bytes: Vec<u8> = (0..20)
        .filter_map(|i| u8::from_str_radix(&compact[i * 2..i * 2 + 2], 16).ok())
        .collect();
    if bytes.len() != 20 {
        return None;
    }
    Some(base64::engine::general_purpose::STANDARD.encode(bytes))
}

fn hex_colon(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(":")
}

/// 从证书文件读取全部证书 DER：PEM / PKCS#7（DER 或 PEM 封装）/ 裸 DER。
pub fn read_certs_from_file(path: &std::path::Path) -> Result<Vec<Vec<u8>>, String> {
    let raw = std::fs::read(path).map_err(|e| format!("读取证书文件失败: {e}"))?;
    let text = String::from_utf8_lossy(&raw);
    if text.contains("-----BEGIN") {
        return pem_certs(&text);
    }
    if raw.first() == Some(&0x30) {
        // DER 开头：先按 PKCS#7 尝试（.p7b），不匹配则按单张 DER 证书
        if let Some(certs) = pkcs7_certs(&raw) {
            return Ok(certs);
        }
        return Ok(vec![raw]);
    }
    Err("无法识别的证书文件：应为 PEM / PKCS#7（.p7b）/ DER 编码的证书".into())
}

/// 扫描文本中所有 `-----BEGIN xxx-----` 证书块；xxx 为 CERTIFICATE 或 PKCS7（Windows 导出常见）。
fn pem_certs(text: &str) -> Result<Vec<Vec<u8>>, String> {
    const MARKERS: [(&str, bool); 2] = [("CERTIFICATE", false), ("PKCS7", true)];
    let mut out = Vec::new();
    for (label, is_pkcs7) in MARKERS {
        let begin = format!("-----BEGIN {label}-----");
        let end = format!("-----END {label}-----");
        let mut rest = text;
        while let Some(start) = rest.find(&begin) {
            let body = &rest[start + begin.len()..];
            let Some(end_pos) = body.find(&end) else { return Err("PEM 块不完整（缺少 END 行）".into()) };
            let b64: String = body[..end_pos].chars().filter(|c| !c.is_whitespace()).collect();
            let der = base64::engine::general_purpose::STANDARD
                .decode(b64.as_bytes())
                .map_err(|e| format!("PEM base64 解码失败: {e}"))?;
            if is_pkcs7 {
                out.extend(pkcs7_certs(&der).ok_or("PKCS#7 内容解析失败")?);
            } else {
                out.push(der);
            }
            rest = &body[end_pos + end.len()..];
        }
    }
    if out.is_empty() {
        return Err("未找到有效的证书块".into());
    }
    Ok(out)
}

/// 从 PKCS#7 SignedData 中提取证书序列（最小 DER 遍历）。
/// 结构：ContentInfo → SEQUENCE{OID 1.2.840.113549.1.7.2, [0] SEQUENCE(SignedData)}；
/// SignedData 内 [0] IMPLICIT 成员即证书集合。x509-parser 0.16 起不再自带 pkcs7 模块。
fn pkcs7_certs(data: &[u8]) -> Option<Vec<Vec<u8>>> {
    const SIGNED_DATA_OID: &[u8] = &[0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x07, 0x02];
    let (tag, content, _) = tlv(data, 0).ok()?;
    if tag != 0x30 {
        return None;
    }
    // 第一个子元素应为 OID signedData
    let (oid_tag, oid, off) = tlv(content, 0).ok()?;
    if oid_tag != 0x06 || oid != SIGNED_DATA_OID {
        return None;
    }
    // 接下来是 [0] EXPLICIT → SEQUENCE (SignedData)
    let (wrap_tag, wrapped, _) = tlv(content, off).ok()?;
    if wrap_tag != 0xA0 {
        return None;
    }
    let (sd_tag, signed_data, _) = tlv(&wrapped, 0).ok()?;
    if sd_tag != 0x30 {
        return None;
    }
    // 遍历 SignedData 成员找 [0] IMPLICIT 证书集合
    let mut off = 0;
    while let Ok((t, c, next)) = tlv(signed_data, off) {
        off = next;
        if t == 0xA0 {
            // 内容即连续的证书 TLV（每张以 0x30 开头）
            let mut certs = Vec::new();
            let mut pos = 0;
            while let Ok((ct, _cc, cnext)) = tlv(c, pos) {
                if ct != 0x30 {
                    break;
                }
                certs.push(c[pos..cnext].to_vec());
                pos = cnext;
            }
            return (!certs.is_empty()).then_some(certs);
        }
        if next == 0 {
            break;
        }
    }
    None
}

/// 读取一个 DER TLV，返回 (tag, content 切片, 下一个元素偏移)。仅支持通用格式需要的长度编码。
fn tlv(data: &[u8], off: usize) -> Result<(u8, &[u8], usize), String> {
    if off + 2 > data.len() {
        return Err("DER 数据过短".into());
    }
    let tag = data[off];
    let mut pos = off + 1;
    let first = data[pos];
    pos += 1;
    let len = if first < 0x80 {
        first as usize
    } else {
        let n = (first & 0x7f) as usize;
        if n == 0 || n > 4 || pos + n > data.len() {
            return Err("不支持的 DER 长度编码".into());
        }
        let mut len = 0usize;
        for b in &data[pos..pos + n] {
            len = (len << 8) | *b as usize;
        }
        pos += n;
        len
    };
    if pos + len > data.len() {
        return Err("DER 长度越界".into());
    }
    Ok((tag, &data[pos..pos + len], pos + len))
}

/// 签名算法 OID → keytool 风格短名。
fn sig_alg_name(oid: &Oid) -> String {
    let arcs: Vec<u64> = oid.iter().map(|it| it.collect::<Vec<u64>>()).unwrap_or_default();
    let name = match arcs.as_slice() {
        [1, 2, 840, 113549, 1, 1, 2] => "MD5withRSA",
        [1, 2, 840, 113549, 1, 1, 4] => "SHA1withRSA",
        [1, 2, 840, 113549, 1, 1, 11] => "SHA256withRSA",
        [1, 2, 840, 113549, 1, 1, 12] => "SHA384withRSA",
        [1, 2, 840, 113549, 1, 1, 13] => "SHA512withRSA",
        [1, 2, 840, 113549, 1, 1, 10] => "SHAwithRSA/PSS",
        [1, 2, 840, 10045, 4, 1] => "SHA1withECDSA",
        [1, 2, 840, 10045, 4, 3, 2] => "SHA256withECDSA",
        [1, 2, 840, 10045, 4, 3, 3] => "SHA384withECDSA",
        [1, 2, 840, 10045, 4, 3, 4] => "SHA512withECDSA",
        [1, 3, 101, 112] => "Ed25519",
        _ => return oid.to_id_string(),
    };
    name.to_string()
}

/// 公钥算法 → keytool 风格描述（如 "2048-bit RSA key"）。
fn key_alg_name(spki: &SubjectPublicKeyInfo) -> String {
    match spki.parsed() {
        Ok(PublicKey::RSA(rsa)) => {
            // modulus 原始字节可能带符号位前导 0
            let leading = usize::from(rsa.modulus.first() == Some(&0));
            format!("{}-bit RSA key", (rsa.modulus.len() - leading) * 8)
        }
        Ok(PublicKey::EC(_)) => format!("EC 公钥（{}）", ec_curve_name(spki)),
        Ok(PublicKey::DSA(_)) => "DSA 密钥".into(),
        _ => spki.algorithm.algorithm.to_id_string(),
    }
}

fn ec_curve_name(spki: &SubjectPublicKeyInfo) -> String {
    let Some(p) = spki.algorithm.parameters.as_ref() else {
        return "未知曲线".into();
    };
    let Ok(oid) = p.clone().oid() else {
        return "未知曲线".into();
    };
    let arcs: Vec<u64> = oid.iter().map(|it| it.collect::<Vec<u64>>()).unwrap_or_default();
    match arcs.as_slice() {
        [1, 2, 840, 10045, 3, 1, 7] => "P-256".into(),
        [1, 3, 132, 0, 34] => "P-384".into(),
        [1, 3, 132, 0, 35] => "P-521".into(),
        _ => oid.to_id_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// keytool 生成的真实自签证书（CN=test，RSA-2048）的 DER base64，
    /// 指纹与散列均由独立工具（keytool -list / openssl dgst）预先算好并交叉核对。
    const CERT_DER_B64: &str = "MIICwTCCAamgAwIBAgIIQsYabAm1zq8wDQYJKoZIhvcNAQEMBQAwDzENMAsGA1UEAxMEdGVzdDAeFw0yNjA5MTAwMzQyMTVaFw0yNjEwMTAwMzQyMTVaMA8xDTALBgNVBAMTBHRlc3QwggEiMA0GCSqGSIb3DQEBAQUAA4IBDwAwggEKAoIBAQDCGnUCnS9NAwqN8SuOO9N9HqChkFK3eRXARq0g37aHsRnOaotCkv8BTMERB9X+fXGmegtIEPMonAQ6JNhSpCxGFZGUeh606G/sUKL5VRBUBvOjVjdVG8dR/eRoH22xhl4X+tZSvaBF2Rm8/Yendz1tzOK9/0ttLyMEKWtswQsATTLccRS0rCJpymS1PqVn7OhfxAgUrK2af4h2YTg0TgF/SJiHLkveoR+Mfoo8Ndwr3azNHV43bUHD1QDYiFkLIByWFFBKbPf1VeBBAnzat85i2V2sP/+i8IEcm+tfomCkL9klwek00Fbdn61h8ndsmNOXpwVrIV7Mv6KNmb7k7zOJAgMBAAGjITAfMB0GA1UdDgQWBBQecyeWG59neSCUmZVy2hCtcEM7/jANBgkqhkiG9w0BAQwFAAOCAQEAVb3vPIetWFQ6eob9ygmdGigq969fb/GNMbnTkE4H6zxL0N0D3Np/dmdZYYthgS2jRNG/leiQcBl5l/VL8NVX25a1XJAc7UmcWkw/Al+A6iSwduSjEg67Jwjh1KT4eFhxKOyjFiDFrfynL5VxVVVrpro1Woo01nmved7H5endLlnRKPl6XBYtZQggeduYSNovCYelT4uGE4NVDN6ppL1iWnHpeqKjfD2zYXY4SNR/EqwHVO24pJj4i2Lm9Zxw1ZQy6ZbQPN3X+AQ+0kPZF2/Ajulejnx/OF0y6EMjIvpHFjcTpPtSnCD0a4RM6bffB48BwLjJNaUDLY1L+VlKa35mPA==";
    const EXP_MD5: &str = "EC:9A:76:F9:B2:7C:44:02:A0:CB:89:CB:74:20:2E:7E";
    const EXP_SHA1: &str = "C9:75:9D:D5:4D:3A:1D:4E:29:3A:73:54:A3:C4:BB:80:29:8F:1B:57";
    const EXP_SHA256: &str = "FB:49:7E:AC:C1:ED:A9:59:9F:BB:A8:C2:AC:10:83:B2:A8:84:07:4A:CA:FC:75:84:2F:69:4A:1C:39:D2:1A:29";
    const EXP_SHA1_B64: &str = "yXWd1U06HU4pOnNUo8S7gCmPG1c=";

    fn cert_der() -> Vec<u8> {
        use base64::Engine as _;
        base64::engine::general_purpose::STANDARD.decode(CERT_DER_B64).unwrap()
    }

    #[test]
    fn parses_real_certificate() {
        let entry = der_to_entry("test", "X.509 证书", &cert_der()).unwrap();
        assert_eq!(entry.owner, Some("CN=test".into()));
        assert_eq!(entry.serial_number.as_deref(), Some("42c61a6c09b5ceaf"));
        assert_eq!(entry.fingerprint_md5, EXP_MD5);
        assert_eq!(entry.fingerprint_sha1, EXP_SHA1);
        assert_eq!(entry.fingerprint_sha256, EXP_SHA256);
        assert_eq!(entry.sha1_base64.as_deref(), Some(EXP_SHA1_B64));
        assert!(entry.serial_number.is_some());
        assert!(entry.valid_from.is_some());
        assert!(entry.signature_algorithm.unwrap().contains("SHA"));
    }

    #[test]
    fn sha1_base64_from_hex_round_trips() {
        let entry = der_to_entry("t", "x", &cert_der()).unwrap();
        let from_hex = sha1_base64_from_hex(entry.fingerprint_sha1.as_str()).unwrap();
        assert_eq!(from_hex, entry.sha1_base64.unwrap());
        // 长度不对的输入返回 None
        assert!(sha1_base64_from_hex("AA:BB").is_none());
    }

    #[test]
    fn reads_pem_with_multiple_certs() {
        let b64 = CERT_DER_B64;
        let pem = format!(
            "extra text\r\n-----BEGIN CERTIFICATE-----\r\n{b64}\r\n-----END CERTIFICATE-----\r\nmore\r\n-----BEGIN CERTIFICATE-----\r\n{b64}\r\n-----END CERTIFICATE-----\r\n"
        );
        let certs = pem_certs(&pem).unwrap();
        assert_eq!(certs.len(), 2);
        assert_eq!(certs[0], cert_der());
    }

    #[test]
    fn reads_pkcs7_der_with_cert_chain() {
        // 用 openssl crl2pkcs7 生成的真实 .p7b（含 CN=test / CN=test2 两张自签证书）
        const P7B_DER_B64: &str = "MIIFtwYJKoZIhvcNAQcCoIIFqDCCBaQCAQExADALBgkqhkiG9w0BBwGgggWMMIICwTCCAamgAwIBAgIIQsYabAm1zq8wDQYJKoZIhvcNAQEMBQAwDzENMAsGA1UEAxMEdGVzdDAeFw0yNjA5MTAwMzQyMTVaFw0yNjEwMTAwMzQyMTVaMA8xDTALBgNVBAMTBHRlc3QwggEiMA0GCSqGSIb3DQEBAQUAA4IBDwAwggEKAoIBAQDCGnUCnS9NAwqN8SuOO9N9HqChkFK3eRXARq0g37aHsRnOaotCkv8BTMERB9X+fXGmegtIEPMonAQ6JNhSpCxGFZGUeh606G/sUKL5VRBUBvOjVjdVG8dR/eRoH22xhl4X+tZSvaBF2Rm8/Yendz1tzOK9/0ttLyMEKWtswQsATTLccRS0rCJpymS1PqVn7OhfxAgUrK2af4h2YTg0TgF/SJiHLkveoR+Mfoo8Ndwr3azNHV43bUHD1QDYiFkLIByWFFBKbPf1VeBBAnzat85i2V2sP/+i8IEcm+tfomCkL9klwek00Fbdn61h8ndsmNOXpwVrIV7Mv6KNmb7k7zOJAgMBAAGjITAfMB0GA1UdDgQWBBQecyeWG59neSCUmZVy2hCtcEM7/jANBgkqhkiG9w0BAQwFAAOCAQEAVb3vPIetWFQ6eob9ygmdGigq969fb/GNMbnTkE4H6zxL0N0D3Np/dmdZYYthgS2jRNG/leiQcBl5l/VL8NVX25a1XJAc7UmcWkw/Al+A6iSwduSjEg67Jwjh1KT4eFhxKOyjFiDFrfynL5VxVVVrpro1Woo01nmved7H5endLlnRKPl6XBYtZQggeduYSNovCYelT4uGE4NVDN6ppL1iWnHpeqKjfD2zYXY4SNR/EqwHVO24pJj4i2Lm9Zxw1ZQy6ZbQPN3X+AQ+0kPZF2/Ajulejnx/OF0y6EMjIvpHFjcTpPtSnCD0a4RM6bffB48BwLjJNaUDLY1L+VlKa35mPDCCAsMwggGroAMCAQICCEgcoeEfADz3MA0GCSqGSIb3DQEBDAUAMBAxDjAMBgNVBAMTBXRlc3QyMB4XDTI2MDkxMDAzNDIxNloXDTI2MTAxMDAzNDIxNlowEDEOMAwGA1UEAxMFdGVzdDIwggEiMA0GCSqGSIb3DQEBAQUAA4IBDwAwggEKAoIBAQCFmHotvXejHBTzLfn/Z2nbUpOi8y3xme+QqZLY/MsgfIOzGj6os3jvgQZtzWgTRJlJbKnUpG6qFwyrXPydQFWI67WECQLdX/HoqE31La73Hv0wbmH/UZCsKuC1v2916NCN7Ycsvqmr/yh9Rhf48e8bwbgRQ3I23KiCMSf1verf1O6N2Nbkaja2fvFQ4CM7zhOCn+xP/ir2cDHy/MqWVKECQn4Qd3N1yk2oP14rnS6aAdETE2P0nvtrO5oC17LETUZNjnQ2E+nXEvB/nOusjhd8j2iu6jPfbO7UbKb/rgIMCbxGaXlLNL2Hejptr2+q5DW9sikFCUzWyh+r7oABEaGVAgMBAAGjITAfMB0GA1UdDgQWBBRw8DjazTcHQuX8rUd9oQiAm1x0ijANBgkqhkiG9w0BAQwFAAOCAQEAejl1qvpqZgf09UR5xFaL8nXrXGxrPwQ9duHjgSC1+Y3dKXEtO0Pf9Xn8HyU22zaPQScR4tykBQWoE/9ueeuRZTM18pOSGFOP4UXsQeiFBAgSWJpCLyWwQCa/vugPmOREdn5L90brgjoV7OpmkSP4O10N0ocMrs4PeH+AkXQZ5yyD/c29AvrIZvBFEaZCAewifrKEkrXtzMyDDmhRXR2y6h+V/C4aBMatIO4XrK5MyhSRY5zMug1t8RPoW+9ZZNHd4QL/gJn9c1qBTWrypZlsXXkyBjh8ZUBTWXOOM9Np5Kvk121BY31FkYmXk9CqhyBLOmfP10+2jDxo737Ssj/KVzEA";
        use base64::Engine as _;
        let der = base64::engine::general_purpose::STANDARD.decode(P7B_DER_B64).unwrap();
        let certs = pkcs7_certs(&der).expect("应解析出证书");
        assert_eq!(certs.len(), 2);
        // 解析出的每张证书都能被 x509-parser 认出
        for cert in &certs {
            assert!(X509Certificate::from_der(cert).is_ok());
        }
        // 非 PKCS#7 的 DER 返回 None（走单证书分支）
        assert!(pkcs7_certs(&cert_der()).is_none());
    }
}
