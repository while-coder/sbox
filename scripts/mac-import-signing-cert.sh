#!/usr/bin/env bash
# Compatibility entry for the shared release workflow; signing logic lives in native/macos.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
exec bash "$ROOT/packages/app/src-tauri/native/macos/sign.sh" --import-ci "$@"
