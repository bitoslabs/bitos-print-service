# BNOS & BDGO OS Native Print Service

A high-performance, multi-platform native print service for **BDGO OS** and **BNOS** (`https://app.bnos.space`). Built to replace the legacy Node.js `print-bridge.mjs` bridge with a zero-dependency native runtime across **macOS, Windows, Linux, iOS, and Android**.

## Why Native?

- **Zero Runtime Dependencies**: Single native binary on desktop (no Node.js installation required for POS terminals).
- **Multi-Transport**: Raw LAN TCP (`9100`), USB (direct libusb / POS class), Serial/COM, Bluetooth (Classic SPP & BLE), and OS Print Spooler fallback.
- **Mobile Browser Integration**: Seamless **Deeplink (`bnos-print://`)** and **Universal/App Link** support so web apps in Safari (iOS) and Chrome (Android) can print directly to Xprinter and ESC/POS printers.
- **Backward Compatible**: Drop-in compatible with `print-bridge.mjs` (`POST http://127.0.0.1:3100/print`).
- **Xprinter & ESC/POS First**: Built-in profiles for Xprinter POS series (58mm/80mm receipt & label/TSPL printers) and standard ESC/POS devices.

## Project Structure

```text
print-service/
├── AGENTS.md                     # Root instructions for AI coding agents
├── README.md                     # This file
├── Cargo.toml                    # Rust workspace manifest (core, transport, daemon, ffi)
├── package.json                  # Workspace scripts & JS/TS SDK management
├── .agents/
│   └── agents/                   # Specialized role-based agent definitions
│       ├── rust-core-agent.md
│       ├── desktop-agent.md
│       ├── mobile-agent.md
│       ├── web-sdk-agent.md
│       └── qa-printer-agent.md
├── crates/
│   ├── print-core/               # ESC/POS & TSPL command builder, encoding, job queue
│   ├── print-transport/          # TCP 9100, USB, Serial, Bluetooth, OS Spooler drivers
│   ├── print-daemon/             # Desktop HTTP/WS server + system tray + auto-start
│   └── print-ffi/                # UniFFI / C-ABI bindings for iOS (Swift) & Android (Kotlin)
├── apps/
│   ├── desktop/                  # Desktop packaging & tray/installer configs (macOS/Win/Linux)
│   ├── ios/                      # iOS SwiftUI companion & bnos-print:// deeplink handler
│   └── android/                  # Android Kotlin companion & bnos-print:// deeplink handler
├── packages/
│   └── web-sdk/                  # TypeScript SDK for app.bnos.space & BDGO OS web apps
├── docs/
│   ├── ARCHITECTURE.md           # System architecture & crate/app boundaries
│   ├── PROTOCOL_AND_DEEPLINK.md  # Local HTTP/WS API & mobile deeplink specification
│   ├── PRINTER_SUPPORT.md        # Xprinter POS profiles, ESC/POS, codepages, transports
│   └── PLATFORM_GUIDES.md        # OS permissions, daemons, signing & deployment
└── scripts/                      # Build, codegen (UniFFI), and release automation
```

## Quick Reference: Legacy vs. Native

| Feature            | `print-bridge.mjs` (Reference)       | Native Print Service                                  |
| :----------------- | :----------------------------------- | :---------------------------------------------------- |
| **Runtime**        | Node.js                              | Rust Core + Native Swift/Kotlin                       |
| **Desktop OS**     | macOS / Windows / Linux (manual CLI) | macOS / Windows / Linux (Daemon + Tray)               |
| **Mobile OS**      | Not supported                        | iOS & Android via `bnos-print://` deeplink            |
| **Transports**     | TCP `9100` only                      | TCP `9100`, USB, Serial, BLE/SPP, Spooler             |
| **Local Endpoint** | `http://127.0.0.1:3100/print`        | `http://127.0.0.1:3100/print` + WebSocket + `/health` |

## Documentation

- [BNOS (`app.bnos.space`) Setup Guide](./docs/BNOS_SETUP.md)
- [BNOS Svelte app integration guide](https://github.com/bitoslabs/bnos-svelte/blob/main/docs/THERMAL_PRINTING.md)
- [Build, Run & Development Guide](./docs/BUILD_AND_RUN.md)
- [Architecture](./docs/ARCHITECTURE.md)
- [Protocol & Deeplink Specification](./docs/PROTOCOL_AND_DEEPLINK.md)
- [Printer Support (Xprinter & ESC/POS)](./docs/PRINTER_SUPPORT.md)
- [Platform Build & Deployment Guides](./docs/PLATFORM_GUIDES.md)

**Source:** [github.com/bitoslabs/bitos-print-service](https://github.com/bitoslabs/bitos-print-service)

## Quick Start

```bash
# Run Web SDK tests & Swift checks
pnpm check

# Start mock TCP 9100 printer for local testing
pnpm dev:mock-printer

# Run native Desktop Daemon (requires Rust/Cargo) -> http://127.0.0.1:3100/
pnpm dev:daemon
```
