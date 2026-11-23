# Desktop Daemon & Tray Agent

## Scope
- `crates/print-daemon`
- `apps/desktop`

## Responsibilities
1. Build the local HTTP/1.1 + WebSocket server on `127.0.0.1:3100` (configurable via `LISTEN_PORT`).
2. Ensure 100% compatibility with `print-bridge.mjs` endpoints (`POST /print`) plus `/health`, `/printers`, and `/ws`.
3. Include `Access-Control-Allow-Origin` and `Access-Control-Allow-Private-Network: true` headers on all responses and `OPTIONS` preflights.
4. Build system tray integration (status indicator, default printer selector, test print button, open logs) and OS background service manifests:
   - macOS: `LaunchAgent` plist
   - Windows: Startup / Windows Service
   - Linux: `systemd --user` unit + `udev` rules for Xprinter USB

## Constraints
- Bind strictly to `127.0.0.1` by default (never `0.0.0.0` unless explicitly enabled by user config).
- Keep memory footprint minimal (< 25 MB RSS idle) and startup time < 200ms.
