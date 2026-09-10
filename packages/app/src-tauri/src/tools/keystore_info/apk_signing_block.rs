//! APK Signing Block（v2/v3/v3.1）解析：从 APK/AAB 中提取签名证书 DER。
//!
//! 文件布局：`[zip entries][APK Signing Block][Central Directory][EOCD]`，
//! 签名块紧贴 Central Directory 之前。结构参照 Android 官方格式文档：
//! - 外层长度前缀均为 u64 LE；块尾为 [size2: u64][magic: "APK Sig Block 42"]
//! - 块内是若干 pair：[pair_len: u64][id: u32][value]，按 ID 查找目标 scheme，
//!   跳过 padding（0x42726577）与渠道自定义等未知 pair
//! - v2/v3/v3.1 的 value 内部才是 u32 LE 长度前缀，逐级嵌套：
//!   signer 序列 → signer{signed_data, signatures, public_key}
//!   → signed_data{digests, certificates, [v3: minSDK u32, maxSDK u32,] attrs}
//!   v3 的 minSDK/maxSDK 在 certificates 之后，取证书只需前两个字段，
//!   因此三种 scheme 共用同一 signer 解析器

/// 从整个 APK/AAB 文件字节中提取签名证书，返回 (scheme 名, 证书 DER 列表)。
/// scheme 优先级 v3.1 > v3 > v2：v3.1 仅在密钥轮换面向 Android 13+ 时存在，
/// 此时 v3.1 里的才是现行生效的证书。
pub fn extract_certificates(data: &[u8]) -> Result<(&'static str, Vec<Vec<u8>>), String> {
    let (_, cd_offset) = find_eocd(data)?;
    let pairs = read_signing_block(data, cd_offset)?;
    let pick = |id: u32| pairs.iter().find(|(i, _)| *i == id).map(|(_, v)| v);
    for (id, scheme) in [(ID_V3_1, "v3.1"), (ID_V3, "v3"), (ID_V2, "v2")] {
        if let Some(value) = pick(id) {
            return Ok((scheme, certs_from_scheme(value)?));
        }
    }
    Err("未找到 v2/v3 签名块：文件可能未签名，或仅有 v1 (JAR) 签名（Android 7+ 的包通常都有 v2/v3）".into())
}

const MAGIC: &[u8; 16] = b"APK Sig Block 42";
const ID_V2: u32 = 0x7109_871a;
const ID_V3: u32 = 0xf053_68c0;
const ID_V3_1: u32 = 0x1b93_ad61;

/// 定位 EOCD，返回 (eocd 偏移, Central Directory 偏移)。
/// 从后向前扫描并要求 comment 长度自洽，防止 zip comment 中被注入伪造 EOCD（加固壳常见）。
fn find_eocd(data: &[u8]) -> Result<(usize, usize), String> {
    if data.len() < 22 {
        return Err("文件太小，不是有效的 APK/AAB".into());
    }
    let lo = data.len().saturating_sub(22 + 65535);
    for i in (lo..=data.len() - 22).rev() {
        if data[i..i + 4] != [0x50, 0x4B, 0x05, 0x06] {
            continue;
        }
        let comment_len = u16::from_le_bytes([data[i + 20], data[i + 21]]) as usize;
        if i + 22 + comment_len != data.len() {
            continue; // comment 长度不自洽，视为伪造记录
        }
        let cd_offset = u32::from_le_bytes(data[i + 16..i + 20].try_into().unwrap()) as usize;
        if cd_offset == 0xFFFF_FFFF {
            return Err("该文件使用 zip64 格式，暂不支持（正常 APK/AAB 不会出现）".into());
        }
        return Ok((i, cd_offset));
    }
    Err("未找到 ZIP EOCD 记录，文件可能损坏或不是 APK/AAB".into())
}

