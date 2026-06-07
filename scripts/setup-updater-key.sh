#!/bin/bash
# Generate signing keypair for Tauri updater
# Run this ONCE per project. Keep the private key SECRET.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
HELPER_DIR="$(dirname "$SCRIPT_DIR")"

echo "🔑 Generating Tauri updater signing keys..."
echo ""

# Generate keypair
cd "$HELPER_DIR/src-tauri"
cargo tauri signer generate -w ~/.tauri/claude-zh-helper.key 2>&1 || {
  echo ""
  echo "❌ Key generation failed. Make sure @tauri-apps/cli is installed:"
  echo "   npm install -g @tauri-apps/cli"
  exit 1
}

PUBKEY=$(cat ~/.tauri/claude-zh-helper.key.pub 2>/dev/null || echo "")

if [ -z "$PUBKEY" ]; then
  echo "❌ Could not read public key"
  exit 1
fi

echo ""
echo "✅ Keypair generated:"
echo "   Private key: ~/.tauri/claude-zh-helper.key  ← KEEP SECRET"
echo "   Public key:  ~/.tauri/claude-zh-helper.key.pub"
echo ""
echo "📋 Public key (add this to tauri.conf.json → plugins.updater.pubkey):"
echo "   $PUBKEY"
echo ""
echo "⚙️  Replace 'PLACEHOLDER_RUN_tauri_signer_generate' in tauri.conf.json with this public key."
