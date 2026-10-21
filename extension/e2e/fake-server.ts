import { readFile } from 'node:fs/promises';
import { createServer } from 'node:http';
import type { AddressInfo } from 'node:net';
import { basename, dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// A stand-in for both providers: no test ever reaches a real API.
const PAGES = join(dirname(fileURLToPath(import.meta.url)), 'pages');
const SUGGESTION = { original: 'Foi decidido', replacement: 'A equipe decidiu', rule: 'PTW-1', severity: 'erro' };

export interface FakeServer {
  port: number;
  calls: { path: string; host: string }[];
  close(): Promise<void>;
}

export async function startFakeServer(): Promise<FakeServer> {
  const calls: FakeServer['calls'] = [];
  const text = JSON.stringify([SUGGESTION]);
  const server = createServer(async (req, res) => {
    const path = new URL(req.url ?? '/', 'http://x').pathname;
    if (req.method === 'POST') {
      for await (const _ of req); // drain the body
      calls.push({ path, host: req.headers.host ?? '' });
      const body =
        path === '/v1/messages'
          ? { content: [{ type: 'text', text }] }
          : path === '/chat/completions'
            ? { choices: [{ message: { role: 'assistant', content: text } }] }
            : null;
      res.writeHead(body ? 200 : 404, { 'content-type': 'application/json' });
      res.end(JSON.stringify(body ?? { error: 'not found' }));
      return;
    }
    try {
      const html = await readFile(join(PAGES, basename(path)));
      res.writeHead(200, { 'content-type': 'text/html; charset=utf-8' });
      res.end(html);
    } catch {
      res.writeHead(404).end();
    }
  });
  await new Promise<void>((ok) => server.listen(0, '127.0.0.1', ok));
  return {
    port: (server.address() as AddressInfo).port,
    calls,
    close: () => new Promise((ok, fail) => server.close((e) => (e ? fail(e) : ok()))),
  };
}
