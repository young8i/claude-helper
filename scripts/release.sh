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

# 2. 构建
echo "🔨 开始构建…"
npm install
npm run tauri:build

# 3. 找到产物
BUNDLE_DIR="$HELPER_DIR/src-tauri/target/release/bundle"

MAC_APP_TAR=$(ls "$BUNDLE_DIR"/macos/*.app.tar.gz 2>/dev/null | head -1 || echo "")
WIN_MSI=$(ls "$BUNDLE_DIR"/msi/*.msi 2>/dev/null | head -1 || echo "")

if [ -z "$MAC_APP_TAR" ] && [ -z "$WIN_MSI" ]; then
  echo "❌ 未找到任何构建产物"
  exit 1
fi

# 4. 发布到公开 Releases 仓库
echo "📦 发布到公开仓库 ${PUBLIC_RELEASES_REPO}…"

RELEASE_FILES=""
[ -n "$MAC_APP_TAR" ] && RELEASE_FILES="$RELEASE_FILES $MAC_APP_TAR"
[ -n "$WIN_MSI" ] && RELEASE_FILES="$RELEASE_FILES $WIN_MSI"

gh release create "v${VERSION}" \
  --repo "$PUBLIC_RELEASES_REPO" \
  --title "Claude 中文助手 v${VERSION}" \
  --notes "Claude 中文助手 v${VERSION}。汉化包已同步到最新。" \
  $RELEASE_FILES

echo ""
echo "✅ 发布完成！"
echo "  下载地址: https://github.com/${PUBLIC_RELEASES_REPO}/releases/tag/v${VERSION}"
echo ""
echo "⚠️  注意: 首次使用需要先创建公开仓库 ${PUBLIC_RELEASES_REPO}"
echo "   并且确保 gh 已登录: gh auth login"
