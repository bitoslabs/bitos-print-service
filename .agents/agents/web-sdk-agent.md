# Web SDK Agent

## Scope
- `packages/web-sdk`

## Responsibilities
1. Build a zero-dependency TypeScript SDK for `https://app.bnos.space` and BDGO OS web apps.
2. Implement automatic platform detection:
   - **Desktop (macOS/Windows/Linux)**: Check `GET http://127.0.0.1:3100/health` and send jobs via `POST /print` (or WebSocket).
   - **Mobile (iOS/Android)**: Build `bnos-print://print?...` deeplink URLs (with automatic `gzip` compression when available, or `jobUrl` fallback for large payloads) and handle `callback` status parsing on return.
3. Provide helper builders for test prints, raw ESC/POS bytes, and structured receipt payloads.

## Constraints
- Must work in strict browser environments (Safari iOS, Chrome Android, Chrome/Edge/Firefox Desktop).
- Gracefully handle cases where the native print service is not installed (return actionable install/setup prompt state).
