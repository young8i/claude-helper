#!/bin/bash
# Generate placeholder app icons for Tauri
# For production, replace with proper icon design.
#
# macOS: Use an actual .icns file
#   Create a 1024x1024 PNG, then:
#   mkdir MyIcon.iconset
#   sips -z 16 16   icon.png --out MyIcon.iconset/icon_16x16.png
#   sips -z 32 32   icon.png --out MyIcon.iconset/icon_16x16@2x.png
#   sips -z 32 32   icon.png --out MyIcon.iconset/icon_32x32.png
#   sips -z 64 64   icon.png --out MyIcon.iconset/icon_32x32@2x.png
#   sips -z 128 128 icon.png --out MyIcon.iconset/icon_128x128.png
#   sips -z 256 256 icon.png --out MyIcon.iconset/icon_128x128@2x.png
#   sips -z 256 256 icon.png --out MyIcon.iconset/icon_256x256.png
#   sips -z 512 512 icon.png --out MyIcon.iconset/icon_256x256@2x.png
#   sips -z 512 512 icon.png --out MyIcon.iconset/icon_512x512.png
#   sips -z 1024 1024 icon.png --out MyIcon.iconset/icon_512x512@2x.png
#   iconutil -c icns MyIcon.iconset
#
# Windows: Use a 256x256 .ico file
#   Convert a 256x256 PNG using ImageMagick:
#   convert icon.png -resize 256x256 icon.ico

echo "For development, Tauri CLI can generate placeholder icons:"
echo "  npm install -g @tauri-apps/cli"
echo "  npx tauri icon --help"
echo ""
echo "Place the generated icons in src-tauri/icons/"
echo ""
echo "Icon sizes needed:"
echo "  32x32.png"
echo "  128x128.png"
echo "  128x128@2x.png (256x256)"
echo "  icon.icns (macOS)"
echo "  icon.ico (Windows)"
echo ""
echo "A simple emoji-based icon can be made by taking a screenshot of '🀄' on a gradient background."
