#!/usr/bin/env bash
# 在 GitHub macOS runner 中导入长期自签名证书。证书必须跨发布保持不变，
# 否则 macOS TCC 仍会把新版本识别成另一个应用。
set -euo pipefail

: "${APPLE_CERTIFICATE:?缺少 GitHub Secret APPLE_CERTIFICATE}"
: "${APPLE_CERTIFICATE_PASSWORD:?缺少 GitHub Secret APPLE_CERTIFICATE_PASSWORD}"

IDENTITY="sbox-dev"
KEYCHAIN_PASSWORD="sbox-ci-keychain"
KEYCHAIN_PATH="$RUNNER_TEMP/sbox-signing.keychain-db"
P12_PATH="$RUNNER_TEMP/sbox-signing.p12"
CERT_PATH="$RUNNER_TEMP/sbox-signing.crt"

printf '%s' "$APPLE_CERTIFICATE" | base64 -D > "$P12_PATH"
openssl pkcs12 -in "$P12_PATH" -nokeys \
  -passin env:APPLE_CERTIFICATE_PASSWORD -out "$CERT_PATH"

security create-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN_PATH"
security set-keychain-settings -lut 21600 "$KEYCHAIN_PATH"
security unlock-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN_PATH"
security import "$P12_PATH" -k "$KEYCHAIN_PATH" \
  -P "$APPLE_CERTIFICATE_PASSWORD" -T /usr/bin/codesign
security add-trusted-cert -r trustRoot -p codeSign -k "$KEYCHAIN_PATH" "$CERT_PATH"
security set-key-partition-list -S apple-tool:,apple:,codesign: \
  -s -k "$KEYCHAIN_PASSWORD" "$KEYCHAIN_PATH"

existing_keychains="$(security list-keychains -d user | tr -d '"')"
# shellcheck disable=SC2086
security list-keychains -d user -s "$KEYCHAIN_PATH" $existing_keychains

if ! security find-identity -v -p codesigning "$KEYCHAIN_PATH" | grep -Fq "\"$IDENTITY\""; then
  echo "导入后仍未找到有效签名身份: $IDENTITY" >&2
  security find-identity -v -p codesigning "$KEYCHAIN_PATH" >&2
  exit 1
fi

echo "APPLE_SIGNING_IDENTITY=$IDENTITY" >> "$GITHUB_ENV"
echo "已导入固定 macOS 签名身份: $IDENTITY"
