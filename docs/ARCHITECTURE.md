# Architecture

## Overview

The BNOS & BDGO OS Native Print Service uses a **shared-core, native-shell** architecture. All printer protocol logic, job queueing, retry handling, and raw byte generation live in Rust crates, while platform-specific UI, background lifecycle, and OS permissions live in native shells.

```text
+-------------------------------------------------------------------+
|                   BNOS / BDGO OS Web Client                       |
|                  (packages/web-sdk - TypeScript)                  |
+----------------------+-------------------------------+------------+
                       |                               |
         Desktop       |                               | Mobile (iOS/Android)
   HTTP/WS (127.0.0.1) |                               | Deeplink (bnos-print://)
                       v                               v
+----------------------------------+    +---------------------------+
|      apps/desktop + daemon       |    |   apps/ios & apps/android |
|  (macOS / Windows / Linux)       |    |   (Swift / Kotlin Shell)  |
|  - HTTP/WS Server (:3100)        |    |   - Deeplink Router       |
|  - System Tray & Status UI       |    |   - BLE / Classic BT / USB|
|  - OS Service (launchd/systemd)  |    |   - Callback to Browser   |
+----------------+-----------------+    +-------------+-------------+
                 |                                    |
                 |         UniFFI / C-ABI             |
                 +-----------------+------------------+
                                   |
                                   v
+-------------------------------------------------------------------+
|                     Shared Rust Core (crates/)                    |
|  - print-core: ESC/POS, TSPL, receipt layout, codepage, queue     |
|  - print-transport: TCP 9100, USB, Serial, BT, Spooler            |
+-------------------------------------------------------------------+
```

## Workspace Crates (`crates/`)

### 1. `crates/print-core`
- **Responsibility**: Pure, platform-agnostic printer logic.
- **Capabilities**:
  - ESC/POS command builder (init, text, alignment, bold/double-size, QR code, barcode, raster/bitmap image, cut, cash drawer kick).
  - TSPL/CPCL basic support for Xprinter label printers (e.g., XP-365B).
  - Character encoding & codepage mapping (UTF-8, CP1258 / TCVN3 for Vietnamese, CP437, CP850, GBK).
  - Persistent or in-memory print job queue with retry/backoff and idempotency keys.
  - Payload parsing for JSON, Base64 raw bytes, and structured receipt JSON.

### 2. `crates/print-transport`
- **Responsibility**: Physical and network I/O abstraction (`PrinterTransport` trait).
- **Transports**:
  - `tcp`: Raw TCP socket to port `9100` (with `TCP_NODELAY`, configurable connect/write timeouts, matching `print-bridge.mjs`).
  - `usb`: Direct USB bulk-transfer printing (`rusb`) for Xprinter USB POS printers without needing OS drivers.
  - `serial`: RS-232 / USB-to-Serial COM port printing (`serialport`).
  - `spooler`: Fallback to OS print spooler (CUPS on macOS/Linux, Winspool on Windows) for driver-installed printers.
  - `mobile_bridge`: Callback trait implemented by Swift/Kotlin on iOS/Android for CoreBluetooth / Android Bluetooth SPP / USB OTG.

### 3. `crates/print-daemon`
- **Responsibility**: Desktop local service binary.
- **Capabilities**:
  - HTTP/1.1 + WebSocket server bound to `127.0.0.1:3100` (configurable).
  - Strict CORS + Private Network Access (PNA) headers (`Access-Control-Allow-Private-Network: true`) so `https://app.bnos.space` works in Chrome/Edge/Safari without mixed-content or PNA blocks.
  - Printer discovery (mDNS/Bonjour, subnet 9100 scan, USB VID/PID enumeration).
  - Structured logging & local SQLite/JSON config for saved printer profiles.

### 4. `crates/print-ffi`
- **Responsibility**: Expose `print-core` and `print-transport` (TCP + payload encoding + job formatting) to Swift (iOS) and Kotlin (Android) via **UniFFI**.

## Applications (`apps/`)

- **`apps/desktop`**: Tray icon, quick settings window (select default Xprinter, test print, view logs), and installers (`.dmg`, `.msi`, `.AppImage`/`.deb`).
- **`apps/ios`**: SwiftUI utility app registering `bnos-print://` custom URL scheme and Universal Links. Uses `Network.framework` (TCP 9100) and `CoreBluetooth` (BLE) / `ExternalAccessory` (MFi) + `print-ffi`.
- **`apps/android`**: Kotlin app registering `bnos-print://` Intent Filter and App Links. Uses `Socket` (TCP 9100), `BluetoothAdapter` (Classic SPP & BLE), and `UsbManager` (USB OTG) + `print-ffi`.

## Web SDK (`packages/web-sdk`)

- Auto-detects environment:
  - **Desktop Browser**: Sends `POST http://127.0.0.1:3100/print` (with health check `GET /health` and WebSocket status stream).
  - **iOS / Android Browser**: Encodes job into `bnos-print://print?...` (or fetches short-lived job token if payload > URL limit) and triggers navigation/deeplink, optionally receiving result via `callback` URL.
