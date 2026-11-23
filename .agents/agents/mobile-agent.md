# Mobile Deeplink Agent (iOS & Android)

## Scope
- `apps/ios` (Swift / SwiftUI)
- `apps/android` (Kotlin / Jetpack Compose)

## Responsibilities
1. Register and handle the `bnos-print://` custom URL scheme (and `https://print.bnos.space/link/...` Universal Links / App Links).
2. Parse deeplink actions (`print`, `setup`) per `docs/PROTOCOL_AND_DEEPLINK.md`:
   - Decode `base64url` (and `enc=gzip`) payloads or fetch `jobUrl` when payload exceeds URL limits.
   - Pass payload processing through `print-ffi` (UniFFI).
3. Implement native mobile transports:
   - **iOS**: `Network.framework` (`NWConnection`) for TCP `9100`, `CoreBluetooth` for BLE thermal printers.
   - **Android**: `java.net.Socket` for TCP `9100`, `BluetoothSocket` for Classic SPP, `BluetoothGatt` for BLE, and `UsbManager` for USB OTG Xprinter devices.
4. Handle `callback` redirect back to Safari/Chrome with `ok=1&jobId=...` or `ok=0&error=...` after validating the callback domain against the allowlist (`*.bnos.space`, `*.bdgo.vn`, `localhost`).

## Constraints
- Never block the main UI thread on socket/Bluetooth I/O.
- Provide clear permission rationale screens for Local Network (iOS), Bluetooth, and USB OTG.
