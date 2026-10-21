import type { Level, Settings } from './types';

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
};

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
    models: { ...d.models, ...stored.models },
    sites: { ...d.sites, ...stored.sites },
  };
}

export async function saveSettings(s: Settings): Promise<void> {
  await chrome.storage.local.set({ [STORAGE_KEY]: s });
}
