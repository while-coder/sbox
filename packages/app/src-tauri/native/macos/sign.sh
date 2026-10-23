#!/usr/bin/env bash
# 创建/使用自签名证书 sbox-dev 给 macOS 构建产物签名，让「录屏」权限跨重编译保持授权。
#
# 背景：macOS 的 TCC 权限绑定应用的代码签名 Designated Requirement。
# 未签名 / ad-hoc 的 app 每次重编译 cdhash 都变，会被当成新 app 反复弹窗。
# 用固定证书签名后，DR 变为 identifier + 证书哈希，重编译不变，只需授权一次。
#
# 用法：
#   packages/app/src-tauri/native/macos/sign.sh --setup    # 首次创建并安装长期自签名证书（无需 Apple 账号）
#   packages/app/src-tauri/native/macos/sign.sh --check    # 检查当前钥匙串是否有可用签名身份
#   packages/app/src-tauri/native/macos/sign.sh --dev      # 构建并启动固定签名开发包
#   packages/app/src-tauri/native/macos/sign.sh --import-ci # CI 导入签名证书
#   packages/app/src-tauri/native/macos/sign.sh            # 自动签名存在的 debug/release 二进制与 .app
#   packages/app/src-tauri/native/macos/sign.sh <路径>      # 签名指定的 .app 或可执行文件
#
# `pnpm --filter @sbox/app macos:build` 会让 Tauri 使用这张证书签名打包产物；
# 本脚本的直接签名模式用于已有二进制以及排查签名问题。
set -euo pipefail

IDENTITY="sbox-dev"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
TARGET_DIR="$ROOT/packages/app/src-tauri/target"
SIGNING_DIR="${SBOX_SIGNING_DIR:-$HOME/.config/sbox/signing}"
P12_PATH="$SIGNING_DIR/$IDENTITY.p12"

has_identity() {
  security find-identity -v -p codesigning 2>/dev/null | grep -Fq "\"$IDENTITY\""
}

check_identity() {
  if ! has_identity; then
    echo "未找到可用的 macOS 签名身份: $IDENTITY" >&2
    echo "请先运行: packages/app/src-tauri/native/macos/sign.sh --setup" >&2
    return 1
  fi
  security find-identity -v -p codesigning | grep -F "\"$IDENTITY\""
}

setup_identity() {
  if has_identity; then
    echo "签名身份已存在: $IDENTITY"
    check_identity
    return 0
  fi

  command -v openssl >/dev/null || { echo "缺少 openssl" >&2; return 1; }
  command -v security >/dev/null || { echo "缺少 macOS security 工具" >&2; return 1; }

  mkdir -p "$SIGNING_DIR"
  chmod 700 "$SIGNING_DIR"
  if [ -e "$P12_PATH" ]; then
    echo "证书文件已存在但钥匙串中没有对应身份，请先处理: $P12_PATH" >&2
    return 1
  fi

  local key_path="$SIGNING_DIR/$IDENTITY.key"
  local cert_path="$SIGNING_DIR/$IDENTITY.crt"

  if [ -z "${SBOX_CERTIFICATE_PASSWORD:-}" ]; then
    read -r -s -p "设置 $IDENTITY.p12 的导出密码（用于 GitHub Secret）: " SBOX_CERTIFICATE_PASSWORD
    echo
  fi
  if [ -z "$SBOX_CERTIFICATE_PASSWORD" ]; then
    echo "证书导出密码不能为空" >&2
    return 1
  fi
  export SBOX_CERTIFICATE_PASSWORD

  openssl req -x509 -newkey rsa:3072 -sha256 -days 3650 -nodes \
    -config /dev/stdin \
    -keyout "$key_path" \
    -out "$cert_path" <<'OPENSSL_CONFIG'
[req]
prompt = no
distinguished_name = subject
x509_extensions = codesign

[subject]
CN = sbox-dev
O = sbox local development

[codesign]
basicConstraints = critical, CA:true
keyUsage = critical, digitalSignature, keyCertSign
extendedKeyUsage = codeSigning
subjectKeyIdentifier = hash
authorityKeyIdentifier = keyid:always
OPENSSL_CONFIG
  openssl pkcs12 -export \
    -name "$IDENTITY" \
    -inkey "$key_path" \
    -in "$cert_path" \
    -out "$P12_PATH" \
    -passout env:SBOX_CERTIFICATE_PASSWORD
  chmod 600 "$P12_PATH"

  security import "$P12_PATH" \
    -k "$HOME/Library/Keychains/login.keychain-db" \
    -P "$SBOX_CERTIFICATE_PASSWORD" \
    -T /usr/bin/codesign
  security add-trusted-cert -r trustRoot -p codeSign \
    -k "$HOME/Library/Keychains/login.keychain-db" "$cert_path"

  rm -f "$key_path" "$cert_path"
  unset SBOX_CERTIFICATE_PASSWORD
  check_identity

  echo
  echo "自签名证书已保存到: $P12_PATH"
  echo "请把它的 Base64 内容保存为 GitHub Secret APPLE_CERTIFICATE，"
  echo "并把刚才的导出密码保存为 APPLE_CERTIFICATE_PASSWORD。"
  echo "macOS 可运行: base64 -i '$P12_PATH' | pbcopy"
}

