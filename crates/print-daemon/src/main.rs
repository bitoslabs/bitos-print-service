use std::env;
use std::fs;
use std::net::UdpSocket;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use print_core::EscPosBuilder;
use print_transport::{DefaultTransport, PrinterTarget, PrinterTransport};
use serde::{Deserialize, Serialize};
use tiny_http::{Header, Method, Response, Server, StatusCode};

static JOB_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PrinterConfig {
    id: String,
    name: String,
    transport: String,
    ip: String,
    port: u16,
    protocol: String,
    paper_width_mm: u16,
    is_default: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddPrinterRequest {
    name: String,
    ip: String,
    port: Option<u16>,
    paper_width_mm: Option<u16>,
    is_default: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PrintJsonRequest {
    #[serde(rename = "type")]
    job_type: Option<String>,
    printer_id: Option<String>,
    ip: Option<String>,
    port: Option<u16>,
    data_base64: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PrintSuccessResponse {
    ok: bool,
    bytes: usize,
    job_id: String,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    ok: bool,
    error: String,
}

struct PrinterStore {
    path: PathBuf,
    printers: Vec<PrinterConfig>,
}

impl PrinterStore {
    fn load(default_ip: &str, default_port: u16) -> Self {
        let path = env::var("BNOS_PRINTERS_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = env::var("HOME")
                    .or_else(|_| env::var("USERPROFILE"))
                    .unwrap_or_else(|_| ".".to_string());
                PathBuf::from(home).join(".bnos-print").join("printers.json")
            });

        let printers = fs::read_to_string(&path)
            .ok()
            .and_then(|raw| serde_json::from_str::<Vec<PrinterConfig>>(&raw).ok())
            .filter(|list| !list.is_empty())
            .unwrap_or_else(|| {
                vec![PrinterConfig {
                    id: "default-lan".to_string(),
                    name: "Default Xprinter LAN".to_string(),
                    transport: "tcp".to_string(),
                    ip: default_ip.to_string(),
                    port: default_port,
                    protocol: "escpos".to_string(),
                    paper_width_mm: 80,
                    is_default: true,
                }]
            });

        Self { path, printers }
    }

    fn save(&self) {
        if let Some(parent) = self.path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&self.printers) {
            let _ = fs::write(&self.path, json);
        }
    }

    fn resolve_target(&self, printer_id: Option<&str>, ip: Option<&str>, port: Option<u16>) -> (String, u16) {
        if let Some(explicit_ip) = ip.map(str::trim).filter(|s| !s.is_empty()) {
            return (explicit_ip.to_string(), port.unwrap_or(9100));
        }
        if let Some(id) = printer_id {
            if let Some(found) = self.printers.iter().find(|p| p.id == id) {
                return (found.ip.clone(), port.unwrap_or(found.port));
            }
        }
        if let Some(def) = self.printers.iter().find(|p| p.is_default).or_else(|| self.printers.first()) {
            return (def.ip.clone(), port.unwrap_or(def.port));
        }
        ("192.168.2.100".to_string(), port.unwrap_or(9100))
    }

    fn add(&mut self, req: AddPrinterRequest) -> PrinterConfig {
        let id = format!(
            "printer_{}",
            JOB_COUNTER.fetch_add(1, Ordering::Relaxed)
        );
        let make_default = req.is_default.unwrap_or(self.printers.is_empty());
        if make_default {
            for p in &mut self.printers {
                p.is_default = false;
            }
        }
        let item = PrinterConfig {
            id,
            name: req.name.trim().to_string(),
            transport: "tcp".to_string(),
            ip: req.ip.trim().to_string(),
            port: req.port.unwrap_or(9100),
            protocol: "escpos".to_string(),
            paper_width_mm: req.paper_width_mm.unwrap_or(80),
            is_default: make_default,
        };
        self.printers.push(item.clone());
        self.save();
        item
    }

    fn remove(&mut self, id: &str) -> bool {
        let prev_len = self.printers.len();
        self.printers.retain(|p| p.id != id);
        if self.printers.len() != prev_len {
            if !self.printers.iter().any(|p| p.is_default) {
                if let Some(first) = self.printers.first_mut() {
                    first.is_default = true;
                }
            }
            self.save();
            true
        } else {
            false
        }
    }

    fn set_default(&mut self, id: &str) -> bool {
        if !self.printers.iter().any(|p| p.id == id) {
            return false;
        }
        for p in &mut self.printers {
            p.is_default = p.id == id;
        }
        self.save();
        true
    }
}

fn cors_headers() -> Vec<Header> {
    vec![
        Header::from_bytes("Access-Control-Allow-Origin", "*").expect("valid header"),
        Header::from_bytes(
            "Access-Control-Allow-Headers",
            "Content-Type, Authorization, X-BNOS-Request-Id",
        )
        .expect("valid header"),
        Header::from_bytes("Access-Control-Allow-Methods", "GET, POST, DELETE, OPTIONS")
            .expect("valid header"),
        Header::from_bytes("Access-Control-Allow-Private-Network", "true").expect("valid header"),
    ]
}

fn json_response<T: Serialize>(status: u16, body: &T) -> Response<std::io::Cursor<Vec<u8>>> {
    let bytes = serde_json::to_vec(body).unwrap_or_else(|_| b"{\"ok\":false}".to_vec());
    let mut resp = Response::from_data(bytes).with_status_code(StatusCode(status));
    resp.add_header(Header::from_bytes("Content-Type", "application/json").expect("valid"));
    for h in cors_headers() {
        resp.add_header(h);
    }
    resp
}

fn html_response(html: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let mut resp = Response::from_string(html).with_status_code(StatusCode(200));
    resp.add_header(Header::from_bytes("Content-Type", "text/html; charset=utf-8").expect("valid"));
    resp
}

fn empty_response(status: u16) -> Response<std::io::Empty> {
    let mut resp = Response::empty(StatusCode(status));
    for h in cors_headers() {
        resp.add_header(h);
    }
    resp
}

fn next_job_id() -> String {
    let seq = JOB_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("job_{seq:06}")
}

fn query_param(url: &str, key: &str) -> Option<String> {
    let query = url.split_once('?')?.1;
    for pair in query.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            if k == key && !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}

const SETTINGS_UI_HTML: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width,initial-scale=1" />
  <title>BNOS Print Service — Settings</title>
  <style>
    body { font-family: -apple-system, BlinkMacSystemFont, sans-serif; max-width: 680px; margin: 32px auto; padding: 0 16px; color: #111; }
    h1 { font-size: 20px; margin-bottom: 4px; }
    .card { border: 1px solid #e5e7eb; border-radius: 10px; padding: 16px; margin: 16px 0; }
    .row { display: flex; gap: 8px; flex-wrap: wrap; align-items: center; }
    input, select { padding: 8px 10px; border: 1px solid #d1d5db; border-radius: 6px; font-size: 14px; }
    button { padding: 8px 12px; border: 0; border-radius: 6px; background: #111827; color: #fff; cursor: pointer; font-size: 13px; }
    button.sec { background: #e5e7eb; color: #111827; }
    button.danger { background: #dc2626; }
    .printer { display: flex; justify-content: space-between; align-items: center; padding: 10px 0; border-bottom: 1px solid #f3f4f6; }
    .badge { font-size: 11px; background: #dcfce7; color: #166534; padding: 2px 8px; border-radius: 999px; margin-left: 6px; }
    #status { font-size: 13px; color: #374151; margin-top: 8px; }
  </style>
</head>
<body>
  <h1>BNOS &amp; BDGO OS Print Service</h1>
  <div id="status">Loading printers...</div>

  <div class="card">
    <strong>Add IP Printer (Xprinter / ESC-POS)</strong>
    <div class="row" style="margin-top:10px">
      <input id="name" placeholder="Printer Name (e.g. Cashier 80mm)" style="flex:1" />
      <input id="ip" placeholder="192.168.2.100" style="width:140px" />
      <input id="port" type="number" value="9100" style="width:80px" />
      <select id="width"><option value="80">80mm</option><option value="58">58mm</option></select>
      <button onclick="addPrinter()">Add</button>
    </div>
  </div>

  <div class="card">
    <div style="display:flex;justify-content:space-between;align-items:center">
      <strong>Configured Printers</strong>
      <button class="sec" onclick="copyUrl('http://127.0.0.1:3100/print')">Copy Default Webhook</button>
    </div>
    <div id="list" style="margin-top:8px"></div>
  </div>

  <script>
    const setStatus = (msg) => { document.getElementById('status').textContent = msg; };
    async function copyUrl(url) {
      try {
        await navigator.clipboard.writeText(url);
        setStatus(`Copied Webhook URL: ${url}`);
      } catch {
        prompt('Copy BNOS Webhook URL:', url);
      }
    }
    async function load() {
      const res = await fetch('/printers');
      const data = await res.json();
      const list = document.getElementById('list');
      list.innerHTML = '';
      for (const p of data.printers || []) {
        const webhookUrl = `http://127.0.0.1:3100/print?ip=${encodeURIComponent(p.ip)}&port=${p.port}`;
        const el = document.createElement('div');
        el.className = 'printer';
        el.innerHTML = `
          <div>
            <strong>${p.name}</strong>${p.isDefault ? '<span class="badge">Default</span>' : ''}
            <div style="font-size:12px;color:#6b7280">${p.ip}:${p.port} · ${p.paperWidthMm}mm</div>
          </div>
          <div class="row">
            <button onclick="testPrint('${p.id}')">Test</button>
            <button class="sec" onclick="copyUrl('${webhookUrl}')">Copy Webhook</button>
            ${p.isDefault ? '' : `<button class="sec" onclick="setDefault('${p.id}')">Set Default</button>`}
            <button class="danger" onclick="removePrinter('${p.id}')">Delete</button>
          </div>`;
        list.appendChild(el);
      }
      setStatus(`Ready · ${(data.printers || []).length} printer(s) configured`);
    }
    async function addPrinter() {
      const name = document.getElementById('name').value.trim() || 'Xprinter LAN';
      const ip = document.getElementById('ip').value.trim();
      const port = Number(document.getElementById('port').value || 9100);
      const paperWidthMm = Number(document.getElementById('width').value || 80);
      if (!ip) return setStatus('Please enter printer IP');
      await fetch('/printers', { method: 'POST', headers: {'Content-Type':'application/json'}, body: JSON.stringify({ name, ip, port, paperWidthMm }) });
      document.getElementById('name').value = '';
      document.getElementById('ip').value = '';
      await load();
    }
    async function testPrint(printerId) {
      setStatus('Sending test print...');
      const res = await fetch('/print', { method: 'POST', headers: {'Content-Type':'application/json'}, body: JSON.stringify({ type: 'test', printerId }) });
      const data = await res.json();
      setStatus(data.ok ? `Test print OK (${data.bytes} bytes)` : `Error: ${data.error}`);
    }
    async function setDefault(id) {
      await fetch(`/printers/${id}/default`, { method: 'POST' });
      await load();
    }
    async function removePrinter(id) {
      await fetch(`/printers/${id}`, { method: 'DELETE' });
      await load();
    }
    load();
  </script>
</body>
</html>"#;

fn detect_lan_ip() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let ip = socket.local_addr().ok()?.ip();
    if ip.is_loopback() {
        None
    } else {
        Some(ip.to_string())
    }
}

fn main() {
    let default_ip = env::var("PRINTER_IP").unwrap_or_else(|_| "192.168.2.100".to_string());
    let default_port: u16 = env::var("PRINTER_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(9100);
    let listen_port: u16 = env::var("LISTEN_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3100);
    let allow_lan = env::var("ALLOW_LAN")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let listen_host = env::var("LISTEN_HOST").unwrap_or_else(|_| {
        if allow_lan {
            "0.0.0.0".to_string()
        } else {
            "127.0.0.1".to_string()
        }
    });
    let lan_ip = if listen_host == "0.0.0.0" {
        detect_lan_ip()
    } else {
        None
    };

    let store = Arc::new(Mutex::new(PrinterStore::load(&default_ip, default_port)));
    let bind_addr = format!("{listen_host}:{listen_port}");
    let server = Server::http(&bind_addr).expect("Failed to bind local print daemon");
    let transport = DefaultTransport::default();

    println!("BNOS print daemon listening on http://{bind_addr} (UI: http://127.0.0.1:{listen_port}/)");
    if let Some(ref ip) = lan_ip {
        println!("LAN print endpoint enabled: http://{ip}:{listen_port}/print");
    }

    for mut req in server.incoming_requests() {
        if *req.method() == Method::Options {
            let _ = req.respond(empty_response(204));
            continue;
        }

        let path = req.url().split('?').next().unwrap_or("/").to_string();

        match (req.method(), path.as_str()) {
            (&Method::Get, "/") => {
                let _ = req.respond(html_response(SETTINGS_UI_HTML));
            }
            (&Method::Get, "/health") => {
                let body = serde_json::json!({
                    "ok": true,
                    "service": "bnos-print-daemon",
                    "version": env!("CARGO_PKG_VERSION"),
                    "platform": std::env::consts::OS,
                    "listenHost": listen_host,
                    "lanIp": lan_ip,
                });
                let _ = req.respond(json_response(200, &body));
            }
            (&Method::Get, "/printers") => {
                let guard = store.lock().expect("store lock");
                let printers: Vec<serde_json::Value> = guard
                    .printers
                    .iter()
                    .map(|p| {
                        serde_json::json!({
                            "id": p.id,
                            "name": p.name,
                            "transport": p.transport,
                            "ip": p.ip,
                            "port": p.port,
                            "address": format!("{}:{}", p.ip, p.port),
                            "protocol": p.protocol,
                            "paperWidthMm": p.paper_width_mm,
                            "isDefault": p.is_default,
                        })
                    })
                    .collect();
                let _ = req.respond(json_response(200, &serde_json::json!({ "ok": true, "printers": printers })));
            }
            (&Method::Post, "/printers") => {
                let mut raw = Vec::new();
                if req.as_reader().read_to_end(&mut raw).is_err() {
                    let _ = req.respond(json_response(400, &ErrorResponse { ok: false, error: "Invalid body".into() }));
                    continue;
                }
                match serde_json::from_slice::<AddPrinterRequest>(&raw) {
                    Ok(add_req) if !add_req.ip.trim().is_empty() => {
                        let created = store.lock().expect("store lock").add(add_req);
                        let _ = req.respond(json_response(201, &serde_json::json!({ "ok": true, "printer": created })));
                    }
                    _ => {
                        let _ = req.respond(json_response(400, &ErrorResponse { ok: false, error: "Printer name and ip are required".into() }));
                    }
                }
            }
            (&Method::Post, p) if p.starts_with("/printers/") && p.ends_with("/default") => {
                let id = p.trim_start_matches("/printers/").trim_end_matches("/default");
                let ok = store.lock().expect("store lock").set_default(id);
                let _ = req.respond(json_response(if ok { 200 } else { 404 }, &serde_json::json!({ "ok": ok })));
            }
            (&Method::Delete, p) if p.starts_with("/printers/") => {
                let id = p.trim_start_matches("/printers/");
                let ok = store.lock().expect("store lock").remove(id);
                let _ = req.respond(json_response(if ok { 200 } else { 404 }, &serde_json::json!({ "ok": ok })));
            }
            (&Method::Post, "/print") => {
                let is_json = req.headers().iter().any(|h| {
                    h.field.as_str().as_str().eq_ignore_ascii_case("content-type")
                        && h.value.as_str().contains("application/json")
                });

                let mut raw = Vec::new();
                if let Err(err) = req.as_reader().read_to_end(&mut raw) {
                    let _ = req.respond(json_response(400, &ErrorResponse { ok: false, error: format!("Failed to read request body: {err}") }));
                    continue;
                }

                let req_url = req.url().to_string();
                let mut requested_printer_id: Option<String> = query_param(&req_url, "printerId");
                let mut requested_ip: Option<String> = query_param(&req_url, "ip");
                let mut requested_port: Option<u16> = query_param(&req_url, "port")
                    .and_then(|p| p.parse::<u16>().ok())
                    .filter(|&p| p > 0);

                let payload: Vec<u8> = if is_json {
                    match serde_json::from_slice::<PrintJsonRequest>(&raw) {
                        Ok(body) => {
                            if body.printer_id.is_some() {
                                requested_printer_id = body.printer_id;
                            }
                            if body.ip.is_some() {
                                requested_ip = body.ip;
                            }
                            if let Some(p) = body.port.filter(|&p| p > 0) {
                                requested_port = Some(p);
                            }
                            if body.job_type.as_deref() == Some("test") {
                                EscPosBuilder::test_receipt()
                            } else if let Some(b64) = body.data_base64 {
                                match STANDARD.decode(b64.trim()) {
                                    Ok(bytes) => bytes,
                                    Err(e) => {
                                        let _ = req.respond(json_response(400, &ErrorResponse { ok: false, error: format!("Invalid dataBase64: {e}") }));
                                        continue;
                                    }
                                }
                            } else {
                                Vec::new()
                            }
                        }
                        Err(e) => {
                            let _ = req.respond(json_response(400, &ErrorResponse { ok: false, error: format!("Invalid JSON body: {e}") }));
                            continue;
                        }
                    }
                } else {
                    raw
                };

                if payload.is_empty() {
                    let _ = req.respond(json_response(400, &ErrorResponse { ok: false, error: "Empty print payload".into() }));
                    continue;
                }

                let (target_ip, target_port) = store.lock().expect("store lock").resolve_target(
                    requested_printer_id.as_deref(),
                    requested_ip.as_deref(),
                    requested_port,
                );

                let target = PrinterTarget::Tcp { ip: target_ip, port: target_port };
                match transport.send(&target, &payload) {
                    Ok(bytes) => {
                        let _ = req.respond(json_response(200, &PrintSuccessResponse { ok: true, bytes, job_id: next_job_id() }));
                    }
                    Err(err) => {
                        let _ = req.respond(json_response(502, &ErrorResponse { ok: false, error: err.to_string() }));
                    }
                }
            }
            _ => {
                let _ = req.respond(empty_response(405));
            }
        }
    }
}
