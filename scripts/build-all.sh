#!/usr/bin/env bash
set -euo pipefail

export PATH="$HOME/.cargo/bin:$PATH"
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist"
mkdir -p "$DIST_DIR"

echo "=== 1. Running checks ==="
"$ROOT_DIR/scripts/check.sh"

echo "=== 2. Building Desktop (macOS) ==="
if [[ "$(uname -s)" == "Darwin" ]] && command -v cargo >/dev/null 2>&1 && command -v swift >/dev/null 2>&1; then
  "$ROOT_DIR/scripts/package-macos-dmg.sh"
else
  echo "Skipping macOS .dmg (requires macOS + cargo + swift)"
fi

echo "=== 3. Building Desktop (Windows .exe) ==="
if command -v cargo >/dev/null 2>&1 && rustup target list --installed 2>/dev/null | grep -q "x86_64-pc-windows-gnu"; then
  cargo build --release -p print-daemon --target x86_64-pc-windows-gnu --manifest-path "$ROOT_DIR/Cargo.toml"
  cp "$ROOT_DIR/target/x86_64-pc-windows-gnu/release/print-daemon.exe" "$DIST_DIR/bnos-print-daemon-win64.exe"
else
  echo "Skipping Windows daemon .exe (run: rustup target add x86_64-pc-windows-gnu)"
fi

if command -v dotnet >/dev/null 2>&1; then
  dotnet publish "$ROOT_DIR/apps/desktop/windows/BNOSPrintTray.csproj" -c Release -r win-x64 --self-contained -p:PublishSingleFile=true -o "$DIST_DIR/windows-tray"
else
  echo "Skipping Windows Tray UI .exe (dotnet SDK not installed)"
fi

echo "=== 4. Building Desktop (Linux) ==="
if [[ "$(uname -s)" == "Linux" ]] && command -v cargo >/dev/null 2>&1; then
  cargo build --release -p print-daemon --manifest-path "$ROOT_DIR/Cargo.toml"
  cp "$ROOT_DIR/target/release/print-daemon" "$DIST_DIR/bnos-print-daemon-linux"
else
  echo "Skipping Linux native binary (not running on Linux)"
fi

echo "=== 5. Building Mobile (Android / iOS) ==="
if [[ -x "$ROOT_DIR/apps/android/gradlew" ]]; then
  (cd "$ROOT_DIR/apps/android" && ./gradlew assembleRelease)
elif command -v gradle >/dev/null 2>&1 && [[ -n "${ANDROID_HOME:-}" ]]; then
  (cd "$ROOT_DIR/apps/android" && gradle assembleDebug)
else
  echo "Skipping Android .apk (install Android SDK + gradle or open apps/android in Android Studio)"
fi

if [[ "$(uname -s)" == "Darwin" ]] && ls "$ROOT_DIR/apps/ios/"*.xcodeproj >/dev/null 2>&1; then
  echo "Xcode project detected in apps/ios — run xcodebuild archive for signed .ipa"
else
  echo "Skipping iOS .ipa (Xcode project / signing profile not configured in apps/ios)"
fi

echo "=== Build Summary ($DIST_DIR) ==="
ls -lh "$DIST_DIR"
