# Build, Run & Development Guide

This guide explains how to develop, build, test, and run every component of the **BNOS & BDGO OS Native Print Service**.

---

## 1. Prerequisites

| Component | Required Tools |
| :--- | :--- |
| **Web SDK & QA Scripts** | Node.js `>= 22` (tested on `v24.22.1`) & `pnpm` |
| **Desktop Daemon & Rust Core** | Rust toolchain (`rustup`, `cargo`, `rustc`) |
| **iOS App** | macOS + Xcode 15+ (SwiftUI) |
| **Android App** | Android Studio / JDK 17+ & Android SDK (API 34+) |

---

## 2. Quick Dev: Mock Printer + Verification

Test ESC/POS printing without physical Xprinter hardware using the built-in TCP `9100` mock printer:

```bash
# Terminal 1: Start mock thermal printer on 127.0.0.1:9100
pnpm dev:mock-printer
# (or: MOCK_PRINTER_PORT=9100 node scripts/mock-printer.mjs)
```

Run all available automated checks (Web SDK tests, Swift typecheck/parse, Rust tests if `cargo` is installed):

```bash
pnpm check
# (or: ./scripts/check.sh)
```

---

## 3. Desktop Daemon (macOS, Windows, Linux)

### Run in Development
```bash
# Default: listens on http://127.0.0.1:3100, default printer 192.168.2.100:9100
pnpm dev:daemon
# (or: cargo run -p print-daemon)
```

To test against the local mock printer:
```bash
PRINTER_IP=127.0.0.1 PRINTER_PORT=9100 LISTEN_PORT=3100 cargo run -p print-daemon
```

Once running, open the built-in **Settings & Multi-Printer UI** in your browser:
- **Settings UI**: `http://127.0.0.1:3100/` (Add multiple IP printers, set default, run Test Print)
- **Health Check**: `curl http://127.0.0.1:3100/health`
- **Test Print via API**:
  ```bash
  curl -X POST http://127.0.0.1:3100/print \
    -H "Content-Type: application/json" \
    -d '{"type":"test","ip":"127.0.0.1","port":9100}'
  ```

### Build Release Binary
```bash
# Native build (produces target/release/print-daemon on macOS/Linux, target/release/print-daemon.exe on Windows)
pnpm build:daemon

# Cross-compile Windows .exe from macOS/Linux (requires mingw-w64 + rustup target x86_64-pc-windows-gnu)
rustup target add x86_64-pc-windows-gnu
pnpm build:daemon:win
# Output binary: target/x86_64-pc-windows-gnu/release/print-daemon.exe
```

### Install as Background Service
- **Windows (`.exe` + System Tray UI + Auto-Start)**:
  ```powershell
  # 1. Build daemon .exe
  cargo build --release -p print-daemon
  powershell -ExecutionPolicy Bypass -File apps\desktop\windows\install-startup.ps1

  # 2. Build & run native Windows System Tray & Settings UI (.NET 8)
  dotnet publish apps\desktop\windows\BNOSPrintTray.csproj -c Release -r win-x64 --self-contained -p:PublishSingleFile=true
  ```
- **macOS (`LaunchAgent` + Menu Bar App)**:
  ```bash
  sudo cp target/release/print-daemon /usr/local/bin/bnos-print-daemon
  cp apps/desktop/macos/space.bnos.print-daemon.plist ~/Library/LaunchAgents/
  launchctl load -w ~/Library/LaunchAgents/space.bnos.print-daemon.plist

  # Build & run native macOS Menu Bar & Settings UI app
  swift build --package-path apps/desktop/macos -c release
  apps/desktop/macos/.build/release/BNOSPrintTray
  ```
- **Linux (`systemd --user` + USB `udev`)**:
  ```bash
  sudo cp target/release/print-daemon /usr/local/bin/bnos-print-daemon
  sudo cp apps/desktop/linux/99-xprinter.rules /etc/udev/rules.d/
  sudo udevadm control --reload-rules
  mkdir -p ~/.config/systemd/user
  cp apps/desktop/linux/bnos-print.service ~/.config/systemd/user/
  systemctl --user enable --now bnos-print.service
  ```

---

## 4. Web SDK (`packages/web-sdk`)

### Run Unit Tests
```bash
pnpm test
# (or: node --test packages/web-sdk/test/sdk.test.ts)
```

### Usage in `app.bnos.space` / BDGO OS
```ts
import { BnosPrintClient, renderReceiptToEscPos } from '@bnos/print-sdk';

const printer = new BnosPrintClient();

// 1. Check desktop daemon status
const health = await printer.checkHealth();
if (health.needsInstall) {
  console.warn('Prompt user to install BNOS Print Daemon');
}

// 2. Print a structured receipt (auto-routes Desktop HTTP vs Mobile Deeplink)
const data = renderReceiptToEscPos({
  title: 'BNOS POS',
  lines: [{ text: 'Total: 150.000 VND', bold: true }],
  cut: 'full',
});

await printer.print({
  type: 'raw',
  ip: '192.168.2.100',
  port: 9100,
  data,
  callback: 'https://app.bnos.space/pos',
});
```

---

## 5. Mobile Apps (iOS & Android)

### iOS (`apps/ios`)
1. Open or create an Xcode iOS App target including:
   - `apps/ios/Info.plist` (`bnos-print` URL scheme + Local Network / Bluetooth descriptions)
   - `apps/ios/Sources/*.swift` (`PrinterStore.swift`, `PrinterSettingsView.swift`, `DeeplinkRouter.swift`, `TcpPrinterTransport.swift`)
2. Test deeplink in iOS Simulator:
   ```bash
   xcrun simctl openurl booted "bnos-print://print?v=1&type=test&ip=127.0.0.1&port=9100"
   ```

### Android (`apps/android`)
1. Open or create an Android Gradle module including:
   - `apps/android/AndroidManifest.xml`
   - `apps/android/src/main/java/space/bnos/print/*.kt` (`PrinterSettingsActivity.kt`, `DeeplinkActivity.kt`, `PrinterStore.kt`, `TcpPrinterTransport.kt`)
2. Test deeplink via `adb`:
   ```bash
   adb shell am start -a android.intent.action.VIEW \
     -d "bnos-print://print?v=1&type=test&ip=192.168.2.100&port=9100"
   ```
