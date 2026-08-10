use serde::{Deserialize, Serialize};
use ssh_key::{rand_core::OsRng, Algorithm, HashAlg, LineEnding, PrivateKey};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SshKeyGenerateResult {
    pub key_type: String,
    pub private_key_path: String,
    pub public_key_path: String,
    pub private_key: String,
    pub public_key: String,
    pub fingerprint: String,
    pub encrypted: bool,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SshKeyType {
    Ed25519,
    Rsa4096,
}

impl SshKeyType {
    fn algorithm(self) -> Algorithm {
        match self {
            Self::Ed25519 => Algorithm::Ed25519,
            Self::Rsa4096 => Algorithm::Rsa { hash: None },
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Ed25519 => "Ed25519",
            Self::Rsa4096 => "RSA 4096",
        }
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn ssh_key_generate(
    path: String,
    key_type: SshKeyType,
    comment: String,
    passphrase: Option<String>,
) -> Result<SshKeyGenerateResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        generate_key_pair(path, key_type, comment, passphrase)
    })
    .await
    .map_err(|e| format!("生成任务失败: {e}"))?
}

fn generate_key_pair(
    path: String,
    key_type: SshKeyType,
    comment: String,
    passphrase: Option<String>,
) -> Result<SshKeyGenerateResult, String> {
    let target = PathBuf::from(path.trim());
    if target.as_os_str().is_empty() {
        return Err("请选择私钥保存位置".into());
    }

    if let Some(parent) = target.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
        }
    }

    let passphrase = passphrase.filter(|value| !value.is_empty());
    let encrypted = passphrase.is_some();
    let mut key = PrivateKey::random(&mut OsRng, key_type.algorithm())
        .map_err(|e| format!("生成 {} 密钥失败: {e}", key_type.label()))?;
    key.set_comment(comment.trim());

    let public_key = key.public_key().clone();
    let key = match passphrase.as_deref() {
        Some(password) => key
            .encrypt(&mut OsRng, password)
            .map_err(|e| format!("加密私钥失败: {e}"))?,
        None => key,
    };

    let private_text = key
        .to_openssh(LineEnding::LF)
        .map_err(|e| format!("编码私钥失败: {e}"))?;
    let public_text = public_key
        .to_openssh()
        .map_err(|e| format!("编码公钥失败: {e}"))?;
    let fingerprint = public_key.fingerprint(HashAlg::Sha256).to_string();
    let public_path = PathBuf::from(format!("{}.pub", target.to_string_lossy()));

    write_private_file(&target, private_text.as_bytes())?;
    if let Err(error) = std::fs::write(&public_path, format!("{public_text}\n")) {
        return Err(format!(
            "私钥已保存到 {}，但保存公钥失败: {error}",
            target.display()
        ));
    }

    Ok(SshKeyGenerateResult {
        key_type: key_type.label().into(),
        private_key_path: target.to_string_lossy().into_owned(),
        public_key_path: public_path.to_string_lossy().into_owned(),
        private_key: private_text.to_string(),
        public_key: public_text,
        fingerprint,
        encrypted,
    })
}

fn write_private_file(path: &Path, contents: &[u8]) -> Result<(), String> {
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }

    let mut file = options
        .open(path)
        .map_err(|e| format!("保存私钥失败: {e}"))?;
    file.write_all(contents)
        .map_err(|e| format!("写入私钥失败: {e}"))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
            .map_err(|e| format!("设置私钥权限失败: {e}"))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_key_round_trips_with_password() {
        let mut key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519).unwrap();
        key.set_comment("test@example.com");
        let encrypted = key.encrypt(&mut OsRng, "test-passphrase").unwrap();
        let encoded = encrypted.to_openssh(LineEnding::LF).unwrap();
        let parsed = PrivateKey::from_openssh(encoded.as_bytes()).unwrap();

        assert!(parsed.is_encrypted());
        let decrypted = parsed.decrypt("test-passphrase").unwrap();
        assert_eq!(decrypted.comment(), "test@example.com");
        assert_eq!(decrypted.algorithm(), Algorithm::Ed25519);
    }

    #[test]
    fn rsa_4096_key_uses_requested_size() {
        let key = PrivateKey::random(&mut OsRng, SshKeyType::Rsa4096.algorithm()).unwrap();
        let rsa = key.key_data().rsa().unwrap();
        assert_eq!(rsa.public.n.as_positive_bytes().unwrap().len(), 512);

        let encrypted = key.encrypt(&mut OsRng, "test-passphrase").unwrap();
        let encoded = encrypted.to_openssh(LineEnding::LF).unwrap();
        let parsed = PrivateKey::from_openssh(encoded.as_bytes()).unwrap();
        let decrypted = parsed.decrypt("test-passphrase").unwrap();
        assert!(decrypted.key_data().rsa().is_some());
    }

    #[test]
    fn key_type_api_values_are_stable() {
        assert!(matches!(
            serde_json::from_str::<SshKeyType>("\"ed25519\"").unwrap(),
            SshKeyType::Ed25519
        ));
        assert!(matches!(
            serde_json::from_str::<SshKeyType>("\"rsa4096\"").unwrap(),
            SshKeyType::Rsa4096
        ));
    }
}
