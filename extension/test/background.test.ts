import { afterEach, describe, expect, it, vi } from 'vitest';
import { checkFromSender, disableSite, enableSite, handleCheck, syncSites } from '../src/background';
import { buildSystemPrompt } from '../src/rules';
import { CIENTIFICO, SKILL } from '../src/skills';
import { DEFAULT_SETTINGS } from '../src/settings';
import type { CheckRequest, Settings } from '../src/types';

const settings: Settings = { ...DEFAULT_SETTINGS, keys: { claude: 'sk-test', deepseek: 'sk-test', ollama: '', ollaya: '' } };
const req: CheckRequest = { type: 'check', key: 'k1', text: 'texto a nível de teste aqui', level: 'estrito' };
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

  it('aviso with an erro inside it returns 1 suggestion', async () => {
    const erro = { original: 'nível', replacement: 'nivel', rule: 'PTW-2', severity: 'erro', occurrence: 1 };
    const body = { content: [{ type: 'text', text: JSON.stringify({ suggestions: [suggestion, erro] }) }] };
    const r = await handleCheck(req, { fetch: okFetch(body), settings });
    expect(r.ok && r.suggestions).toEqual([{ ...suggestion }]);
  });

  it('401 -> auth, 429 -> rate-limit', async () => {
    expect(await handleCheck(req, { fetch: okFetch({}, 401), settings })).toEqual({ ok: false, error: 'auth' });
    expect(await handleCheck(req, { fetch: okFetch({}, 429), settings })).toEqual({ ok: false, error: 'rate-limit' });
  });

  it('rejecting fetch -> network', async () => {
    const f = vi.fn(async () => { throw new TypeError('fail'); }) as unknown as typeof fetch;
    expect(await handleCheck(req, { fetch: f, settings })).toEqual({ ok: false, error: 'network' });
  });

  it('ollama: checks without a key; down or 403 -> local', async () => {
    const ollama: Settings = { ...DEFAULT_SETTINGS, provider: 'ollama' };
    const body = { message: { content: JSON.stringify({ suggestions: [suggestion] }) }, done_reason: 'stop' };
    const r = await handleCheck(req, { fetch: okFetch(body), settings: ollama });
    expect(r.ok && r.suggestions).toHaveLength(1);
    expect(await handleCheck(req, { fetch: okFetch({}, 403), settings: ollama })).toEqual({ ok: false, error: 'local' });
    const down = vi.fn(async () => { throw new TypeError('fail'); }) as unknown as typeof fetch;
    expect(await handleCheck(req, { fetch: down, settings: ollama })).toEqual({ ok: false, error: 'local' });
  });

  it('ollaya: one decide request per sentence, detections back; down or 403 -> local-ollaya; truncated -> provider', async () => {
    const ollaya: Settings = { ...DEFAULT_SETTINGS, provider: 'ollaya' };
    const text = 'Foi decidido que o deploy será feito. A equipe valida o build hoje.';
    const first = 'Foi decidido que o deploy será feito.';
    const body = { answers: { regra: { type: 'choice', choice: 'PTW-1', probabilities: { 'PTW-1': 0.9 } } }, state_truncated: false };
    const f = okFetch(body);
    const r = await handleCheck({ ...req, text }, { fetch: f, settings: ollaya });
    expect(r.ok && r.suggestions[0]).toEqual({ original: first, replacement: '', detectOnly: true, rule: 'PTW-1 (90%)', severity: 'aviso', occurrence: 1 });
    expect(r.ok && r.suggestions.map((s) => s.original)).toEqual([first, 'A equipe valida o build hoje.']);
    const [url, init] = (f as any).mock.calls[0];
    expect(url).toBe('http://127.0.0.1:11435/api/decide');
    expect((f as any).mock.calls).toHaveLength(2);
    expect(JSON.parse(init.body).model).toBe('decima');
    expect(JSON.parse(init.body).state).toBe(first);
    expect(await handleCheck(req, { fetch: okFetch({}, 403), settings: ollaya })).toEqual({ ok: false, error: 'local-ollaya' });
    const down = vi.fn(async () => { throw new TypeError('fail'); }) as unknown as typeof fetch;
    expect(await handleCheck(req, { fetch: down, settings: ollaya })).toEqual({ ok: false, error: 'local-ollaya' });
    const cut = { ...body, state_truncated: true };
    expect(await handleCheck({ ...req, text }, { fetch: okFetch(cut), settings: ollaya })).toEqual({ ok: false, error: 'provider' });
  });

  it('ollaya + ollama: Ollaya picks the style, Ollama writes in it; Ollaya down, Ollama writes alone', async () => {
    const both: Settings = { ...DEFAULT_SETTINGS, provider: 'ollama', ollayaStyle: true };
    const chat = { message: { content: JSON.stringify([suggestion]) }, done_reason: 'stop' };
    const f = vi.fn(async (url: string) =>
      new Response(JSON.stringify(url.endsWith('/api/decide') ? { answers: { estilo: { choice: 'conciso' } } } : chat)),
    ) as unknown as typeof fetch;
    const r = await handleCheck(req, { fetch: f, settings: both });
    expect(r.ok && r.suggestions[0].rule).toBe('PTW-1 · estilo conciso');
    const [first, second] = (f as any).mock.calls;
    expect(first[0]).toBe('http://127.0.0.1:11435/api/decide');
    expect(second[0]).toBe('http://127.0.0.1:11434/api/chat');
    expect(JSON.parse(second[1].body).messages[0].content).toContain('Estilo da reescrita: `conciso`');
    const down = vi.fn(async (url: string) => {
      if (url.endsWith('/api/decide')) throw new TypeError('fail');
      return new Response(JSON.stringify(chat));
    }) as unknown as typeof fetch;
    const alone = await handleCheck(req, { fetch: down, settings: both });
    expect(alone.ok && alone.suggestions[0].rule).toBe('PTW-1');
  });

  it('ollama and ollaya replies pass through filterSuggestions', async () => {
    const ollama: Settings = { ...DEFAULT_SETTINGS, provider: 'ollama', neverFix: ['nível'] };
    const chat = { message: { content: JSON.stringify([suggestion]) }, done_reason: 'stop' };
    expect(await handleCheck(req, { fetch: okFetch(chat), settings: ollama })).toEqual({ ok: true, suggestions: [] });
    const missing = { message: { content: JSON.stringify([{ ...suggestion, original: 'não está no texto' }]) }, done_reason: 'stop' };
    const plain: Settings = { ...DEFAULT_SETTINGS, provider: 'ollama' };
    expect(await handleCheck(req, { fetch: okFetch(missing), settings: plain })).toEqual({ ok: true, suggestions: [] });
    // A detection is not a rewrite: "só erros" and the never-fix list keep it.
    const ollaya: Settings = { ...DEFAULT_SETTINGS, provider: 'ollaya', rewrite: false, neverFix: ['deploy'] };
    const text = 'Foi decidido que o deploy será feito.';
    const body = { answers: { regra: { type: 'choice', choice: 'PTW-1', probabilities: { 'PTW-1': 0.9 } } }, state_truncated: false };
    const r = await handleCheck({ ...req, text }, { fetch: okFetch(body), settings: ollaya });
    expect(r.ok && r.suggestions.map((s) => s.original)).toEqual([text]);
  });

  it('a truncated reply -> provider, not "no problems"', async () => {
    const cut = { ...claudeBody, stop_reason: 'max_tokens' };
    expect(await handleCheck(req, { fetch: okFetch(cut), settings })).toEqual({ ok: false, error: 'provider' });
    const ds = { choices: [{ finish_reason: 'length', message: { content: '[{"original":' } }] };
    const dsSettings: Settings = { ...settings, provider: 'deepseek' };
    expect(await handleCheck(req, { fetch: okFetch(ds), settings: dsSettings })).toEqual({ ok: false, error: 'provider' });
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

describe('checkFromSender', () => {
  const site = (enabled: boolean): Settings => ({
    ...settings,
    sites: { ...settings.sites, 'example.com': { enabled, level: 'leve' } },
  });
  const tab = (id: number, url = 'https://example.com/x') => ({ tab: { id, url }, frameId: 0 });

  it('a sender whose host is not enabled gets [] and no fetch', async () => {
    const f = okFetch(claudeBody);
    const r = await checkFromSender(req, tab(1), { fetch: f, settings: site(false) });
    expect(r).toEqual({ ok: true, suggestions: [] });
    expect(f).not.toHaveBeenCalled();
  });

  it('the extension switched off: no fetch', async () => {
    const f = okFetch(claudeBody);
    const r = await checkFromSender(req, tab(1), { fetch: f, settings: { ...site(true), enabled: false } });
    expect(r).toEqual({ ok: true, suggestions: [] });
    expect(f).not.toHaveBeenCalled();
  });

  it('a sender with no url: no fetch', async () => {
    const f = okFetch(claudeBody);
    expect(await checkFromSender(req, {}, { fetch: f, settings: site(true) })).toEqual({ ok: true, suggestions: [] });
    expect(f).not.toHaveBeenCalled();
  });

  it('takes the level from the settings, not from the message', async () => {
    const f = okFetch(claudeBody);
    await checkFromSender({ ...req, level: 'estrito' }, tab(1), { fetch: f, settings: site(true) });
    const [, init] = (f as unknown as ReturnType<typeof vi.fn>).mock.calls[0] as [string, RequestInit];
    const system = JSON.parse(init.body as string).system as string;
    const s = site(true);
    expect(system).toBe(buildSystemPrompt('leve', SKILL, CIENTIFICO, { tone: s.tone, rewrite: s.rewrite, neverFix: s.neverFix }));
  });

  it('passes the tone, rewrite and never-fix settings into the prompt', async () => {
    const f = okFetch(claudeBody);
    const s: Settings = { ...site(true), tone: 'informal', rewrite: false, neverFix: ['Kafka'] };
    await checkFromSender(req, tab(1), { fetch: f, settings: s });
    const [, init] = (f as unknown as ReturnType<typeof vi.fn>).mock.calls[0] as [string, RequestInit];
    const system = JSON.parse(init.body as string).system as string;
    expect(system).toContain('mantenha a voz e as gírias');
    expect(system).toContain('não devolva nenhum aviso');
    expect(system).toContain('nunca troque estas palavras, nem dentro de uma frase reescrita: Kafka');
  });

  it('falls back to sender.url', async () => {
    const f = okFetch(claudeBody);
    const r = await checkFromSender(req, { url: 'https://example.com/y' }, { fetch: f, settings: site(true) });
    expect(r.ok && r.suggestions).toHaveLength(1);
  });

  it('the same message key from two tabs does not abort either', async () => {
    const signals: AbortSignal[] = [];
    const resolvers: Array<() => void> = [];
    const f = vi.fn((_u: unknown, init?: RequestInit) => {
      signals.push(init!.signal!);
      return new Promise<Response>((ok) =>
        resolvers.push(() => ok(new Response(JSON.stringify(claudeBody), { status: 200 }))),
      );
    }) as unknown as typeof fetch;
    const a = checkFromSender(req, tab(1), { fetch: f, settings: site(true) });
    const b = checkFromSender(req, tab(2), { fetch: f, settings: site(true) });
    await vi.waitFor(() => expect(signals).toHaveLength(2));
    expect(signals.map((s) => s.aborted)).toEqual([false, false]);
    resolvers.forEach((r) => r());
    for (const r of [await a, await b]) expect(r.ok && r.suggestions).toHaveLength(1);
  });
});
