import { buildRequest, classifyError, readText } from './providers';
import { buildSystemPrompt } from './rules';
import { DEFAULT_SETTINGS, levelFor, loadSettings } from './settings';
import { CIENTIFICO, SKILL } from './skills';
import { parseSuggestions } from './suggestions';
import type { CheckRequest, CheckResult, Settings } from './types';

const inflight = new Map<string, AbortController>();

export async function handleCheck(
  req: CheckRequest,
  deps: { fetch: typeof fetch; settings: Settings },
): Promise<CheckResult> {
  const { settings } = deps;
  const provider = settings.provider;
  if (!settings.keys[provider]) return { ok: false, error: 'no-key' };

  inflight.get(req.key)?.abort();
  const controller = new AbortController();
  inflight.set(req.key, controller);

  try {
    const system = buildSystemPrompt(req.level, SKILL, CIENTIFICO);
    const { url, init } = buildRequest(provider, settings, system, req.text);
    let res: Response;
    try {
      res = await deps.fetch(url, { ...init, signal: controller.signal });
    } catch {
      return controller.signal.aborted
        ? { ok: true, suggestions: [] }
        : { ok: false, error: 'network' };
    }
    if (controller.signal.aborted) return { ok: true, suggestions: [] };
    if (!res.ok) return { ok: false, error: classifyError(res.status) };
    let json: unknown;
    try {
      json = await res.json();
    } catch {
      return controller.signal.aborted
        ? { ok: true, suggestions: [] }
        : { ok: false, error: 'provider' };
    }
    if (controller.signal.aborted) return { ok: true, suggestions: [] };
    return { ok: true, suggestions: parseSuggestions(readText(provider, json)) };
  } finally {
    if (inflight.get(req.key) === controller) inflight.delete(req.key);
  }
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
