# Getting Started

**sbox** is the companion desktop toolbox for [sbot](https://github.com/while-coder/sbot). It's a small [Tauri](https://tauri.app/) + Vue 3 app that bundles the one-off chores you hit while setting sbot up, so you don't have to script them by hand:

- [**XiaoAI Login**](./xiaoai-login) — sign in to your Mi account and export the device credentials `channel.xiaoai` needs.
- [**Keystore Generator**](./keystore-gen) — create an Android signing keystore (PKCS12) and the GitHub Actions secrets that go with it.
- [**Codec**](./codec) — Base64 / URL / Hex / HTML / Unicode / JSON conversion, MD5 & SHA hashes, UUID, timestamps and byte sizes.

Each tool produces a value (often a JSON blob) that you copy and paste into sbot admin's **"Paste credential JSON"** entry, or into your repository secrets.

## Install

Grab the latest installer for your platform from the [Releases page](https://github.com/while-coder/sbox/releases), then run it.

- **Windows** — `.msi` / `.exe`
- **macOS** — `.dmg`
- **Linux** — `.AppImage` / `.deb`

The app ships with an auto-updater, so it will prompt you when a new version is available.

## Run from source

```bash
pnpm install
pnpm dev            # start tauri dev
```

Build a desktop bundle:

```bash
pnpm windows:build  # or macos:build / linux:build
```

> Building requires the [Tauri prerequisites](https://tauri.app/start/prerequisites/) (Rust toolchain + your platform's webview deps).

## Workflow

1. Launch sbox and pick a tool from the home grid.
2. Follow the on-screen steps.
3. Copy the output and paste it where it's needed — sbot admin, a `.env`, or a GitHub secret.

Head to [XiaoAI Login](./xiaoai-login) to see a full example.
