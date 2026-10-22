import { readDecision, readStyle } from './decisions';
import { buildRequest, classifyError, decideRequests, isTruncated, readText, styleRequest } from './providers';
import { buildSystemPrompt } from './rules';
import { DEFAULT_SETTINGS, isLocal, levelFor, loadSettings } from './settings';
import { CIENTIFICO, SKILL } from './skills';
import { parseSuggestions } from './suggestions';
import type { CheckError, CheckRequest, CheckResult, Settings, Style, Suggestion } from './types';

const inflight = new Map<string, AbortController>();

export async function handleCheck(
  req: CheckRequest,
  deps: { fetch: typeof fetch; settings: Settings },
): Promise<CheckResult> {
  const { settings } = deps;
  const provider = settings.provider;
  if (!isLocal(provider) && !settings.keys[provider]) return { ok: false, error: 'no-key' };

  inflight.get(req.key)?.abort();
  const controller = new AbortController();
  inflight.set(req.key, controller);

  try {
    if (provider === 'ollaya') return await checkDecisions(req, deps, controller.signal);
    const style = provider === 'ollama' && settings.ollayaStyle ? await pickStyle(req.text, deps, controller.signal) : null;
    if (controller.signal.aborted) return { ok: true, suggestions: [] };
    const system = buildSystemPrompt(req.level, SKILL, CIENTIFICO, style);
    const { url, init } = buildRequest(provider, settings, system, req.text);
    let res: Response;
    try {
      res = await deps.fetch(url, { ...init, signal: controller.signal });
    } catch {
      return controller.signal.aborted
        ? { ok: true, suggestions: [] }
        : { ok: false, error: classifyError(null, provider) };
    }
    if (controller.signal.aborted) return { ok: true, suggestions: [] };
    if (!res.ok) return { ok: false, error: classifyError(res.status, provider) };
    let json: unknown;
    try {
      json = await res.json();
    } catch {
      return controller.signal.aborted
        ? { ok: true, suggestions: [] }
        : { ok: false, error: 'provider' };
    }
    if (controller.signal.aborted) return { ok: true, suggestions: [] };
    if (isTruncated(provider, json)) return { ok: false, error: 'provider' };
    const suggestions = parseSuggestions(readText(provider, json));
    // The card says which style Ollaya chose, so the rewrite's tone has a visible reason.
    return { ok: true, suggestions: style ? suggestions.map((s) => ({ ...s, rule: `${s.rule} · estilo ${style}` })) : suggestions };
  } finally {
    if (inflight.get(req.key) === controller) inflight.delete(req.key);
  }
}

// Ollaya + Ollama: Ollaya picks the style of the rewrite. A feature flag, so any failure (Ollaya down,
// a 403, an odd reply) gives null and Ollama writes as it does alone.
async function pickStyle(
  text: string,
  deps: { fetch: typeof fetch; settings: Settings },
  signal: AbortSignal,
): Promise<Style | null> {
  const { url, init } = styleRequest(deps.settings, text);
  try {
    const res = await deps.fetch(url, { ...init, signal });
    return res.ok ? readStyle(await res.json()) : null;
  } catch {
    return null;
  }
}

// Ollaya: one /api/decide per sentence, in parallel. Any failure fails the check, as one failed
// reply does for the other providers: a sentence left unread is not "no problems".
async function checkDecisions(
  req: CheckRequest,
  deps: { fetch: typeof fetch; settings: Settings },
  signal: AbortSignal,
): Promise<CheckResult> {
  const results = await Promise.all(
    decideRequests(deps.settings, req.level, req.text).map(
      async ({ ask, url, init }): Promise<{ error: CheckError } | { found: Suggestion | null }> => {
        let res: Response;
        try {
          res = await deps.fetch(url, { ...init, signal });
        } catch {
          return { error: classifyError(null, 'ollaya') };
        }
        if (!res.ok) return { error: classifyError(res.status, 'ollaya') };
        try {
          const json: unknown = await res.json();
          return isTruncated('ollaya', json) ? { error: 'provider' } : { found: readDecision(json, ask) };
        } catch {
          return { error: 'provider' };
        }
      },
    ),
  );
  if (signal.aborted) return { ok: true, suggestions: [] };
  const failed = results.find((r) => 'error' in r);
  if (failed && 'error' in failed) return { ok: false, error: failed.error };
  return { ok: true, suggestions: results.flatMap((r) => ('found' in r && r.found ? [r.found] : [])) };
}

