import assert from 'node:assert/strict';
import test from 'node:test';
import {
  BnosPrintClient,
  EscPosBuilder,
  buildDeeplinkUrl,
  buildDeeplinkUrlAsync,
  detectPrintPlatform,
  parseCallbackResult,
  renderReceiptToEscPos,
  toBase64Url,
} from '../src/index.ts';

test('detectPrintPlatform identifies desktop, ios, and android', () => {
  assert.equal(
    detectPrintPlatform('Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)'),
    'desktop'
  );
  assert.equal(
    detectPrintPlatform('Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X)'),
    'ios'
  );
  assert.equal(
    detectPrintPlatform('Mozilla/5.0 (Linux; Android 14; Pixel 8)'),
    'android'
  );
});

test('buildDeeplinkUrl encodes base64url payload and query params', () => {
  const raw = new Uint8Array([0x1b, 0x40, 0x1c, 0x2e]);
  const url = buildDeeplinkUrl({
    type: 'raw',
    transport: 'tcp',
    ip: '192.168.2.100',
    port: 9100,
    data: raw,
    cut: true,
    callback: 'https://app.bnos.space/pos',
  });

  const parsed = new URL(url);
  assert.equal(parsed.protocol, 'bnos-print:');
  assert.equal(parsed.hostname, 'print');
  assert.equal(parsed.searchParams.get('v'), '1');
  assert.equal(parsed.searchParams.get('ip'), '192.168.2.100');
  assert.equal(parsed.searchParams.get('data'), toBase64Url(raw));
  assert.equal(parsed.searchParams.get('cut'), '1');
});

test('buildDeeplinkUrlAsync gzips payloads exceeding 16 KB', async () => {
  const large = new Uint8Array(20_000).fill(0x41);
  const url = await buildDeeplinkUrlAsync({
    type: 'raw',
    ip: '192.168.2.100',
    data: large,
  });
  const parsed = new URL(url);
  assert.equal(parsed.searchParams.get('enc'), 'gzip');
  assert.ok((parsed.searchParams.get('data') ?? '').length < 1000);
});

test('BnosPrintClient sends desktop HTTP POST /print and lists printers', async () => {
  const calls: string[] = [];
  const mockFetch = (async (input: RequestInfo | URL) => {
    const url = String(input);
    calls.push(url);
    if (url.endsWith('/printers')) {
      return new Response(
        JSON.stringify({
          ok: true,
          printers: [{ id: 'xp', name: 'Xprinter', transport: 'tcp', address: '192.168.2.100:9100', protocol: 'escpos', paperWidthMm: 80 }],
        }),
        { status: 200, headers: { 'Content-Type': 'application/json' } }
      );
    }
    return new Response(
      JSON.stringify({ ok: true, bytes: 37, jobId: 'job_000001' }),
      { status: 200, headers: { 'Content-Type': 'application/json' } }
    );
  }) as typeof fetch;

  const client = new BnosPrintClient({
    platform: 'desktop',
    fetchImpl: mockFetch,
  });

  const printers = await client.listPrinters();
  assert.equal(printers.length, 1);
  assert.equal(printers[0].name, 'Xprinter');

  const res = await client.printTest({ ip: '192.168.2.100', port: 9100 });
  assert.equal(res.ok, true);
  assert.equal(res.mode, 'desktop-http');
  assert.equal(res.jobId, 'job_000001');
  assert.deepEqual(calls, [
    'http://127.0.0.1:3100/printers',
    'http://127.0.0.1:3100/print',
  ]);
});

test('BnosPrintClient reports needsInstall when desktop daemon is unreachable', async () => {
  const failingFetch = (async () => {
    throw new TypeError('fetch failed');
  }) as unknown as typeof fetch;

  const client = new BnosPrintClient({
    platform: 'desktop',
    fetchImpl: failingFetch,
  });

  const health = await client.checkHealth();
  assert.equal(health.ok, false);
  assert.equal(health.needsInstall, true);

  const res = await client.printTest();
  assert.equal(res.ok, false);
  assert.equal(res.needsInstall, true);
});

test('BnosPrintClient navigates to deeplink on iOS', async () => {
  let navigatedTo = '';
  const client = new BnosPrintClient({
    platform: 'ios',
    navigate: (url) => {
      navigatedTo = url;
    },
  });

  const res = await client.print({
    type: 'test',
    transport: 'tcp',
    ip: '192.168.2.100',
  });

  assert.equal(res.ok, true);
  assert.equal(res.mode, 'mobile-deeplink');
  assert.ok(navigatedTo.startsWith('bnos-print://print?'));
});

test('parseCallbackResult parses success and error states', () => {
  assert.deepEqual(parseCallbackResult('?ok=1&jobId=job_123'), {
    ok: true,
    jobId: 'job_123',
    error: undefined,
  });
  assert.deepEqual(parseCallbackResult('https://app.bnos.space/pos?ok=0&error=Timeout'), {
    ok: false,
    jobId: undefined,
    error: 'Timeout',
  });
});

test('EscPosBuilder and renderReceiptToEscPos emit Xprinter init and cut bytes', () => {
  const bytes = renderReceiptToEscPos({
    title: 'BNOS POS',
    lines: [{ text: 'Item A x1     50.000', align: 'left' }],
    kickDrawer: true,
    cut: 'partial',
  });

  // Starts with ESC @ (1B 40) + FS . (1C 2E) + ESC t 0 (1B 74 00)
  assert.deepEqual(Array.from(bytes.slice(0, 7)), [0x1b, 0x40, 0x1c, 0x2e, 0x1b, 0x74, 0x00]);
  // Ends with kickDrawer (1B 70 00 19 FA) + partial cut (1D 56 42 00)
  assert.deepEqual(Array.from(bytes.slice(-9)), [
    0x1b, 0x70, 0x00, 0x19, 0xfa, 0x1d, 0x56, 0x42, 0x00,
  ]);

  const manual = new EscPosBuilder().init().line('Hello').cut('full').build();
  assert.deepEqual(Array.from(manual.slice(-3)), [0x1d, 0x56, 0x00]);
});
