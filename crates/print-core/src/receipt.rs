use serde::{Deserialize, Serialize};

use crate::escpos::{Alignment, CutMode, EscPosBuilder};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReceiptAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReceiptCut {
    Full,
    Partial,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptLineItem {
    pub text: String,
    pub align: Option<ReceiptAlign>,
    pub bold: Option<bool>,
    pub width: Option<u8>,
    pub height: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptPayload {
    pub title: Option<String>,
    pub lines: Vec<ReceiptLineItem>,
    pub cut: Option<ReceiptCut>,
    pub kick_drawer: Option<bool>,
    pub codepage: Option<u8>,
}

impl ReceiptPayload {
    pub fn render_escpos(&self) -> Vec<u8> {
        let mut builder = EscPosBuilder::new()
            .init()
            .codepage(self.codepage.unwrap_or(0));

        if let Some(title) = &self.title {
            builder = builder
                .align(Alignment::Center)
                .bold(true)
                .text_size(2, 2)
                .line(title)
                .text_size(1, 1)
                .bold(false)
                .align(Alignment::Left);
        }

        for item in &self.lines {
            let align = match item.align {
                Some(ReceiptAlign::Center) => Alignment::Center,
                Some(ReceiptAlign::Right) => Alignment::Right,
                _ => Alignment::Left,
            };
            builder = builder
                .align(align)
                .bold(item.bold.unwrap_or(false))
                .text_size(item.width.unwrap_or(1), item.height.unwrap_or(1))
                .line(&item.text);
        }

        builder = builder
            .text_size(1, 1)
            .bold(false)
            .align(Alignment::Left)
            .feed(3);

        if self.kick_drawer.unwrap_or(false) {
            builder = builder.kick_drawer();
        }

        let cut = match self.cut {
            Some(ReceiptCut::Partial) => CutMode::Partial,
            _ => CutMode::Full,
        };
        builder.cut(cut).build()
    }
}