// The part of chrome.runtime.MessageSender the gate reads.
export interface Sender {
  tab?: { id?: number; url?: string };
  frameId?: number;
  url?: string;
}

function senderHost(sender: Sender): string | null {
  const url = sender.tab?.url ?? sender.url;
  if (!url) return null;
  try {
    return new URL(url).hostname;
  } catch {
    return null;
  }
}

// A check from a content script: the sender's site must be enabled now (an open tab may have read
// older settings), and its level comes from the settings, not from the message. Keys are per tab
// and frame, so two tabs never cancel each other.
export async function checkFromSender(
  msg: CheckRequest,
  sender: Sender,
  deps: { fetch: typeof fetch; settings: Settings },
): Promise<CheckResult> {
  const host = senderHost(sender);
  const level = host ? levelFor(host, deps.settings) : null;
  if (!level) return { ok: true, suggestions: [] };
  const key = `${sender.tab?.id}:${sender.frameId}:${msg.key}`;
  return handleCheck({ ...msg, key, level }, deps);
}

// --- Content script registration for sites the user enables ---

type Scripting = Pick<
  typeof chrome.scripting,
  'registerContentScripts' | 'unregisterContentScripts' | 'getRegisteredContentScripts'
>;

const DEFAULT_HOSTS = new Set(Object.keys(DEFAULT_SETTINGS.sites));
const siteId = (host: string) => `site:${host}`;

export async function enableSite(host: string, scripting: Scripting): Promise<void> {
  if (DEFAULT_HOSTS.has(host)) return;
  const id = siteId(host);
  const existing = await scripting.getRegisteredContentScripts({ ids: [id] });
  if (existing.length > 0) return;
  await scripting.registerContentScripts([
    {
      id,
      matches: [`https://${host}/*`],
      js: ['content.js'],
      css: ['content.css'],
      runAt: 'document_idle',
      persistAcrossSessions: true,
    },
  ]);
}

export async function disableSite(host: string, scripting: Scripting): Promise<void> {
  if (DEFAULT_HOSTS.has(host)) return;
  const id = siteId(host);
  const existing = await scripting.getRegisteredContentScripts({ ids: [id] });
  if (existing.length === 0) return;
  await scripting.unregisterContentScripts({ ids: [id] });
}

export async function syncSites(settings: Settings, scripting: Scripting): Promise<void> {
  const registered = new Set((await scripting.getRegisteredContentScripts()).map((s) => s.id));
  const scripts = Object.entries(settings.sites)
    .filter(([host, cfg]) => cfg.enabled && !DEFAULT_HOSTS.has(host) && !registered.has(siteId(host)))
    .map(([host]) => ({
      id: siteId(host),
      matches: [`https://${host}/*`],
      js: ['content.js'],
      css: ['content.css'],
      runAt: 'document_idle' as const,
      persistAcrossSessions: true,
    }));
  if (scripts.length > 0) await scripting.registerContentScripts(scripts);
}

type Message = CheckRequest | { type: 'enable-site'; host: string } | { type: 'disable-site'; host: string };

export function init(): void {
  chrome.runtime.onMessage.addListener((msg: Message, sender, sendResponse) => {
    if (msg?.type === 'check') {
      loadSettings()
        .then((settings) => checkFromSender(msg, sender, { fetch: globalThis.fetch.bind(globalThis), settings }))
        .then(sendResponse, () => sendResponse({ ok: false, error: 'provider' } satisfies CheckResult));
      return true;
    }
    if (msg?.type === 'enable-site' || msg?.type === 'disable-site') {
      const run = msg.type === 'enable-site' ? enableSite : disableSite;
      run(msg.host, chrome.scripting).then(
        () => sendResponse({ ok: true }),
        () => sendResponse({ ok: false }),
      );
      return true;
    }
    return false;
  });
  const sync = () => loadSettings().then((s) => syncSites(s, chrome.scripting)).catch(() => {});
  chrome.runtime.onStartup?.addListener(sync);
  chrome.runtime.onInstalled?.addListener(sync);
}

if (typeof chrome !== 'undefined' && chrome.runtime?.onMessage) init();
