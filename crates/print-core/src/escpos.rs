/// ESC/POS command builder tailored for Xprinter POS and compatible thermal printers.
#[derive(Debug, Default, Clone)]
pub struct EscPosBuilder {
    buf: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Left = 0,
    Center = 1,
    Right = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CutMode {
    Full,
    Partial,
}

impl EscPosBuilder {
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }

    /// Initialize printer (`ESC @`) and cancel Chinese character mode (`FS .`)
    /// for reliable Xprinter single-byte/codepage output.
    pub fn init(mut self) -> Self {
        self.buf.extend_from_slice(&[0x1B, 0x40, 0x1C, 0x2E]);
        self
    }

    /// Select hardware codepage (`ESC t n`).
    pub fn codepage(mut self, page: u8) -> Self {
        self.buf.extend_from_slice(&[0x1B, 0x74, page]);
        self
    }

    /// Set text alignment (`ESC a n`).
    pub fn align(mut self, align: Alignment) -> Self {
        self.buf.extend_from_slice(&[0x1B, 0x61, align as u8]);
        self
    }

    /// Enable or disable bold text (`ESC E n`).
    pub fn bold(mut self, enabled: bool) -> Self {
        self.buf.extend_from_slice(&[0x1B, 0x45, u8::from(enabled)]);
        self
    }

    /// Set character size multiplier (`GS ! n`), 1..=8 for width and height.
    pub fn text_size(mut self, width: u8, height: u8) -> Self {
        let w = width.saturating_sub(1).min(7);
        let h = height.saturating_sub(1).min(7);
        self.buf.extend_from_slice(&[0x1D, 0x21, (w << 4) | h]);
        self
    }

    /// Append UTF-8/ASCII text followed by a newline.
    pub fn line(mut self, text: &str) -> Self {
        self.buf.extend_from_slice(text.as_bytes());
        self.buf.push(b'\n');
        self
    }

    /// Append raw ESC/POS bytes.
    pub fn raw(mut self, bytes: &[u8]) -> Self {
        self.buf.extend_from_slice(bytes);
        self
    }

    /// Feed `lines` lines (`ESC d n`).
    pub fn feed(mut self, lines: u8) -> Self {
        self.buf.extend_from_slice(&[0x1B, 0x64, lines]);
        self
    }

    /// Cut paper (`GS V`).
    pub fn cut(mut self, mode: CutMode) -> Self {
        match mode {
            CutMode::Full => self.buf.extend_from_slice(&[0x1D, 0x56, 0x00]),
            CutMode::Partial => self.buf.extend_from_slice(&[0x1D, 0x56, 0x42, 0x00]),
        }
        self
    }

    /// Kick cash drawer on pin 2 (`ESC p 0 25 250`).
    pub fn kick_drawer(mut self) -> Self {
        self.buf.extend_from_slice(&[0x1B, 0x70, 0x00, 0x19, 0xFA]);
        self
    }

    /// Encode a 1-bit monochrome raster bitmap (`GS v 0`).
    /// `width_bytes` is the row stride in bytes (`ceil(width_px / 8)`).
    pub fn raster_bitmap(mut self, width_bytes: u16, height_dots: u16, data: &[u8]) -> Self {
        let xl = (width_bytes & 0xFF) as u8;
        let xh = (width_bytes >> 8) as u8;
        let yl = (height_dots & 0xFF) as u8;
        let yh = (height_dots >> 8) as u8;
        self.buf
            .extend_from_slice(&[0x1D, 0x76, 0x30, 0x00, xl, xh, yl, yh]);
        self.buf.extend_from_slice(data);
        self
    }

    pub fn build(self) -> Vec<u8> {
        self.buf
    }

    /// Diagnostic test receipt compatible with the legacy `print-bridge.mjs` test payload.
    pub fn test_receipt() -> Vec<u8> {
        Self::new()
            .init()
            .codepage(0)
            .line("BNOS Print Service Test OK")
            .feed(3)
            .cut(CutMode::Full)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_receipt_starts_with_xprinter_init() {
        let bytes = EscPosBuilder::test_receipt();
        assert_eq!(&bytes[..7], &[0x1B, 0x40, 0x1C, 0x2E, 0x1B, 0x74, 0x00]);
        assert!(bytes.ends_with(&[0x1D, 0x56, 0x00]));
    }
}
