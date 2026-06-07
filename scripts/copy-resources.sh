#!/bin/bash
# Copy Chinese localization resources from parent project
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
HELPER_DIR="$(dirname "$SCRIPT_DIR")"
PARENT_DIR="$(dirname "$HELPER_DIR")"
RESOURCES_SRC="$PARENT_DIR/resources"
SCRIPTS_SRC="$PARENT_DIR/scripts"
RESOURCES_DST="$HELPER_DIR/resources"
SCRIPTS_DST="$HELPER_DIR/scripts"

echo "📦 Copying Chinese localization resources..."

# Copy resource files
if [ -d "$RESOURCES_SRC" ]; then
  mkdir -p "$RESOURCES_DST"
  cp -v "$RESOURCES_SRC"/*.json "$RESOURCES_DST/" 2>/dev/null || true
  cp -v "$RESOURCES_SRC"/*.strings "$RESOURCES_DST/" 2>/dev/null || true
  echo "✅ Resources copied to $RESOURCES_DST"
else
  echo "⚠️  Resources not found at $RESOURCES_SRC"
fi

# Copy scripts
if [ -d "$SCRIPTS_SRC" ]; then
  mkdir -p "$SCRIPTS_DST"
  cp -v "$SCRIPTS_SRC"/*.py "$SCRIPTS_DST/" 2>/dev/null || true
  cp -v "$SCRIPTS_SRC"/*.command "$SCRIPTS_DST/" 2>/dev/null || true
  cp -v "$SCRIPTS_SRC"/*.ps1 "$SCRIPTS_DST/" 2>/dev/null || true
  cp -v "$SCRIPTS_SRC"/*.bat "$SCRIPTS_DST/" 2>/dev/null || true
  echo "✅ Scripts copied to $SCRIPTS_DST"
else
  echo "⚠️  Scripts not found at $SCRIPTS_SRC"
fi

# Copy install scripts from parent root
if [ -f "$PARENT_DIR/install-mac.command" ]; then
  cp -v "$PARENT_DIR/install-mac.command" "$HELPER_DIR/resources/"
  echo "✅ install-mac.command copied"
fi
if [ -f "$PARENT_DIR/install-windows.bat" ]; then
  cp -v "$PARENT_DIR/install-windows.bat" "$HELPER_DIR/resources/"
  echo "✅ install-windows.bat copied"
fi

echo ""
echo "🎉 Resources ready!"
