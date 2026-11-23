#!/usr/bin/env bash
set -euo pipefail

export PATH="$HOME/.cargo/bin:$PATH"

echo "==> Running Web SDK tests"
node --test packages/web-sdk/test/sdk.test.ts

if command -v swiftc >/dev/null 2>&1; then
  echo "==> Typechecking iOS Swift sources"
  swiftc -typecheck -module-cache-path /tmp/swift-module-cache \
    apps/ios/Sources/DeeplinkRouter.swift \
    apps/ios/Sources/TcpPrinterTransport.swift \
    apps/ios/Sources/PrinterStore.swift
  swiftc -parse apps/ios/Sources/PrinterSettingsView.swift

  echo "==> Typechecking macOS Menu Bar app"
  swiftc -typecheck -module-cache-path /tmp/swift-module-cache \
    apps/desktop/macos/Sources/DaemonClient.swift \
    apps/desktop/macos/Sources/AppDelegate.swift \
    apps/desktop/macos/Sources/main.swift
fi

if command -v cargo >/dev/null 2>&1; then
  echo "==> Running Rust workspace tests"
  cargo test --workspace
else
  echo "==> Skipping Rust tests (cargo not installed)"
fi
