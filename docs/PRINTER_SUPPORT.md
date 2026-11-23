# Printer Support (Xprinter POS & ESC/POS)

## Target Hardware

### 1. Xprinter POS Thermal Receipt Printers (Primary)
- **80mm Network / USB / WiFi / Bluetooth**:
  - `XP-N160I`, `XP-Q200`, `XP-Q800`, `XP-T80Q`, `XP-C260`, `XP-S200M`
  - Standard ESC/POS command set, TCP port `9100`, USB Printer Class (`0x07`) or Vendor-specific bulk endpoints.
  - Common Xprinter USB VIDs: `0x0483` (STMicroelectronics), `0x1FC9` (NXP), `0x0416` (Winbond), `0x1504` (Bixolon/Xprinter OEM), `0x0FE6` (ICS Advent).
- **58mm Portable / Desktop**:
  - `XP-58IIH`, `XP-P300`, `XP-P800`, `XP-58` series.
  - Bluetooth Classic SPP (Android) / BLE (iOS & Android) / USB.

### 2. Xprinter Label Printers (TSPL / CPCL)
- `XP-365B`, `XP-420B`, `XP-470B`, `XP-DT108B`
- Uses **TSPL** (Taiwan Semiconductor Printer Language) for barcode/shipping labels, and some models support switching to ESC/POS mode.

### 3. Generic ESC/POS Compatible Printers
- Epson TM-T82 / TM-T88, Star Micronics (ESC/POS emulation), Bixolon, Rongta, ZJiang, Sunmi built-in printers.

---

## ESC/POS Command Reference (Core Subset)

| Function | Hex Bytes | Notes |
| :--- | :--- | :--- |
| Initialize Printer | `1B 40` (`ESC @`) | Resets formatting & buffer |
| Cancel Chinese Mode | `1C 2E` (`FS .`) | Important on Xprinter before single-byte/UTF-8 codepages |
| Select Codepage | `1B 74 n` (`ESC t n`) | `n=0` (CP437), `n=19` (CP858), `n=30` (TCVN3/CP1258 depending on firmware) |
| Bold On / Off | `1B 45 01` / `1B 45 00` | `ESC E n` |
| Align Left/Center/Right | `1B 61 00` / `01` / `02` | `ESC a n` |
| Text Size | `1D 21 n` (`GS ! n`) | Width & height multiplier |
| Feed & Cut (Full/Partial) | `1D 56 00` / `1D 56 42 00` | `GS V` |
| Kick Cash Drawer (Pin 2) | `1B 70 00 19 FA` | `ESC p 0 25 250` |
| Raster Bit Image | `1D 76 30 m xL xH yL yH d...` | `GS v 0` — best fallback for Vietnamese/Unicode text & logos |

### Vietnamese & Unicode Handling on Xprinter
Many Xprinter firmware versions have inconsistent built-in Vietnamese codepages (CP1258 vs TCVN3 vs UTF-8). `print-core` supports two strategies:
1. **Hardware Codepage Mode**: `ESC @` -> `FS .` -> `ESC t <page>` for fast ASCII / supported codepage printing.
2. **Raster / Bitmap Fallback**: Render complex Unicode (Vietnamese diacritics, CJK, custom fonts, logos) into 1-bit monochrome bitmap and send via `GS v 0` (`1D 76 30`). This guarantees 100% accurate rendering on every Xprinter model regardless of firmware font ROM.

---

## Transport Matrix

| Transport | macOS | Windows | Linux | iOS | Android |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **LAN / WiFi TCP (`9100`)** | Yes | Yes | Yes | Yes (`Network.framework`) | Yes (`Socket`) |
| **Direct USB (`libusb`)** | Yes | Yes (WinUSB/Driver) | Yes (`udev` rule) | No (MFi only) | Yes (`UsbManager` OTG) |
| **Serial / COM** | Yes | Yes | Yes | No | Via USB-Serial |
| **Bluetooth Classic (SPP)** | Yes | Yes | Yes | No (MFi only) | Yes (`BluetoothSocket`) |
| **Bluetooth Low Energy (BLE)** | Yes | Yes | Yes | Yes (`CoreBluetooth`) | Yes (`BluetoothGatt`) |
| **OS Spooler (CUPS/Winspool)** | Yes | Yes | Yes | AirPrint only | PrintService |
