export type TextAlign = 'left' | 'center' | 'right';
export type CutMode = 'full' | 'partial';

export interface ReceiptLineItem {
  text: string;
  align?: TextAlign;
  bold?: boolean;
  width?: number;
  height?: number;
}

export interface ReceiptPayload {
  title?: string;
  lines: ReceiptLineItem[];
  cut?: CutMode;
  kickDrawer?: boolean;
  codepage?: number;
}

const ALIGN_BYTE: Record<TextAlign, number> = {
  left: 0,
  center: 1,
  right: 2,
};

/**
 * Lightweight ESC/POS byte builder for BNOS / BDGO OS web clients.
 * Always initializes with `ESC @` (1B 40) + `FS .` (1C 2E) for Xprinter reliability.
 */
export class EscPosBuilder {
  private readonly chunks: number[] = [];
  private readonly encoder = new TextEncoder();

  init(codepage = 0): this {
    this.chunks.push(0x1b, 0x40, 0x1c, 0x2e, 0x1b, 0x74, codepage & 0xff);
    return this;
  }

  align(align: TextAlign): this {
    this.chunks.push(0x1b, 0x61, ALIGN_BYTE[align]);
    return this;
  }

  bold(enabled: boolean): this {
    this.chunks.push(0x1b, 0x45, enabled ? 1 : 0);
    return this;
  }

  textSize(width = 1, height = 1): this {
    const w = Math.max(0, Math.min(7, width - 1));
    const h = Math.max(0, Math.min(7, height - 1));
    this.chunks.push(0x1d, 0x21, (w << 4) | h);
    return this;
  }

  line(text: string): this {
    const bytes = this.encoder.encode(text);
    for (let i = 0; i < bytes.length; i += 1) {
      this.chunks.push(bytes[i]);
    }
    this.chunks.push(0x0a);
    return this;
  }

  feed(lines = 3): this {
    this.chunks.push(0x1b, 0x64, Math.max(0, Math.min(255, lines)));
    return this;
  }

  cut(mode: CutMode = 'full'): this {
    if (mode === 'partial') {
      this.chunks.push(0x1d, 0x56, 0x42, 0x00);
    } else {
      this.chunks.push(0x1d, 0x56, 0x00);
    }
    return this;
  }

  kickDrawer(): this {
    this.chunks.push(0x1b, 0x70, 0x00, 0x19, 0xfa);
    return this;
  }

  build(): Uint8Array {
    return new Uint8Array(this.chunks);
  }
}

export function renderReceiptToEscPos(receipt: ReceiptPayload): Uint8Array {
  const builder = new EscPosBuilder().init(receipt.codepage ?? 0);

  if (receipt.title) {
    builder.align('center').bold(true).textSize(2, 2).line(receipt.title);
    builder.textSize(1, 1).bold(false).align('left');
  }

  for (const item of receipt.lines) {
    builder
      .align(item.align ?? 'left')
      .bold(Boolean(item.bold))
      .textSize(item.width ?? 1, item.height ?? 1)
      .line(item.text);
  }

  builder.textSize(1, 1).bold(false).align('left').feed(3);

  if (receipt.kickDrawer) {
    builder.kickDrawer();
  }
  builder.cut(receipt.cut ?? 'full');

  return builder.build();
}
