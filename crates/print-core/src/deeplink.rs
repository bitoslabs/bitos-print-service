use std::io::Read;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::error::{PrintError, Result};
use crate::escpos::EscPosBuilder;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeeplinkTransport {
    Tcp,
    Ble,
    BtSpp,
    Usb,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeeplinkJob {
    pub version: String,
    pub action: String,
    pub job_type: String,
    pub transport: Option<DeeplinkTransport>,
    pub ip: Option<String>,
    pub port: u16,
    pub mac: Option<String>,
    pub payload: Vec<u8>,
    pub job_url: Option<String>,
    pub cut: bool,
    pub callback: Option<String>,
    pub silent: bool,
}

impl DeeplinkJob {
    /// Parse a `bnos-print://print?...` or `https://print.bnos.space/link/print?...` URL.
    pub fn parse(raw_url: &str) -> Result<Self> {
        let parsed = Url::parse(raw_url)
            .map_err(|e| PrintError::InvalidDeeplink(format!("URL parse error: {e}")))?;

        let action = if parsed.scheme() == "bnos-print" {
            parsed.host_str().unwrap_or("print").to_string()
        } else {
            parsed
                .path_segments()
                .and_then(|mut s| s.next_back())
                .unwrap_or("print")
                .to_string()
        };

        let mut version = "1".to_string();
        let mut job_type = "raw".to_string();
        let mut transport = None;
        let mut ip = None;
        let mut port = 9100u16;
        let mut mac = None;
        let mut data_b64 = None;
        let mut enc = None;
        let mut job_url = None;
        let mut cut = false;
        let mut callback = None;
        let mut silent = false;

        for (k, v) in parsed.query_pairs() {
            match k.as_ref() {
                "v" => version = v.into_owned(),
                "type" => job_type = v.into_owned(),
                "transport" => {
                    transport = match v.as_ref() {
                        "tcp" => Some(DeeplinkTransport::Tcp),
                        "ble" => Some(DeeplinkTransport::Ble),
                        "bt_spp" => Some(DeeplinkTransport::BtSpp),
                        "usb" => Some(DeeplinkTransport::Usb),
                        other => {
                            return Err(PrintError::InvalidDeeplink(format!(
                                "Unsupported transport: {other}"
                            )))
                        }
                    }
                }
                "ip" => ip = Some(v.into_owned()),
                "port" => {
                    port = v
                        .parse::<u16>()
                        .map_err(|_| PrintError::InvalidDeeplink("Invalid port".into()))?;
                }
                "mac" => mac = Some(v.into_owned()),
                "data" => data_b64 = Some(v.into_owned()),
                "enc" => enc = Some(v.into_owned()),
                "jobUrl" => job_url = Some(v.into_owned()),
                "cut" => cut = v == "1" || v.eq_ignore_ascii_case("true"),
                "callback" => callback = Some(v.into_owned()),
                "silent" => silent = v == "1" || v.eq_ignore_ascii_case("true"),
                _ => {}
            }
        }

        if let Some(cb) = &callback {
            validate_callback_url(cb)?;
        }

        let payload = if job_type == "test" {
            EscPosBuilder::test_receipt()
        } else if let Some(b64) = data_b64 {
            let decoded = URL_SAFE_NO_PAD
                .decode(b64.trim_end_matches('='))
                .map_err(|e| PrintError::InvalidBase64(e.to_string()))?;
            if enc.as_deref() == Some("gzip") {
                let mut decoder = GzDecoder::new(decoded.as_slice());
                let mut out = Vec::new();
                decoder
                    .read_to_end(&mut out)
                    .map_err(|e| PrintError::InvalidGzip(e.to_string()))?;
                out
            } else {
                decoded
            }
        } else {
            Vec::new()
        };

        Ok(Self {
            version,
            action,
            job_type,
            transport,
            ip,
            port,
            mac,
            payload,
            job_url,
            cut,
            callback,
            silent,
        })
    }
}

/// Allowlist check for deeplink `callback` redirects (`*.bnos.space`, `*.bdgo.vn`, `localhost`).
pub fn validate_callback_url(raw: &str) -> Result<()> {
    let url = Url::parse(raw)
        .map_err(|_| PrintError::DisallowedCallback("Malformed callback URL".into()))?;

    let scheme = url.scheme();
    let host = url
        .host_str()
        .ok_or_else(|| PrintError::DisallowedCallback("Missing callback host".into()))?
        .to_ascii_lowercase();

    let is_localhost = host == "localhost" || host == "127.0.0.1";
    let is_allowed_domain = host == "bnos.space"
        || host.ends_with(".bnos.space")
        || host == "bdgo.vn"
        || host.ends_with(".bdgo.vn");

    if is_localhost && (scheme == "http" || scheme == "https") {
        return Ok(());
    }
    if is_allowed_domain && scheme == "https" {
        return Ok(());
    }

    Err(PrintError::DisallowedCallback(host))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_test_deeplink_with_allowed_callback() {
        let url = "bnos-print://print?v=1&type=test&transport=tcp&ip=192.168.2.100&callback=https%3A%2F%2Fapp.bnos.space%2Fpos";
        let job = DeeplinkJob::parse(url).expect("valid deeplink");
        assert_eq!(job.action, "print");
        assert_eq!(job.ip.as_deref(), Some("192.168.2.100"));
        assert_eq!(job.port, 9100);
        assert!(!job.payload.is_empty());
    }

    #[test]
    fn rejects_untrusted_callback() {
        let url = "bnos-print://print?v=1&type=test&callback=https%3A%2F%2Fevil.example.com";
        assert!(DeeplinkJob::parse(url).is_err());
    }
}
