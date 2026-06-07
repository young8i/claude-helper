#!/bin/bash
# Build Claude ZH Helper for macOS
set -euo pipefail

echo "🍎 Building Claude 中文助手 for macOS..."
echo ""

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
HELPER_DIR="$(dirname "$SCRIPT_DIR")"
cd "$HELPER_DIR"

# Check prerequisites
check_cmd() {
  if ! command -v "$1" &> /dev/null; then
    echo "❌ $1 is required but not installed."
    echo "   Install: $2"
    exit 1
  fi
}

check_cmd node "brew install node"
check_cmd npm "comes with Node.js"
check_cmd cargo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"

# Copy resources
echo "📦 Copying resources..."
bash scripts/copy-resources.sh

# Install npm dependencies
echo ""
echo "📦 Installing npm dependencies..."
npm install

# Build for macOS
echo ""
echo "🔨 Building for macOS..."
npm run tauri:build -- --target universal-apple-darwin

echo ""
echo "✅ Build complete!"
echo "📂 Output: src-tauri/target/release/bundle/"
ls -la src-tauri/target/release/bundle/dmg/ 2>/dev/null || echo "Check target/release/bundle/ for output"
