#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ANDROID_DIR="$ROOT_DIR/apps/android"

if [[ -z "${JAVA_HOME:-}" ]] && [[ -d "/Applications/Android Studio.app/Contents/jbr/Contents/Home" ]]; then
  export JAVA_HOME="/Applications/Android Studio.app/Contents/jbr/Contents/Home"
  export PATH="$JAVA_HOME/bin:$PATH"
fi

if [[ -z "${ANDROID_HOME:-}" ]] && [[ -d "$HOME/Library/Android/sdk" ]]; then
  export ANDROID_HOME="$HOME/Library/Android/sdk"
fi

cd "$ANDROID_DIR"

if [[ -x "./gradlew" ]]; then
  ./gradlew assembleDebug
elif command -v gradle >/dev/null 2>&1; then
  gradle assembleDebug
else
  echo "Error: Neither ./gradlew nor 'gradle' was found in PATH."
  echo "To fix on macOS:"
  echo "  1. Install Gradle: brew install gradle"
  echo "  2. Or open apps/android in Android Studio (which generates gradlew & builds the APK)."
  exit 1
fi
