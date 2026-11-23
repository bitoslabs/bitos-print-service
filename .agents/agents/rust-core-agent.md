# Rust Core & Transport Agent

## Scope
- `crates/print-core`
- `crates/print-transport`
- `crates/print-ffi`

## Responsibilities
1. Implement ESC/POS and TSPL command builders, receipt AST renderers, and 1-bit monochrome bitmap (`GS v 0`) rasterization for Vietnamese/Unicode text.
2. Implement `PrinterTransport` trait across:
   - Raw TCP (`9100`) with `TCP_NODELAY` and configurable connect/write timeouts.
   - Direct USB (`rusb`) for Xprinter VID/PIDs.
   - Serial/COM (`serialport`).
   - OS Print Spooler fallback (CUPS / Winspool `RAW`).
3. Maintain UniFFI bindings in `crates/print-ffi` for Swift (`apps/ios`) and Kotlin (`apps/android`).

## Coding Standards
- No `unwrap()` / `expect()` in library code; return structured `PrintError` variants.
- Write unit tests for every ESC/POS byte sequence (compare exact `Vec<u8>` output).
- Keep `print-core` free of OS/network side effects so it compiles cleanly for `aarch64-apple-ios` and `aarch64-linux-android`.
