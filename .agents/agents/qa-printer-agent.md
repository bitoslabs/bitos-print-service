# QA & Printer Verification Agent

## Scope
- Cross-cutting tests across `crates/`, `packages/web-sdk/`, `apps/`, and `scripts/`

## Responsibilities
1. Maintain a local mock TCP `9100` printer server in tests to capture and assert exact ESC/POS byte streams without requiring physical Xprinter hardware.
2. Verify backward compatibility against `print-bridge.mjs` behavior (`POST /print` with raw binary, `{ type: "test" }`, `{ ip, port, dataBase64 }`).
3. Verify deeplink URL encoding/decoding round-trips (`base64url`, `gzip`, `callback` validation).
4. Validate CORS and `Access-Control-Allow-Private-Network: true` headers on HTTP responses.
