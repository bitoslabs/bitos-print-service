#!/usr/bin/env bash
set -euo pipefail

export PATH="$HOME/.cargo/bin:$PATH"
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist"
STAGING_DIR="$DIST_DIR/dmg-staging"
APP_BUNDLE="$STAGING_DIR/BNOSPrint.app"
DMG_PATH="$DIST_DIR/BNOSPrint.dmg"

echo "==> Building print-daemon (release)"
cargo build --release -p print-daemon --manifest-path "$ROOT_DIR/Cargo.toml"

rm -rf "$STAGING_DIR" "$DMG_PATH"
mkdir -p "$APP_BUNDLE/Contents/MacOS" "$APP_BUNDLE/Contents/Resources"

echo "==> Building BNOSPrintTray macOS Menu Bar app (release)"
swiftc -O -module-cache-path /tmp/swift-module-cache \
  "$ROOT_DIR/apps/desktop/macos/Sources/DaemonClient.swift" \
  "$ROOT_DIR/apps/desktop/macos/Sources/AppDelegate.swift" \
  "$ROOT_DIR/apps/desktop/macos/Sources/main.swift" \
  -o "$APP_BUNDLE/Contents/MacOS/BNOSPrintTray"

cp "$ROOT_DIR/target/release/print-daemon" "$APP_BUNDLE/Contents/MacOS/bnos-print-daemon"
if [[ -f "$ROOT_DIR/apps/desktop/macos/Resources/AppIcon.icns" ]]; then
  cp "$ROOT_DIR/apps/desktop/macos/Resources/AppIcon.icns" "$APP_BUNDLE/Contents/Resources/AppIcon.icns"
fi

cat > "$APP_BUNDLE/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key>
  <string>BNOS Print</string>
  <key>CFBundleIdentifier</key>
  <string>space.bnos.print.desktop</string>
  <key>CFBundleExecutable</key>
  <string>BNOSPrintTray</string>
  <key>CFBundleIconFile</key>
  <string>AppIcon</string>
  <key>CFBundleVersion</key>
  <string>0.1.0</string>
  <key>CFBundleShortVersionString</key>
  <string>0.1.0</string>
  <key>LSUIElement</key>
  <true/>
</dict>
</plist>
PLIST

# Add Applications symlink for drag-and-drop installation
ln -s /Applications "$STAGING_DIR/Applications"

echo "==> Ad-hoc signing $APP_BUNDLE"
codesign --force --deep --sign - "$APP_BUNDLE"

echo "==> Creating $DMG_PATH"
hdiutil create -volname "BNOS Print" -srcfolder "$STAGING_DIR" -ov -format UDZO "$DMG_PATH"
rm -rf "$STAGING_DIR"
echo "Done: $DMG_PATH (drag BNOSPrint.app to Applications)"
