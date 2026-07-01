#!/usr/bin/env bash
# 用自签名证书 sbox-dev 给 macOS 构建产物签名，让「录屏」权限跨重编译保持授权。
#
# 背景：macOS 的 TCC 权限绑定应用的代码签名 Designated Requirement。
# 未签名 / ad-hoc 的 app 每次重编译 cdhash 都变，会被当成新 app 反复弹窗。
# 用固定证书签名后，DR 变为 identifier + 证书哈希，重编译不变，只需授权一次。
#
# 用法：
#   scripts/mac-sign.sh            # 自动签名存在的 debug/release 二进制与 .app
#   scripts/mac-sign.sh <路径>      # 签名指定的 .app 或可执行文件
#
# 注意：`tauri build` 已在 tauri.conf.json 里配置了自动签名，一般无需手动。
#       本脚本主要用于 `tauri dev`（裸二进制不会自动签名）——改动 Rust 代码
#       重编译后重跑一次即可（改前端热重载不会重建 Rust，无需重签）。
set -euo pipefail

IDENTITY="sbox-dev"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_DIR="$ROOT/packages/app/src-tauri/target"

sign() {
  [ -e "$1" ] || return 0
  codesign --force --deep -s "$IDENTITY" "$1"
  echo "已签名: $1"
}

if [ $# -ge 1 ]; then
  sign "$1"
else
  # tauri dev 实际运行的裸二进制
  sign "$TARGET_DIR/debug/sbox"
  sign "$TARGET_DIR/release/sbox"
  # 打包产物（若存在）
  sign "$TARGET_DIR/debug/bundle/macos/sbox.app"
  sign "$TARGET_DIR/release/bundle/macos/sbox.app"
fi
