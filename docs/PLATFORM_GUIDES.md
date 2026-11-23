# Platform Build & Deployment Guides

## Desktop (macOS, Windows, Linux)

### Background Service & Auto-Start
- **macOS**:
  - Install binary to `/Applications/BNOSPrint.app` or `/usr/local/bin/bnos-print-daemon`.
  - Register `LaunchAgent` plist at `~/Library/LaunchAgents/space.bnos.print-daemon.plist` with `KeepAlive = true` and `RunAtLoad = true`.
  - Menu bar tray icon for printer selection, test print, and log viewing.
- **Windows**:
  - Install via `.msi` / NSIS installer.
  - Run as per-user tray app (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`) or Windows Service (`sc create BNOSPrintService`).
  - Note on USB: Direct USB via `libusb` on Windows requires either WinUSB driver (Zadig/installer INF) or using the Windows Spooler (`winspool.drv` `RAW` datatype) when the official Xprinter driver is installed.
- **Linux**:
  - Install `.deb` / `.AppImage` / binary at `/usr/local/bin/bnos-print-daemon`.
  - User `systemd` unit at `~/.config/systemd/user/bnos-print.service`.
  - Direct USB requires a `udev` rule (`/etc/udev/rules.d/99-xprinter.rules`) granting `MODE="0666"` or `plugdev` group access to Xprinter USB VID/PIDs.

---

## Mobile (iOS)

### `Info.plist` Requirements
- **URL Scheme**: Register `bnos-print` under `CFBundleURLTypes`.
- **Local Network Permission** (iOS 14+):
  - `NSLocalNetworkUsageDescription`: *"BNOS Print needs local network access to send receipts to your LAN/WiFi thermal printer."*
  - `NSBonjourServices`: `["_pdl-datastream._tcp", "_ipp._tcp"]` (if using mDNS discovery).
- **Bluetooth Permission**:
  - `NSBluetoothAlwaysUsageDescription`: *"BNOS Print uses Bluetooth to connect to portable thermal printers."*
- **Universal Links**: Associated Domains entitlement `applinks:print.bnos.space`.

---

## Mobile (Android)

### `AndroidManifest.xml` Requirements
- **Deeplink Intent Filter**:
  ```xml
  <intent-filter>
      <action android:name="android.intent.action.VIEW" />
      <category android:name="android.intent.category.DEFAULT" />
      <category android:name="android.intent.category.BROWSABLE" />
      <data android:scheme="bnos-print" />
  </intent-filter>
  ```
- **Permissions**:
  - `INTERNET`, `ACCESS_NETWORK_STATE`, `ACCESS_WIFI_STATE` (for TCP 9100 LAN printing).
  - `BLUETOOTH_CONNECT`, `BLUETOOTH_SCAN` (Android 12+ / API 31+) and `BLUETOOTH`, `BLUETOOTH_ADMIN`, `ACCESS_FINE_LOCATION` (Android <= 11).
  - `android.hardware.usb.host` feature + `USB_PERMISSION` runtime prompt for USB OTG Xprinter connection.
