// 内置 Google OAuth 凭证的构建期注入（见 src/tools/gdrive_login.rs）。
// 优先级：同名环境变量 > 仓库根 secrets/.env.local（gitignored）。两者都没有时跳过，
// 编译出的程序内置客户端为空，须在界面里填写自定义客户端。
const KEYS: &[&str] = &["GDRIVE_BUILTIN_CLIENT_ID", "GDRIVE_BUILTIN_CLIENT_SECRET"];

fn main() {
    for key in KEYS {
        let value = std::env::var(key)
            .ok()
            .filter(|v| !v.is_empty())
            .or_else(|| read_secrets_env(key));
        if let Some(value) = value {
            println!("cargo:rustc-env={key}={value}");
        }
    }
    if let Some(path) = secrets_env_path() {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    tauri_build::build()
}

/// 仓库根的 secrets/.env.local（src-tauri 的上三级：app → packages → 仓库根）。
fn secrets_env_path() -> Option<std::path::PathBuf> {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").ok()?;
    let path = std::path::Path::new(&manifest)
        .join("../../../secrets/.env.local")
        .canonicalize()
        .ok()?;
    path.is_file().then_some(path)
}

fn read_secrets_env(key: &str) -> Option<String> {
    let content = std::fs::read_to_string(secrets_env_path()?).ok()?;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else { continue };
        if k.trim() != key {
            continue;
        }
        let v = v.trim().trim_matches('"').trim_matches('\'');
        if !v.is_empty() {
            return Some(v.to_string());
        }
    }
    None
}
