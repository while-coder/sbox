#!/usr/bin/env node
const { execSync } = require('child_process');
const path = require('path');
const fs = require('fs');

const PKG_JSON = 'package.json';
const TAG_PREFIX = 'v';
const CHANGELOG = 'CHANGELOG.md';
const WORKFLOW_NAME = 'Release';

const SEMVER_RE = /^\d+\.\d+\.\d+$/;
const BUMP_TYPES = ['patch', 'minor', 'major'];

function usage(msg) {
  if (msg) console.error(`error: ${msg}\n`);
  console.error('Usage: node scripts/release.js [<version>|patch|minor|major]');
  console.error('  no arg           : use current version, tag & push');
  console.error('  patch|minor|major: bump version, commit, tag & push');
  console.error('  X.Y.Z            : set version, commit, tag & push');
  process.exit(1);
}

function run(cmd, opts = {}) {
  console.log(`> ${cmd}`);
  return execSync(cmd, { stdio: 'inherit', ...opts });
}

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, 'utf8'));
}

function writeJson(file, data) {
  fs.writeFileSync(file, JSON.stringify(data, null, 2) + '\n');
}

function bumpVersion(version, type) {
  const [maj, min, pat] = version.split('.').map(Number);
  if (type === 'major') return `${maj + 1}.0.0`;
  if (type === 'minor') return `${maj}.${min + 1}.0`;
  return `${maj}.${min}.${pat + 1}`;
}

function isWorkingTreeClean() {
  return execSync('git status --porcelain', { encoding: 'utf8' }).trim().length === 0;
}

function tagExists(tag) {
  try {
    execSync(`git rev-parse -q --verify "refs/tags/${tag}"`, { stdio: 'ignore' });
    return true;
  } catch {
    return false;
  }
}

// the Release workflow pulls release notes from the "## <version>" section of CHANGELOG.md
function changelogHasVersion(file, version) {
  if (!fs.existsSync(file)) return false;
  const lines = fs.readFileSync(file, 'utf8').split('\n');
  return lines.some((l) => l.trim().split(/\s+/).slice(0, 2).join(' ') === `## ${version}`);
}

function main() {
  const arg = process.argv[2];
  const root = path.resolve(__dirname, '..');
  // 版本以桌面应用包为准（tauri.conf.json 的 version 指向 packages/app/package.json）
  const pkgRel = `packages/app/${PKG_JSON}`;
  const pkgPath = path.join(root, 'packages', 'app', PKG_JSON);
  const pkg = readJson(pkgPath);

  const currentVersion = pkg.version;
  if (!currentVersion || !SEMVER_RE.test(currentVersion)) {
    console.error(`error: cannot resolve a valid version from ${PKG_JSON}`);
    process.exit(1);
  }

  let nextVersion = currentVersion;
  let versionChanged = false;

  if (arg) {
    if (BUMP_TYPES.includes(arg)) {
      nextVersion = bumpVersion(currentVersion, arg);
      versionChanged = true;
    } else if (SEMVER_RE.test(arg)) {
      nextVersion = arg;
      versionChanged = nextVersion !== currentVersion;
    } else {
      usage(`invalid version/bump "${arg}"`);
    }
  }

  const tag = `${TAG_PREFIX}${nextVersion}`;

  console.log(`current : ${currentVersion}`);
  console.log(`next    : ${nextVersion}`);
  console.log(`tag     : ${tag}`);
  console.log('');

  if (tagExists(tag)) {
    console.error(`error: tag "${tag}" already exists`);
    process.exit(1);
  }

  if (!changelogHasVersion(path.join(root, CHANGELOG), nextVersion)) {
    console.warn(`warning: ${CHANGELOG} has no "## ${nextVersion}" section — release notes will fall back to "Release ${nextVersion}"`);
  }

  if (versionChanged) {
    if (!isWorkingTreeClean()) {
      console.error('error: working tree not clean — commit or stash before releasing');
      process.exit(1);
    }

    pkg.version = nextVersion;
    writeJson(pkgPath, pkg);

    run(`git add "${pkgRel}"`);
    run(`git commit -m "chore: release v${nextVersion}"`);
  }

  run(`git tag -a "${tag}" -m "sbox v${nextVersion}"`);
  run('git push');
  run(`git push origin "${tag}"`);

  console.log('');
  console.log(`✓ pushed tag ${tag} — workflow "${WORKFLOW_NAME}" triggered`);
}

main();
