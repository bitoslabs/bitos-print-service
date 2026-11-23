# Configuring BNOS (`app.bnos.space`)

Follow these steps to connect your Xprinter / ESC-POS thermal printer to **BNOS** (`https://app.bnos.space`).

## 1. Add Your Printer in `BNOSPrint`

1. Open **`BNOSPrint.app`** (macOS) or **`BNOSPrintTray.exe`** / **`print-daemon.exe`** (Windows/Linux).
2. Open **Printer Settings...** from the menu bar / system tray (or visit `http://127.0.0.1:3100/`).
3. Enter your printer **Name**, **IP** (e.g. `192.168.2.100`), **Port** (`9100`), and **Paper Width** (`80mm` or `58mm`), then click **Add**.
4. Click **Test** / **Test Print** to confirm it prints.
5. Click **Copy Webhook** (or **Copy Default Webhook**) to copy the exact Webhook URL to your clipboard.

## 2. Add the Printer in BNOS (`https://app.bnos.space`)

> The BNOS Svelte app also ships a first-class **BNOS Print Service** connection
> and a status card in **Settings → Printers**. On desktop it calls this daemon
> at `http://127.0.0.1:3100`; on iOS/Android it hands off through a
> `bnos-print://` deeplink. Prefer that connection when it is available — see the
> [app integration guide](https://github.com/bitoslabs/bnos-svelte/blob/main/docs/THERMAL_PRINTING.md)
> and the [project source](https://github.com/bitoslabs/bitos-print-service).
> The Webhook flow below remains fully supported for hosted/legacy setups.

1. Open **Settings > Printers > Add printer** in BNOS.
2. Fill in **Identity**:
   - **Name**: e.g. `Cashier Xprinter` or `Kitchen printer`
   - **ID**: e.g. `cashier-printer` or `kitchen-printer`
3. Under **CONNECTION**, select either:
   - **BNOS Print Service** (recommended): choose `LAN TCP` and enter the printer IP/port, or Bluetooth/USB. This uses the native daemon/app and the web SDK.
   - **Webhook** (legacy): paste the URL copied in step 1:
     - Default printer:
       ```text
       http://127.0.0.1:3100/print
       ```
     - Specific IP printer (for multi-printer setups like Cashier + Kitchen):
       ```text
       http://127.0.0.1:3100/print?ip=192.168.2.100&port=9100
       ```
4. Click **Add printer** and run a test print from BNOS.

## 3. Sharing with Other Devices on the Local Network (LAN)

By default, `print-daemon` listens only on `127.0.0.1:3100` (same computer).

- **Via `BNOSPrint.app` (macOS)**: Click the menu bar icon and enable **Allow LAN Access (0.0.0.0:3100)**.
- **Via Terminal / CLI**:
  ```bash
  ALLOW_LAN=1 PRINTER_IP=192.168.2.100 ./target/release/print-daemon
  # (or: LISTEN_HOST=0.0.0.0 cargo run -p print-daemon)
  ```

Then other devices on your LAN can reach `http://<this-machine-lan-ip>:3100/print` (shown in the startup log and `GET /health`).

Example test from another device on the LAN (machine at `192.168.2.132`, printer at `192.168.2.100:9100`):

```bash
curl -X POST "http://192.168.2.132:3100/print?ip=192.168.2.100&port=9100" \
  -H "Content-Type: application/json" \
  -d '{"type":"test"}'
```

> [!IMPORTANT]
> Modern browsers (Chrome, Safari) allow `https://app.bnos.space` to call `http://127.0.0.1:3100`, but **block** `https://` pages from calling `http://192.168.x.x:3100` due to Mixed Content rules.
>
> - On **Desktops**: Run `BNOSPrint` on that machine and use `http://127.0.0.1:3100/print`.
> - On **iOS / Android**: Use the native app via `bnos-print://` deeplink, which prints directly to the Xprinter LAN IP (`192.168.x.x:9100`).

## 4. Environment Variables & Config File

| Setting              | Default                       | Description                                        |
| :------------------- | :---------------------------- | :------------------------------------------------- |
| `PRINTER_IP`         | `192.168.2.100`               | Initial default printer IP                         |
| `PRINTER_PORT`       | `9100`                        | Initial default printer TCP port                   |
| `LISTEN_PORT`        | `3100`                        | Local HTTP daemon port                             |
| `ALLOW_LAN`          | `false`                       | Set `1` or `true` to bind `0.0.0.0` for LAN access |
| `LISTEN_HOST`        | `127.0.0.1`                   | Explicit bind host (overrides `ALLOW_LAN`)         |
| `BNOS_PRINTERS_PATH` | `~/.bnos-print/printers.json` | Custom path for saved multi-printer JSON config    |
