import { afterEach, describe, expect, it, vi } from 'vitest';
import { disableSite, enableSite, handleCheck, syncSites } from '../src/background';
import { DEFAULT_SETTINGS } from '../src/settings';
import type { CheckRequest, Settings } from '../src/types';

const settings: Settings = { ...DEFAULT_SETTINGS, keys: { claude: 'sk-test', deepseek: 'sk-test' } };
const req: CheckRequest = { type: 'check', key: 'k1', text: 'texto de teste aqui', level: 'estrito' };
const suggestion = { original: 'a nível de', replacement: 'em', rule: 'PTW-1', severity: 'aviso', occurrence: 1 };
const claudeBody = { content: [{ type: 'text', text: JSON.stringify({ suggestions: [suggestion] }) }] };

const okFetch = (body: unknown, status = 200) =>
  vi.fn(async () => new Response(JSON.stringify(body), { status })) as unknown as typeof fetch;

afterEach(() => vi.restoreAllMocks());

describe('handleCheck', () => {
  it('no-key: does not call fetch', async () => {
    const f = okFetch(claudeBody);
    const r = await handleCheck(req, { fetch: f, settings: DEFAULT_SETTINGS });
    expect(r).toEqual({ ok: false, error: 'no-key' });
    expect(f).not.toHaveBeenCalled();
  });

  it('200 returns suggestions', async () => {
    const r = await handleCheck(req, { fetch: okFetch(claudeBody), settings });
    expect(r.ok && r.suggestions).toHaveLength(1);
  });

  it('401 -> auth, 429 -> rate-limit', async () => {
    expect(await handleCheck(req, { fetch: okFetch({}, 401), settings })).toEqual({ ok: false, error: 'auth' });
    expect(await handleCheck(req, { fetch: okFetch({}, 429), settings })).toEqual({ ok: false, error: 'rate-limit' });
  });

  it('rejecting fetch -> network', async () => {
    const f = vi.fn(async () => { throw new TypeError('fail'); }) as unknown as typeof fetch;
    expect(await handleCheck(req, { fetch: f, settings })).toEqual({ ok: false, error: 'network' });
  });

  it('2xx with non-JSON body -> provider', async () => {
    const f = vi.fn(async () => new Response('<html>', { status: 200 })) as unknown as typeof fetch;
    expect(await handleCheck(req, { fetch: f, settings })).toEqual({ ok: false, error: 'provider' });
  });

  it('second call with same key aborts the first, which returns []', async () => {
    const signals: AbortSignal[] = [];
    const f = vi.fn((_u: unknown, init?: RequestInit) => {
      const signal = init!.signal!;
      signals.push(signal);
      if (signals.length === 1) {
        return new Promise<Response>((_res, rej) =>
          signal.addEventListener('abort', () => rej(new DOMException('aborted', 'AbortError'))),
        );
      }
      return Promise.resolve(new Response(JSON.stringify(claudeBody), { status: 200 }));
    }) as unknown as typeof fetch;
    const first = handleCheck(req, { fetch: f, settings });
    const second = handleCheck(req, { fetch: f, settings });
    expect(signals[0].aborted).toBe(true);
    expect(await first).toEqual({ ok: true, suggestions: [] });
    const r2 = await second;
    expect(r2.ok && r2.suggestions).toHaveLength(1);
  });

  it('keeps the key out of console', async () => {
    const spies = (['log', 'info', 'warn', 'error', 'debug'] as const).map((m) => vi.spyOn(console, m));
    await handleCheck(req, { fetch: okFetch(claudeBody), settings });
    await handleCheck(req, { fetch: okFetch({}, 401), settings });
    for (const s of spies) expect(JSON.stringify(s.mock.calls)).not.toContain('sk-test');
  });
});

function scriptingStub(registered: string[] = []) {
  const ids = new Set(registered);
  return {
    ids,
    getRegisteredContentScripts: vi.fn(async (f?: { ids?: string[] }) =>
      [...ids].filter((id) => !f?.ids || f.ids.includes(id)).map((id) => ({ id }))),
    registerContentScripts: vi.fn(async (s: Array<{ id: string }>) => { s.forEach((x) => ids.add(x.id)); }),
    unregisterContentScripts: vi.fn(async (f: { ids: string[] }) => { f.ids.forEach((i) => ids.delete(i)); }),
  };
}

describe('site registration', () => {
  it('enable-site registers site:<host> for https://<host>/*', async () => {
    const s = scriptingStub();
    await enableSite('example.com', s as never);
    expect(s.registerContentScripts).toHaveBeenCalledWith([
      expect.objectContaining({
        id: 'site:example.com',
        matches: ['https://example.com/*'],
        js: ['content.js'],
        css: ['content.css'],
        runAt: 'document_idle',
      }),
    ]);
  });

  it('never registers default hosts, and is idempotent', async () => {
    const s = scriptingStub();
    await enableSite('github.com', s as never);
    await enableSite('example.com', s as never);
    await enableSite('example.com', s as never);
    expect(s.registerContentScripts).toHaveBeenCalledTimes(1);
  });

  it('disable-site unregisters', async () => {
    const s = scriptingStub(['site:example.com']);
    await disableSite('example.com', s as never);
    expect(s.unregisterContentScripts).toHaveBeenCalledWith({ ids: ['site:example.com'] });
  });

  it('startup skips registered ids and default hosts', async () => {
    const s = scriptingStub(['site:a.com']);
    const st: Settings = {
      ...settings,
      sites: {
        ...settings.sites,
        'a.com': { enabled: true, level: 'descritivo' },
        'b.com': { enabled: true, level: 'descritivo' },
        'c.com': { enabled: false, level: 'descritivo' },
      },
    };
    await syncSites(st, s as never);
    expect(s.registerContentScripts).toHaveBeenCalledTimes(1);
    const arg = s.registerContentScripts.mock.calls[0][0];
    expect(arg.map((x) => x.id)).toEqual(['site:b.com']);
  });
});
