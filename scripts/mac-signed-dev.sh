#!/usr/bin/env bash
# 构建并启动固定签名的 debug App。macOS 截图权限会绑定这个稳定身份，
# 适合需要反复修改 Rust 代码并验证截图的场景。
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
"$ROOT/scripts/mac-sign.sh" --check >/dev/null

cd "$ROOT/packages/app"
APPLE_SIGNING_IDENTITY=sbox-dev pnpm exec tauri build --debug --bundles app
open "$ROOT/packages/app/src-tauri/target/debug/bundle/macos/sbox.app"
