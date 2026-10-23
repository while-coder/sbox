#!/usr/bin/env node
const fs = require("node:fs");
const path = require("node:path");
const { syncNativeIcons } = require("../packages/app/src-tauri/native/sync-icons.cjs");

const root = path.resolve(__dirname, "..");
for (const platform of ["windows", "linux"]) syncNativeIcons(platform);
if (process.platform === "darwin") syncNativeIcons("macos");

const favicon = path.join(root, "packages/app/public/sbox-icon.png");
fs.mkdirSync(path.dirname(favicon), { recursive: true });
fs.copyFileSync(path.join(root, "packages/app/src-tauri/native/linux/icons/128x128.png"), favicon);
console.log("已从 packages/icon/icon.png 刷新桌面和 Web 图标（macOS 图标需在 macOS 上生成）");