/// 读取 CD 之前的签名块，解析出全部 pair。
fn read_signing_block(data: &[u8], cd_offset: usize) -> Result<Vec<(u32, Vec<u8>)>, String> {
    if cd_offset < 24 || cd_offset > data.len() {
        return Err("Central Directory 偏移异常，文件可能损坏".into());
    }
    if &data[cd_offset - 16..cd_offset] != MAGIC {
        return Err("未找到 APK 签名块（APK Sig Block 42）。文件可能未签名，或仅有 v1 (JAR) 签名".into());
    }
    let size2 = u64::from_le_bytes(data[cd_offset - 24..cd_offset - 16].try_into().unwrap());
    // size2 计入 pairs + 尾部 size 字段 + magic，不含头部 size1 字段（8 字节）
    let block_start = cd_offset
        .checked_sub(8 + size2 as usize)
        .ok_or("APK 签名块大小异常，文件可能损坏")?;
    let size1 = u64::from_le_bytes(data[block_start..block_start + 8].try_into().unwrap());
    if size1 != size2 {
        return Err("APK 签名块头尾 size 字段不一致，文件可能被篡改或损坏".into());
    }

    let mut pairs = Vec::new();
    let mut p = block_start + 8;
    let pairs_end = cd_offset - 24; // 末尾 24 字节是 size2 + magic
    while p + 8 <= pairs_end {
        let pair_len = u64::from_le_bytes(data[p..p + 8].try_into().unwrap()) as usize;
        if pair_len < 4 || p + 8 + pair_len > pairs_end {
            return Err("APK 签名块 pair 长度越界，文件可能损坏".into());
        }
        let id = u32::from_le_bytes(data[p + 8..p + 12].try_into().unwrap());
        pairs.push((id, data[p + 12..p + 8 + pair_len].to_vec()));
        p += 8 + pair_len;
    }
    Ok(pairs)
}

/// 解析 u32 LE 长度前缀序列：开头 u32 总长，之后逐个 [u32 元素长][元素]。
fn parse_len_seq(v: &[u8]) -> Result<Vec<&[u8]>, String> {
    if v.len() < 4 {
        return Err("长度前缀序列过短".into());
    }
    let total = u32::from_le_bytes(v[..4].try_into().unwrap()) as usize;
    let end = 4 + total;
    if end > v.len() {
        return Err("长度前缀序列总长越界".into());
    }
    let mut out = Vec::new();
    let mut off = 4;
    while off + 4 <= end {
        let n = u32::from_le_bytes(v[off..off + 4].try_into().unwrap()) as usize;
        if off + 4 + n > end {
            return Err("长度前缀序列元素越界".into());
        }
        out.push(&v[off + 4..off + 4 + n]);
        off += 4 + n;
    }
    Ok(out)
}

/// 从某个 scheme 的 value 中提取所有 signer 的证书 DER。
/// signer = [signed_data][signatures][public_key]，三个 u32 前缀字段；
/// signed_data 的第 2 个字段即证书序列（v2/v3/v3.1 一致）。
fn certs_from_scheme(value: &[u8]) -> Result<Vec<Vec<u8>>, String> {
    let mut certs = Vec::new();
    for signer in parse_len_seq(value)? {
        let fields = read_prefixed_fields(signer, 3)?;
        let signed_data = read_prefixed_fields(fields[0], 3)?;
        let cert_seq = parse_len_seq(signed_data[1])?;
        certs.extend(cert_seq.iter().map(|c| c.to_vec()));
    }
    if certs.is_empty() {
        return Err("签名块中未找到任何证书".into());
    }
    Ok(certs)
}