sign() {
  [ -e "$1" ] || return 0
  check_identity >/dev/null
  codesign --force --deep -s "$IDENTITY" "$1"
  echo "已签名: $1"
}

signed_dev() {
  check_identity >/dev/null
  (
    cd "$ROOT/packages/app"
    APPLE_SIGNING_IDENTITY="$IDENTITY" pnpm exec tauri build --debug --bundles app
    open "$TARGET_DIR/debug/bundle/macos/sbox.app"
  )
}

import_ci() {
  : "${APPLE_CERTIFICATE:?缺少 GitHub Secret APPLE_CERTIFICATE}"
  : "${APPLE_CERTIFICATE_PASSWORD:?缺少 GitHub Secret APPLE_CERTIFICATE_PASSWORD}"

  local ci_keychain_password="sbox-ci-keychain"
  local ci_keychain_path="$RUNNER_TEMP/sbox-signing.keychain-db"
  local ci_p12_path="$RUNNER_TEMP/sbox-signing.p12"
  local ci_cert_path="$RUNNER_TEMP/sbox-signing.crt"

  printf '%s' "$APPLE_CERTIFICATE" | base64 -D > "$ci_p12_path"

  echo "正在创建临时签名钥匙串..."
  security create-keychain -p "$ci_keychain_password" "$ci_keychain_path"
  security set-keychain-settings -lut 21600 "$ci_keychain_path"
  security unlock-keychain -p "$ci_keychain_password" "$ci_keychain_path"

  echo "正在导入签名身份..."
  security import "$ci_p12_path" -k "$ci_keychain_path" \
    -P "$APPLE_CERTIFICATE_PASSWORD" -T /usr/bin/codesign
  # 从已导入的钥匙串读取证书，避免 OpenSSL 3 拒绝旧 P12 中的 RC2 加密。
  security find-certificate -c "$IDENTITY" -p "$ci_keychain_path" > "$ci_cert_path"

  echo "正在信任自签名证书..."
  sudo -n security add-trusted-cert -d -r trustRoot -p codeSign \
    -k /Library/Keychains/System.keychain "$ci_cert_path"

  echo "正在授权 codesign 访问私钥..."
  security set-key-partition-list -S apple-tool:,apple:,codesign: \
    -s -k "$ci_keychain_password" "$ci_keychain_path"

  local existing_keychains
  existing_keychains="$(security list-keychains -d user | tr -d '"')"
  # shellcheck disable=SC2086
  security list-keychains -d user -s "$ci_keychain_path" $existing_keychains

  if ! security find-identity -v -p codesigning "$ci_keychain_path" | grep -Fq "\"$IDENTITY\""; then
    echo "导入后仍未找到有效签名身份: $IDENTITY" >&2
    security find-identity -v -p codesigning "$ci_keychain_path" >&2
    return 1
  fi

  echo "APPLE_SIGNING_IDENTITY=$IDENTITY" >> "$GITHUB_ENV"
  echo "已导入固定 macOS 签名身份: $IDENTITY"
}

case "${1:-}" in
  --setup) setup_identity ;;
  --check) check_identity ;;
  --dev) signed_dev ;;
  --import-ci) import_ci ;;
  --help|-h)
    echo "Usage: bash sign.sh [--setup|--check|--dev|--import-ci|<path>]"
    ;;
  --*) echo "未知参数: $1" >&2; exit 1 ;;
  *)
    if [ $# -ge 1 ]; then
      sign "$1"
    else
      sign "$TARGET_DIR/debug/sbox"
      sign "$TARGET_DIR/release/sbox"
      while IFS= read -r -d '' app; do
        sign "$app"
      done < <(find "$TARGET_DIR" -type d -path '*/bundle/macos/sbox.app' -print0 2>/dev/null)
    fi
    ;;
esac
