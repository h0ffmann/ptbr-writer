import type { Level, ProviderId, Settings, Tone } from './types';

const STORAGE_KEY = 'settings';

export const DEFAULT_SETTINGS: Settings = {
  enabled: true,
  provider: 'claude',
  keys: { claude: '', deepseek: '' },
  models: {
    claude: 'claude-haiku-5-5',
    // Checked 2026-10-21 at https://api-docs.deepseek.com (and /quick_start/pricing): the docs list
    // `deepseek-flash` and `deepseek-v4-pro` and no longer mention `deepseek-chat`.
    // ⚠ Read through a page summarizer, not by hand; confirm in the browser before a release.
    deepseek: 'deepseek-flash',
  },
  sites: {
    'web.whatsapp.com': { enabled: true, level: 'leve' },
    'mail.google.com': { enabled: true, level: 'descritivo' },
    'github.com': { enabled: true, level: 'descritivo' },
  },
  tone: 'neutro',
  rewrite: true,
  neverFix: [],
  delayMs: 1000,
};

// The models the options page offers; the first of each list is the default.
export const MODELS: Readonly<Record<ProviderId, readonly string[]>> = {
  claude: ['claude-haiku-5-5', 'claude-sonnet-5-5', 'claude-opus-5-5'],
  deepseek: ['deepseek-flash', 'deepseek-v4-pro'],
};

export const TONES: readonly Tone[] = ['informal', 'neutro', 'formal'];
export const DELAYS = [1000, 2000, 3000] as const;

const knownModel = (p: ProviderId, m: string | undefined) =>
  m && MODELS[p].includes(m) ? m : DEFAULT_SETTINGS.models[p];

export function levelFor(host: string, s: Settings): Level | null {
  if (!s.enabled) return null;
  const site = s.sites[host];
  if (!site || !site.enabled) return null;
  return site.level;
}

export async function loadSettings(): Promise<Settings> {
  const got = await chrome.storage.local.get(STORAGE_KEY);
  const stored = (got[STORAGE_KEY] ?? {}) as Partial<Settings>;
  const d = DEFAULT_SETTINGS;
  return {
    ...d,
    ...stored,
    keys: { ...d.keys, ...stored.keys },
    models: { claude: knownModel('claude', stored.models?.claude), deepseek: knownModel('deepseek', stored.models?.deepseek) },
    sites: { ...d.sites, ...stored.sites },
    tone: TONES.includes(stored.tone as Tone) ? (stored.tone as Tone) : d.tone,
    rewrite: typeof stored.rewrite === 'boolean' ? stored.rewrite : d.rewrite,
    neverFix: Array.isArray(stored.neverFix) ? stored.neverFix.filter((e) => typeof e === 'string') : d.neverFix,
    delayMs: DELAYS.includes(stored.delayMs as (typeof DELAYS)[number]) ? stored.delayMs! : d.delayMs,
  };
}

export async function saveSettings(s: Settings): Promise<void> {
  await chrome.storage.local.set({ [STORAGE_KEY]: s });
}
