#!/bin/bash
# 本地手动发布脚本 — 从私有仓库构建，推到公开 Releases 仓库
# 需要: GitHub CLI (brew install gh)
set -euo pipefail

VERSION="${1:-}"
PUBLIC_RELEASES_REPO="young8i/claude-releases"

if [ -z "$VERSION" ]; then
  echo "Usage: ./scripts/release.sh <version>"
  echo "Example: ./scripts/release.sh 1.0.1"
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
HELPER_DIR="$(dirname "$SCRIPT_DIR")"

echo "🚀 手动发布 Claude 中文助手 v${VERSION}"
echo ""

# 1. 更新版本号
echo "📝 更新版本号…"
cd "$HELPER_DIR"
sed -i '' "s/\"version\": \"[^\"]*\"/\"version\": \"${VERSION}\"/" package.json
sed -i '' "s/\"version\": \"[^\"]*\"/\"version\": \"${VERSION}\"/" src-tauri/tauri.conf.json
sed -i '' "s/^version = \"[^\"]*\"/version = \"${VERSION}\"/" src-tauri/Cargo.toml

if [ -z "${TAURI_SIGNING_PRIVATE_KEY:-}" ] && [ -z "${TAURI_SIGNING_PRIVATE_KEY_PATH:-}" ] && [ -f "$HOME/.tauri/claude-zh-helper.key" ]; then
  export TAURI_SIGNING_PRIVATE_KEY_PATH="$HOME/.tauri/claude-zh-helper.key"
fi

if [ -z "${TAURI_SIGNING_PRIVATE_KEY:-}" ] && [ -z "${TAURI_SIGNING_PRIVATE_KEY_PATH:-}" ]; then
  echo "❌ 缺少 Tauri updater 签名私钥。"
  echo "   请先运行: ./scripts/setup-updater-key.sh"
  echo "   或设置环境变量: TAURI_SIGNING_PRIVATE_KEY / TAURI_SIGNING_PRIVATE_KEY_PATH"
  exit 1
fi

# 2. 构建
echo "🔨 开始构建…"
npm install
npm run tauri:build

# 3. 找到产物
BUNDLE_DIR="$HELPER_DIR/src-tauri/target/release/bundle"

echo "🧾 生成 updater 清单 latest.json…"
node - "$BUNDLE_DIR" "$VERSION" "$PUBLIC_RELEASES_REPO" <<'NODE'
const fs = require("fs");
const path = require("path");

const bundleDir = process.argv[2];
const version = process.argv[3];
const repo = process.argv[4];

function walk(dir) {
  const out = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) out.push(...walk(full));
    else out.push(full);
  }
  return out;
}

function assetUrl(file) {
  const assetName = encodeURIComponent(path.basename(file));
  return `https://github.com/${repo}/releases/download/v${version}/${assetName}`;
}

function updaterPriority(file) {
  const name = path.basename(file).toLowerCase();
  if (name.endsWith(".app.tar.gz")) return 10;
  if (name.endsWith(".msi.zip") || name.endsWith(".nsis.zip")) return 10;
  if (name.endsWith(".msi") || name.endsWith(".exe")) return 20;
  return 100;
}

function setPlatform(platforms, key, artifact, signature) {
  const current = platforms[key];
  if (current && updaterPriority(current.artifact) <= updaterPriority(artifact)) {
    return;
  }
  platforms[key] = {
    signature,
    url: assetUrl(artifact),
    artifact,
  };
}

const files = walk(bundleDir);
const sigFiles = files.filter((file) => file.endsWith(".sig")).sort();
const platforms = {};

for (const sigFile of sigFiles) {
  const artifact = sigFile.slice(0, -4);
  if (!fs.existsSync(artifact)) continue;

  const name = path.basename(artifact).toLowerCase();
  const signature = fs.readFileSync(sigFile, "utf8").trim();
  if (!signature) continue;

  if (name.endsWith(".app.tar.gz")) {
    setPlatform(platforms, "darwin-x86_64", artifact, signature);
    setPlatform(platforms, "darwin-aarch64", artifact, signature);
  } else if (
    name.endsWith(".msi.zip") ||
    name.endsWith(".nsis.zip") ||
    name.endsWith(".msi") ||
    name.endsWith(".exe")
  ) {
    setPlatform(platforms, "windows-x86_64", artifact, signature);
  }
}

for (const value of Object.values(platforms)) {
  delete value.artifact;
}

if (Object.keys(platforms).length === 0) {
  console.error("No updater signature/artifact pairs found.");
  process.exit(1);
}

const latest = {
  version,
  notes: `Claude 中文助手 v${version}`,
  pub_date: new Date().toISOString(),
  platforms,
};

fs.writeFileSync(
  path.join(bundleDir, "latest.json"),
  JSON.stringify(latest, null, 2) + "\n",
);
NODE

RELEASE_FILES=()
while IFS= read -r file; do
  RELEASE_FILES+=("$file")
done < <(
  find "$BUNDLE_DIR" -type f \( \
    -name "*.dmg" -o \
    -name "*.msi" -o \
    -name "*.exe" -o \
    -name "*.app.tar.gz" -o \
    -name "*.app.tar.gz.sig" -o \
    -name "*.msi.zip" -o \
    -name "*.msi.zip.sig" -o \
    -name "*.nsis.zip" -o \
    -name "*.nsis.zip.sig" -o \
    -name "latest.json" \
  \) | sort
)

if [ "${#RELEASE_FILES[@]}" -eq 0 ]; then
  echo "❌ 未找到任何构建产物"
  exit 1
fi

if ! printf '%s\n' "${RELEASE_FILES[@]}" | grep -q '/latest\.json$'; then
  echo "❌ 未找到 updater 清单 latest.json，自动更新将不可用。"
  echo "   请确认 src-tauri/tauri.conf.json 中 bundle.createUpdaterArtifacts=true 且签名密钥有效。"
  exit 1
fi

if ! printf '%s\n' "${RELEASE_FILES[@]}" | grep -q '\.sig$'; then
  echo "❌ 未找到 updater 签名文件 (*.sig)，自动更新将不可用。"
  exit 1
fi

# 4. 发布到公开 Releases 仓库
echo "📦 发布到公开仓库 ${PUBLIC_RELEASES_REPO}…"
printf '  %s\n' "${RELEASE_FILES[@]}"

gh release create "v${VERSION}" \
  --repo "$PUBLIC_RELEASES_REPO" \
  --title "Claude 中文助手 v${VERSION}" \
  --notes "Claude 中文助手 v${VERSION}。汉化包已同步到最新。" \
  "${RELEASE_FILES[@]}"

echo ""
echo "✅ 发布完成！"
echo "  下载地址: https://github.com/${PUBLIC_RELEASES_REPO}/releases/tag/v${VERSION}"
echo ""
echo "⚠️  注意: 首次使用需要先创建公开仓库 ${PUBLIC_RELEASES_REPO}"
echo "   并且确保 gh 已登录: gh auth login"
