# AGENTS.md — BNOS & BDGO OS Native Print Service

This file provides guidance for AI coding agents working on the `print-service` repository.

## Project Mission
Build a zero-dependency, multi-platform native print service for **BDGO OS** and **BNOS** (`https://app.bnos.space`), replacing the legacy Node.js `print-bridge.mjs` bridge with:
1. **Desktop (macOS, Windows, Linux)**: Native daemon + system tray listening on `http://127.0.0.1:3100` (and WebSocket), supporting LAN TCP `9100`, USB, Serial, Bluetooth, and OS Spooler for Xprinter POS and ESC/POS printers.
2. **Mobile (iOS, Android)**: Native companion apps triggered via **`bnos-print://` deeplinks** (and Universal/App Links) from mobile web browsers, supporting LAN TCP `9100`, Bluetooth (BLE / Classic SPP), and USB OTG (Android).
3. **Web SDK (`packages/web-sdk`)**: Unified TypeScript client for `app.bnos.space` and BDGO OS web apps.

## Core Architectural Rules
1. **Backward Compatibility with `print-bridge.mjs`**:
   - `POST http://127.0.0.1:3100/print` MUST continue to accept:
     - Raw `application/octet-stream` ESC/POS bytes.
     - JSON `{ "type": "test" }` and `{ "ip": "...", "port": 9100, "dataBase64": "..." }`.
   - Must return `{ "ok": true, "bytes": <n> }` on success (`200`) and `{ "ok": false, "error": "..." }` on failure (`400`/`502`).
   - Always include CORS headers + `Access-Control-Allow-Private-Network: true` for HTTPS-to-localhost requests.
2. **Shared Rust Core**:
   - All ESC/POS, TSPL, codepage/raster rendering, and job queueing belong in `crates/print-core` — never duplicate protocol byte logic in Swift, Kotlin, or TypeScript.
   - Expose core functions to iOS/Android via `crates/print-ffi` (UniFFI).
3. **Xprinter Reliability**:
   - Always send `ESC @` (`1B 40`) and `FS .` (`1C 2E`) before text when initializing Xprinter receipt jobs.
   - Prefer 1-bit raster (`GS v 0`) fallback for complex Vietnamese/Unicode receipts when hardware codepage support is uncertain.
4. **Deeplink Safety**:
   - Validate `callback` URLs against an allowlist (`*.bnos.space`, `*.bdgo.vn`, `localhost`) before redirecting back to the browser.
   - Keep deeplink payloads compact (`gzip` + `base64url`) or use `jobUrl` for payloads > 16 KB.

## Specialized Agents
See [`.agents/agents/`](./.agents/agents/) for role-specific instructions:
- [`rust-core-agent.md`](./.agents/agents/rust-core-agent.md) — `print-core`, `print-transport`, `print-ffi`
- [`desktop-agent.md`](./.agents/agents/desktop-agent.md) — `print-daemon` & `apps/desktop`
- [`mobile-agent.md`](./.agents/agents/mobile-agent.md) — `apps/ios` & `apps/android`
- [`web-sdk-agent.md`](./.agents/agents/web-sdk-agent.md) — `packages/web-sdk`
- [`qa-printer-agent.md`](./.agents/agents/qa-printer-agent.md) — Mock printer tests & protocol verification

## Key Docs
- [Architecture](./docs/ARCHITECTURE.md)
- [Protocol & Deeplink Spec](./docs/PROTOCOL_AND_DEEPLINK.md)
- [Printer Support](./docs/PRINTER_SUPPORT.md)
- [Platform Guides](./docs/PLATFORM_GUIDES.md)
