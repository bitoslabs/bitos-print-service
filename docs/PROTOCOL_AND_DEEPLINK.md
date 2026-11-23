# Protocol & Deeplink Specification

This document defines the communication protocols between web clients (`app.bnos.space`, BDGO OS web apps) and the native print service on Desktop and Mobile.

---

## 1. Desktop Local HTTP & WebSocket API

Default bind address: `127.0.0.1:3100` (matches `print-bridge.mjs`).

### Required CORS & Private Network Access Headers
Because `https://app.bnos.space` is an HTTPS origin calling `http://127.0.0.1:3100`, the daemon MUST return:
```http
Access-Control-Allow-Origin: *
Access-Control-Allow-Headers: Content-Type, Authorization, X-BNOS-Request-Id
Access-Control-Allow-Methods: GET, POST, OPTIONS
Access-Control-Allow-Private-Network: true
```
*(In production, `Access-Control-Allow-Origin` can be restricted to an allowlist of `*.bnos.space`, `*.bdgo.vn`, and `localhost` origins.)*

### Endpoints

#### `GET /health`
Returns service version, OS, and active printer status.
```json
{
  "ok": true,
  "service": "bnos-print-daemon",
  "version": "0.1.0",
  "platform": "macos-arm64"
}
```

#### `GET /printers`
Lists discovered or configured printers (LAN, USB, Serial, OS Spooler).
```json
{
  "ok": true,
  "printers": [
    {
      "id": "xprinter-80-lan",
      "name": "Xprinter XP-N160I",
      "transport": "tcp",
      "address": "192.168.2.100:9100",
      "protocol": "escpos",
      "paperWidthMm": 80
    }
  ]
}
```

#### `POST /print` (Backward-Compatible with `print-bridge.mjs`)
Accepts either:
1. **Raw binary** (`Content-Type: application/octet-stream`): Raw ESC/POS bytes sent directly to the default printer.
2. **JSON** (`Content-Type: application/json`):

```json
{
  "type": "raw",
  "transport": "tcp",
  "ip": "192.168.2.100",
  "port": 9100,
  "usbVendorId": 1046,
  "usbProductId": 20497,
  "printerId": "xprinter-80-lan",
  "dataBase64": "G0AbLh..."
}
```

Supported `type` values:
- `"test"`: Sends a built-in diagnostic test receipt (`BNOS Print Service Test OK` + paper cut).
- `"raw"`: Decodes `dataBase64` and sends raw bytes.
- `"receipt"`: *(Future/Structured)* Takes structured JSON receipt AST and renders ESC/POS via `print-core`.

**Response (`200 OK`)**:
```json
{
  "ok": true,
  "bytes": 128,
  "jobId": "job_01J..."
}
```

**Error Response (`400` / `502`)**:
```json
{
  "ok": false,
  "error": "Timed out connecting to 192.168.2.100:9100"
}
```

---

## 2. Mobile Deeplink Specification (iOS & Android)

On iOS Safari/Chrome and Android Chrome, web pages cannot directly open raw TCP sockets or access Bluetooth Classic/USB without WebBluetooth/WebUSB limitations (especially on iOS Safari where neither is supported). Instead, the web app invokes the native companion app via **Custom URL Scheme** or **Universal / App Links**.

### URL Scheme
- **Custom Scheme**: `bnos-print://`
- **Universal / App Link (Recommended fallback)**: `https://print.bnos.space/link/...`

### Actions

#### A. Print Action: `bnos-print://print`
Triggers an immediate print job from the mobile browser.

**Query Parameters**:
| Parameter | Required | Description |
| :--- | :--- | :--- |
| `v` | Yes | Protocol version (`1`). |
| `type` | Yes | `raw` \| `test` \| `receipt` |
| `transport` | No | `tcp` \| `ble` \| `bt_spp` \| `usb` (defaults to user's saved default printer). |
| `ip` | Cond. | Printer IP if `transport=tcp` (e.g. `192.168.2.100`). |
| `port` | No | TCP port (default `9100`). |
| `mac` | Cond. | Bluetooth MAC / UUID if `transport=bt_spp` or `ble`. |
| `data` | Cond. | URL-safe Base64 (`base64url`, no padding) of raw ESC/POS bytes (or gzipped JSON if `enc=gzip`). |
| `jobUrl` | Cond. | Alternative to `data` for large receipts (> 16 KB): HTTPS URL where the native app fetches the job payload. |
| `cut` | No | `1` (auto-cut after print) or `0`. |
| `callback` | No | URL to return to after printing (e.g. `https://app.bnos.space/pos?printStatus=`). App appends `ok=1&jobId=...` or `ok=0&error=...`. |
| `silent` | No | `1` to auto-return to `callback` immediately on success without staying in the print app UI. |

**Example Deeplink**:
```text
bnos-print://print?v=1&type=raw&transport=tcp&ip=192.168.2.100&port=9100&data=G0AbLh...&callback=https%3A%2F%2Fapp.bnos.space%2Fpos
```

#### B. Discover / Setup Action: `bnos-print://setup`
Opens the mobile app's printer pairing screen (LAN scan, Bluetooth pairing, test print) and optionally returns to `callback`.

### Payload Size Considerations for Deeplinks
- iOS and Android handle custom URLs reliably up to ~32 KB–64 KB, though keeping URLs under **16 KB** is safest across all mobile browsers.
- For receipts with large logos or long item lists:
  1. Compress ESC/POS payload with `gzip` + `base64url` (`enc=gzip`).
  2. Or upload the job to BNOS API and pass `jobUrl=https://api.bnos.space/v1/print-jobs/{id}?token={short_lived_token}`.

### Security Rules for Mobile Deeplinks
1. **Never auto-print silently to arbitrary external callbacks** without validating the `callback` origin against an allowlist (`*.bnos.space`, `*.bdgo.vn`, `localhost`) or user confirmation.
2. **Local Network / Bluetooth permissions** must be clearly explained on first launch.
