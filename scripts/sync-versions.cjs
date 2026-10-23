#!/usr/bin/env node
const fs = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const readJson = (file) => JSON.parse(fs.readFileSync(path.join(root, file), "utf8"));
const version = readJson("package.json").version;
if (!/^\d+\.\d+\.\d+$/.test(version ?? "")) {
  throw new Error("根 package.json 的 version 必须为 major.minor.patch 格式");
}

const updates = [];
const packages = fs.readdirSync(path.join(root, "packages"), { withFileTypes: true })
  .filter((entry) => entry.isDirectory())
  .map((entry) => `packages/${entry.name}/package.json`)
  .filter((file) => fs.existsSync(path.join(root, file)));
for (const file of packages) {
  const currentVersion = readJson(file).version;
  if (!currentVersion) {
    console.warn(`${file} 没有 version 字段，已跳过`);
    continue;
  }
  update(file, /("version"\s*:\s*")([^"]+)(")/);
}
// Only update this application's Rust package, leaving all dependencies intact.
update("packages/app/src-tauri/Cargo.toml", /(\[package\][\s\S]*?^version\s*=\s*")([^"]+)(")/m);
update("packages/app/src-tauri/Cargo.lock", /(\[\[package\]\]\r?\nname = "sbox"\r?\nversion = ")([^"]+)(")/);

for (const { file, content, previous } of updates) {
  fs.writeFileSync(path.join(root, file), content);
  console.log(`${file}: ${previous} -> ${version}`);
}
console.log(updates.length ? `已同步 ${updates.length} 个版本文件到 ${version}` : `所有版本已与根版本一致 (${version})`);

function update(file, pattern) {
  const content = fs.readFileSync(path.join(root, file), "utf8");
  const match = content.match(pattern);
  if (!match) throw new Error(`${file} 没有应用版本字段`);
  if (match[2] === version) return;
  updates.push({ file, previous: match[2], content: content.replace(pattern,
    (_match, prefix, _previous, suffix) => `${prefix}${version}${suffix}`) });
}
