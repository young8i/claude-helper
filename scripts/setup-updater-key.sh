#!/bin/bash
# Generate the signing keypair used by the Tauri updater.
# Run this once per app identity and keep the private key secret.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
HELPER_DIR="$(dirname "$SCRIPT_DIR")"
KEY_PATH="$HOME/.tauri/claude-zh-helper.key"

echo "🔑 Generating Tauri updater signing keys..."
echo ""

cd "$HELPER_DIR"
mkdir -p "$(dirname "$KEY_PATH")"
npx tauri signer generate -w "$KEY_PATH" 2>&1 || {
  echo ""
  echo "❌ Key generation failed. Make sure dependencies are installed:"
  echo "   npm install"
  exit 1
}

PUBKEY=$(cat "${KEY_PATH}.pub" 2>/dev/null || echo "")
if [ -z "$PUBKEY" ]; then
  echo "❌ Could not read public key: ${KEY_PATH}.pub"
  exit 1
fi

echo ""
echo "✅ Keypair generated:"
echo "   Private key: $KEY_PATH"
echo "   Public key:  ${KEY_PATH}.pub"
echo ""
echo "📋 Public key for src-tauri/tauri.conf.json → plugins.updater.pubkey:"
echo "   $PUBKEY"
echo ""
echo "The release script reads $KEY_PATH automatically. If you use a different key,"
echo "set TAURI_SIGNING_PRIVATE_KEY before running ./scripts/release.sh."
