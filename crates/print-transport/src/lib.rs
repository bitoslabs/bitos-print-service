use std::io::Write;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;

use print_core::{PrintError, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "transport", rename_all = "snake_case")]
pub enum PrinterTarget {
    Tcp {
        ip: String,
        #[serde(default = "default_tcp_port")]
        port: u16,
    },
    Usb {
        vendor_id: u16,
        product_id: u16,
    },
    Serial {
        port_name: String,
        baud_rate: u32,
    },
    Spooler {
        printer_name: String,
    },
}

fn default_tcp_port() -> u16 {
    9100
}

pub trait PrinterTransport: Send + Sync {
    fn send(&self, target: &PrinterTarget, payload: &[u8]) -> Result<usize>;
}

#[derive(Debug, Clone)]
pub struct DefaultTransport {
    pub connect_timeout: Duration,
    pub write_timeout: Duration,
}

impl Default for DefaultTransport {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(12),
            write_timeout: Duration::from_secs(12),
        }
    }
}

impl DefaultTransport {
    fn send_tcp(&self, ip: &str, port: u16, payload: &[u8]) -> Result<usize> {
        let addr_str = format!("{ip}:{port}");
        let addr: SocketAddr = addr_str
            .to_socket_addrs()
            .map_err(|e| PrintError::Transport(format!("Failed to resolve {addr_str}: {e}")))?
            .next()
            .ok_or_else(|| PrintError::Transport(format!("No address resolved for {addr_str}")))?;

        let mut stream = TcpStream::connect_timeout(&addr, self.connect_timeout)
            .map_err(|e| PrintError::Transport(format!("Connect to {addr_str} failed: {e}")))?;

        stream
            .set_nodelay(true)
            .map_err(|e| PrintError::Transport(format!("set_nodelay failed: {e}")))?;
        stream
            .set_write_timeout(Some(self.write_timeout))
            .map_err(|e| PrintError::Transport(format!("set_write_timeout failed: {e}")))?;

        stream
            .write_all(payload)
            .map_err(|e| PrintError::Transport(format!("Write to {addr_str} failed: {e}")))?;
        stream
            .flush()
            .map_err(|e| PrintError::Transport(format!("Flush to {addr_str} failed: {e}")))?;

        Ok(payload.len())
    }
}

impl PrinterTransport for DefaultTransport {
    fn send(&self, target: &PrinterTarget, payload: &[u8]) -> Result<usize> {
        if payload.is_empty() {
            return Err(PrintError::EmptyPayload);
        }

        match target {
            PrinterTarget::Tcp { ip, port } => self.send_tcp(ip, *port, payload),
            PrinterTarget::Usb { .. } => Err(PrintError::Transport(
                "Direct USB transport requires enabling the `usb` feature (rusb)".into(),
            )),
            PrinterTarget::Serial { .. } => Err(PrintError::Transport(
                "Serial transport requires enabling the `serial` feature (serialport)".into(),
            )),
            PrinterTarget::Spooler { .. } => Err(PrintError::Transport(
                "OS Spooler transport is not yet enabled on this build".into(),
            )),
        }
    }
}