/// 连续读取 count 个 [u32 长度前缀] 的字段。
fn read_prefixed_fields(v: &[u8], count: usize) -> Result<Vec<&[u8]>, String> {
    let mut out = Vec::with_capacity(count);
    let mut off = 0;
    for _ in 0..count {
        if off + 4 > v.len() {
            return Err("字段长度前缀越界".into());
        }
        let n = u32::from_le_bytes(v[off..off + 4].try_into().unwrap()) as usize;
        if off + 4 + n > v.len() {
            return Err("字段内容越界".into());
        }
        out.push(&v[off + 4..off + 4 + n]);
        off += 4 + n;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试用的最小「证书」字节（无需真实证书，只验证结构解析）
    fn dummy_cert(tag: u8) -> Vec<u8> {
        vec![0x30, 0x03, 0x02, 0x01, tag]
    }

    /// 构造 u32 前缀字段序列
    fn prefixed(fields: &[&[u8]]) -> Vec<u8> {
        let mut out = Vec::new();
        for f in fields {
            out.extend_from_slice(&(f.len() as u32).to_le_bytes());
            out.extend_from_slice(f);
        }
        out
    }

    /// u32 前缀序列：开头 u32 总长 + 逐个元素
    fn len_seq(items: &[Vec<u8>]) -> Vec<u8> {
        let refs: Vec<&[u8]> = items.iter().map(|v| v.as_slice()).collect();
        let body = prefixed(&refs);
        let mut out = (body.len() as u32).to_le_bytes().to_vec();
        out.extend_from_slice(&body);
        out
    }

    /// 构造某 scheme 的 value：1 个 signer，signed_data 含 digests + certificates + attrs 三个字段
    fn scheme_value(signer_certs: &[Vec<u8>]) -> Vec<u8> {
        let digests = [0u8; 4]; // 空序列（总长 0）
        let attrs = [0u8; 4];
        let signed_data = prefixed(&[&digests, &len_seq(signer_certs), &attrs]);
        let signer = prefixed(&[&signed_data, &[0, 0, 0, 0], &[0, 0, 0, 0]]);
        len_seq(&[signer])
    }

    /// 合成一个「APK」：junk + [size1][pairs][size2][magic] + CD 占位 + EOCD
    fn build_fake_apk(pairs: &[(u32, Vec<u8>)]) -> Vec<u8> {
        build_fake_apk_with_comment(pairs, &[])
    }

    /// comment 非空时附带 zip 注释（位于真实 EOCD 之后）
    fn build_fake_apk_with_comment(pairs: &[(u32, Vec<u8>)], comment: &[u8]) -> Vec<u8> {
        let mut pairs_bytes = Vec::new();
        for (id, value) in pairs {
            let len = 4 + value.len() as u64;
            pairs_bytes.extend_from_slice(&len.to_le_bytes());
            pairs_bytes.extend_from_slice(&id.to_le_bytes());
            pairs_bytes.extend_from_slice(value);
        }
        let mut out = b"PK junk entries...".to_vec();
        out.extend_from_slice(&((pairs_bytes.len() as u64 + 24).to_le_bytes())); // size1
        out.extend_from_slice(&pairs_bytes);
        out.extend_from_slice(&((pairs_bytes.len() as u64 + 24).to_le_bytes())); // size2
        out.extend_from_slice(MAGIC);
        let cd_offset = out.len() as u32;
        out.extend_from_slice(b"fake central directory........");
        out.extend_from_slice(&eocd(cd_offset, comment.len() as u16));
        out.extend_from_slice(comment);
        out
    }

    /// 22 字节的 EOCD 记录
    fn eocd(cd_offset: u32, comment_len: u16) -> Vec<u8> {
        let mut e = vec![0x50, 0x4B, 0x05, 0x06];
        e.extend_from_slice(&[0, 0]); // disk number
        e.extend_from_slice(&[0, 0]); // CD start disk
        e.extend_from_slice(&[0, 0]); // entries on this disk
        e.extend_from_slice(&[0, 0]); // total entries
        e.extend_from_slice(&0u32.to_le_bytes()); // CD size
        e.extend_from_slice(&cd_offset.to_le_bytes());
        e.extend_from_slice(&comment_len.to_le_bytes());
        e
    }

    #[test]
    fn extracts_v3_over_v2_and_skips_unknown_pairs() {
        let v2 = scheme_value(&[dummy_cert(1)]);
        let v3 = scheme_value(&[dummy_cert(2)]);
        let padding = vec![0u8; 4096];
        let unknown = vec![0xAA; 16];
        let data = build_fake_apk(&[
            (ID_V2, v2),
            (0x4272_6577, padding), // verity padding block
            (0xDEAD_BEEF, unknown), // 渠道自定义
            (ID_V3, v3),
        ]);
        let (scheme, certs) = extract_certificates(&data).unwrap();
        assert_eq!(scheme, "v3");
        assert_eq!(certs, vec![dummy_cert(2)]);
    }

    #[test]
    fn extracts_v3_1_over_v3() {
        let v3 = scheme_value(&[dummy_cert(2)]);
        let v31 = scheme_value(&[dummy_cert(3)]);
        let data = build_fake_apk(&[(ID_V3, v3), (ID_V3_1, v31)]);
        let (scheme, certs) = extract_certificates(&data).unwrap();
        assert_eq!(scheme, "v3.1");
        assert_eq!(certs, vec![dummy_cert(3)]);
    }

    #[test]
    fn v2_only_works_and_collects_all_signers() {
        let v2 = scheme_value(&[dummy_cert(1), dummy_cert(4)]);
        let data = build_fake_apk(&[(ID_V2, v2)]);
        let (scheme, certs) = extract_certificates(&data).unwrap();
        assert_eq!(scheme, "v2");
        assert_eq!(certs, vec![dummy_cert(1), dummy_cert(4)]);
    }

    #[test]
    fn handles_zip_comment_with_forged_eocd() {
        let v3 = scheme_value(&[dummy_cert(2)]);
        // 注释开头放伪造 EOCD（comment_len 不自洽），应被跳过
        let mut comment = vec![0x50, 0x4B, 0x05, 0x06];
        comment.extend_from_slice(&[0u8; 18]);
        comment.extend_from_slice(b"rest of comment");
        let data = build_fake_apk_with_comment(&[(ID_V3, v3)], &comment);
        let (scheme, certs) = extract_certificates(&data).unwrap();
        assert_eq!(scheme, "v3");
        assert_eq!(certs, vec![dummy_cert(2)]);
    }

    #[test]
    fn rejects_missing_or_broken_blocks() {
        // 无 EOCD
        assert!(extract_certificates(b"not a zip").is_err());

        // 构造 pairs + 签名块头部，供下几个用例复用
        let value = scheme_value(&[dummy_cert(1)]);
        let mut pairs_bytes = Vec::new();
        pairs_bytes.extend_from_slice(&((4 + value.len()) as u64).to_le_bytes());
        pairs_bytes.extend_from_slice(&ID_V2.to_le_bytes());
        pairs_bytes.extend_from_slice(&value);
        let block_tail = |mut out: Vec<u8>| {
            out.extend_from_slice(&((pairs_bytes.len() as u64 + 24).to_le_bytes()));
            out.extend_from_slice(MAGIC);
            out.extend_from_slice(&[0u8; 30]); // CD 占位
            out
        };

        // 无签名块（未签名或仅 v1）：CD 前是普通 junk，没有 magic
        let mut unsigned = vec![0x41u8; 40]; // junk ≥ 24 字节
        unsigned.extend_from_slice(&[0u8; 30]); // CD 占位
        let full = append_eocd(unsigned, 40); // CD 从偏移 40 开始
        let err = extract_certificates(&full).unwrap_err();
        assert!(err.contains("未找到 APK 签名块"), "{err}");

        // size1 != size2
        let mut tampered = b"PK junk".to_vec();
        tampered.extend_from_slice(&99u64.to_le_bytes()); // 错误的 size1
        tampered.extend_from_slice(&pairs_bytes);
        let tampered = block_tail(tampered);
        let cd_offset = (tampered.len() - 30) as u32;
        let full = append_eocd(tampered, cd_offset);
        let err = extract_certificates(&full).unwrap_err();
        assert!(err.contains("size 字段不一致"), "{err}");

        // zip64（CD 偏移 0xFFFFFFFF）
        let zip64 = block_tail(b"PK junk".to_vec());
        let full = append_eocd(zip64, 0xFFFF_FFFF);
        let err = extract_certificates(&full).unwrap_err();
        assert!(err.contains("zip64"), "{err}");
    }

    /// 在尾部追加指向 cd_offset 的 EOCD（comment 为空时 EOCD 即文件尾）
    fn append_eocd(mut data: Vec<u8>, cd_offset: u32) -> Vec<u8> {
        data.extend_from_slice(&eocd(cd_offset, 0));
        data
    }
}
