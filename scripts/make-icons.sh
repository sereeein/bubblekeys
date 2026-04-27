#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

ICONS=src-tauri/icons
ICONSET=$ICONS/icon.iconset

if [[ ! -f "$ICONS/icon-1024.png" || ! -f "$ICONS/icon-1024-simplified.png" ]]; then
  echo "ERROR: run 'tsx scripts/render-icons.ts' first to generate the master PNGs" >&2
  exit 1
fi

rm -rf "$ICONSET"
mkdir -p "$ICONSET"

# Small sizes use the simplified design (per spec §6.6).
sips -z 16 16   "$ICONS/icon-1024-simplified.png" --out "$ICONSET/icon_16x16.png"      >/dev/null
sips -z 32 32   "$ICONS/icon-1024-simplified.png" --out "$ICONSET/icon_16x16@2x.png"   >/dev/null
sips -z 32 32   "$ICONS/icon-1024-simplified.png" --out "$ICONSET/icon_32x32.png"      >/dev/null
sips -z 64 64   "$ICONS/icon-1024.png"            --out "$ICONSET/icon_32x32@2x.png"   >/dev/null
sips -z 128 128 "$ICONS/icon-1024.png"            --out "$ICONSET/icon_128x128.png"    >/dev/null
sips -z 256 256 "$ICONS/icon-1024.png"            --out "$ICONSET/icon_128x128@2x.png" >/dev/null
sips -z 256 256 "$ICONS/icon-1024.png"            --out "$ICONSET/icon_256x256.png"    >/dev/null
sips -z 512 512 "$ICONS/icon-1024.png"            --out "$ICONSET/icon_256x256@2x.png" >/dev/null
sips -z 512 512 "$ICONS/icon-1024.png"            --out "$ICONSET/icon_512x512.png"    >/dev/null
cp "$ICONS/icon-1024.png" "$ICONSET/icon_512x512@2x.png"

iconutil -c icns -o "$ICONS/icon.icns" "$ICONSET"

# Update the legacy filenames Tauri 2 auto-discovers.
cp "$ICONS/icon-1024.png" "$ICONS/icon.png"
sips -z 128 128 "$ICONS/icon-1024.png" --out "$ICONS/128x128.png"    >/dev/null
sips -z 256 256 "$ICONS/icon-1024.png" --out "$ICONS/128x128@2x.png" >/dev/null
sips -z 32 32   "$ICONS/icon-1024-simplified.png" --out "$ICONS/32x32.png" >/dev/null

# Clean up intermediate iconset (icns has all sizes embedded).
rm -rf "$ICONSET"

echo "icons generated:"
ls -l "$ICONS"
