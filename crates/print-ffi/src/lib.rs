use print_core::{validate_callback_url, DeeplinkJob, EscPosBuilder};
use print_transport::{DefaultTransport, PrinterTarget, PrinterTransport};

/// Build the diagnostic test receipt bytes (`ESC @`, `FS .`, text, feed, cut).
pub fn build_test_receipt() -> Vec<u8> {
    EscPosBuilder::test_receipt()
}

/// Parse and validate a `bnos-print://` deeplink URL.
pub fn parse_deeplink(url: &str) -> Result<DeeplinkJob, String> {
    DeeplinkJob::parse(url).map_err(|e| e.to_string())
}

/// Validate a deeplink `callback` URL against the BNOS / BDGO OS allowlist.
pub fn is_allowed_callback(url: &str) -> bool {
    validate_callback_url(url).is_ok()
}

/// Send raw ESC/POS bytes to a LAN TCP `9100` printer.
pub fn send_tcp_print(ip: &str, port: u16, payload: &[u8]) -> Result<usize, String> {
    let transport = DefaultTransport::default();
    let target = PrinterTarget::Tcp {
        ip: ip.to_string(),
        port,
    };
    transport.send(&target, payload).map_err(|e| e.to_string())
}
