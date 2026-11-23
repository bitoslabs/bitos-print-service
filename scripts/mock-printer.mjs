#!/usr/bin/env node
/**
 * Local mock TCP 9100 thermal printer for QA and protocol verification.
 * Usage:
 *   MOCK_PRINTER_PORT=9100 node scripts/mock-printer.mjs
 */
import net from 'node:net';

const port = Number(process.env.MOCK_PRINTER_PORT || 9100);
const host = process.env.MOCK_PRINTER_HOST || '127.0.0.1';

const server = net.createServer((socket) => {
  const chunks = [];
  socket.on('data', (chunk) => chunks.push(chunk));
  socket.on('end', () => {
    const buf = Buffer.concat(chunks);
    const hexPreview = buf.subarray(0, 32).toString('hex').match(/.{1,2}/g)?.join(' ') ?? '';
    console.log(
      `[mock-printer] Received ${buf.length} bytes | head: ${hexPreview}`
    );
  });
});

server.listen(port, host, () => {
  console.log(`[mock-printer] Listening on tcp://${host}:${port}`);
});
