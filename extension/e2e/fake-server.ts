import { readFile } from 'node:fs/promises';
import { createServer } from 'node:http';
import type { AddressInfo } from 'node:net';
import { basename, dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// A stand-in for every provider (Claude, DeepSeek, Ollama, Ollaya): no test ever reaches a real API.
const PAGES = join(dirname(fileURLToPath(import.meta.url)), 'pages');

export interface Suggestion {
  original: string;
  replacement: string;
  rule: string;
  severity: 'erro' | 'aviso';
}

export const SUGGESTION: Suggestion = { original: 'Foi decidido', replacement: 'A equipe decidiu', rule: 'PTW-1', severity: 'erro' };

export interface Call {
  path: string;
  host: string;
  /** The user text the check sent (Ollaya: the sentence asked about). */
  text: string;
  /** The system prompt, where the request has one. */
  system: string;
  body: Record<string, unknown>;
}

export interface FakeServer {
  port: number;
  base: string;
  calls: Call[];
  /** What a check answers: these suggestions, or a failure with this HTTP status. */
  reply: Suggestion[] | number;
  /** Models the local servers report in /api/tags; /api/pull adds to it. */
  installed: string[];
  reset(): void;
  close(): Promise<void>;
}

type Msg = { role: string; content: string };

function sentText(path: string, body: Record<string, unknown>): { text: string; system: string } {
  if (path === '/api/decide') return { text: String(body.state ?? ''), system: '' };
  const messages = (body.messages ?? []) as Msg[];
  const text = messages.filter((m) => m.role === 'user').at(-1)?.content ?? '';
  const system = String(body.system ?? messages.find((m) => m.role === 'system')?.content ?? '');
  return { text, system };
}

export async function startFakeServer(): Promise<FakeServer> {
  const state: FakeServer = {
    port: 0,
    base: '',
    calls: [],
    reply: [SUGGESTION],
    installed: ['llama3.2:1b'],
    reset() {
      state.calls = [];
      state.reply = [SUGGESTION];
      state.installed = ['llama3.2:1b'];
    },
    close: () => new Promise((ok, fail) => server.close((e) => (e ? fail(e) : ok()))),
  };
  const server = createServer(async (req, res) => {
    const path = new URL(req.url ?? '/', 'http://x').pathname;
    const json = (status: number, body: unknown) => {
      res.writeHead(status, { 'content-type': 'application/json' });
      res.end(JSON.stringify(body));
    };
    if (req.method === 'GET' && path === '/api/tags') return json(200, { models: state.installed.map((name) => ({ name })) });
    if (req.method === 'POST') {
      let raw = '';
      for await (const chunk of req) raw += chunk;
      let body: Record<string, unknown> = {};
      try {
        body = JSON.parse(raw);
      } catch {
        /* an empty body is a call too */
      }
      state.calls.push({ path, host: req.headers.host ?? '', ...sentText(path, body), body });
      if (path === '/api/pull') {
        const model = String(body.model ?? '');
        if (!state.installed.includes(model)) state.installed.push(model);
        return json(200, { status: 'success' });
      }
      if (typeof state.reply === 'number') return json(state.reply, { error: 'failed on purpose' });
      const text = JSON.stringify(state.reply);
      switch (path) {
        case '/v1/messages':
          return json(200, { content: [{ type: 'text', text }], stop_reason: 'end_turn' });
        case '/chat/completions':
          return json(200, { choices: [{ message: { role: 'assistant', content: text }, finish_reason: 'stop' }] });
        case '/api/chat':
          return json(200, { message: { role: 'assistant', content: text }, done_reason: 'stop' });
        case '/api/decide':
          // Every sentence breaks the first suggestion's rule, with 90% confidence.
          return json(200, {
            answers: { regra: { type: 'choice', choice: state.reply[0]?.rule ?? 'PTW-1', probabilities: { [state.reply[0]?.rule ?? 'PTW-1']: 0.9 } } },
            state_truncated: false,
          });
        default:
          return json(404, { error: 'not found' });
      }
    }
    try {
      const html = await readFile(join(PAGES, basename(path)));
      const headers: Record<string, string> = { 'content-type': 'text/html; charset=utf-8' };
      // A page under /csp/ forbids every script and style of its own: the extension must still run.
      if (path.startsWith('/csp/')) headers['content-security-policy'] = "default-src 'none'; script-src 'none'; style-src 'none'";
      res.writeHead(200, headers);
      res.end(html);
    } catch {
      res.writeHead(404).end();
    }
  });
  await new Promise<void>((ok) => server.listen(0, '127.0.0.1', ok));
  state.port = (server.address() as AddressInfo).port;
  state.base = `http://127.0.0.1:${state.port}`;
  return state;
}
