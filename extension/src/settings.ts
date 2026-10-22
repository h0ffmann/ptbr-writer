import type { Level, ProviderId, Settings } from './types';

const STORAGE_KEY = 'settings';

// Ollama and Ollaya run on this machine: no key, any model they have installed.
export const isLocal = (p: ProviderId): p is 'ollama' | 'ollaya' => p === 'ollama' || p === 'ollaya';

export const DEFAULT_SETTINGS: Settings = {
  enabled: true,
  provider: 'claude',
  keys: { claude: '', deepseek: '', ollama: '', ollaya: '' }, // the local providers take no key
  models: {
    claude: 'claude-haiku-5-5',
    // Checked 2026-10-21 at https://api-docs.deepseek.com (and /quick_start/pricing): the docs list
    // `deepseek-flash` and `deepseek-v4-pro` and no longer mention `deepseek-chat`.
    // ⚠ Read through a page summarizer, not by hand; confirm in the browser before a release.
    deepseek: 'deepseek-flash',
    // The model of just opencode-qwen (Qwen3.6-27B), from Ollama's library; just ext-ollama pulls it.
    ollama: 'qwen3.6:27b',
    // A decision model whose card lists Portuguese, runs on a CPU and is calibrated: the pick of
    // docs/SURVEY_DECISION_MODEL.md (`decima` is `decima:base`). ⚠ Not measured on pt-BR rules.
    ollaya: 'decima',
  },
  ollayaStyle: false,
  sound: false,
  sites: {
    'web.whatsapp.com': { enabled: true, level: 'leve' },
    'mail.google.com': { enabled: true, level: 'descritivo' },
    'github.com': { enabled: true, level: 'descritivo' },
  },
};

// The models the options page offers; the first of each list is the default.
export const MODELS: Readonly<Record<ProviderId, readonly string[]>> = {
  claude: ['claude-haiku-5-5', 'claude-sonnet-5-5', 'claude-opus-5-5'],
  deepseek: ['deepseek-flash', 'deepseek-v4-pro'],
  ollama: ['qwen3.6:27b', 'qwen3:14b', 'qwen3:8b', 'qwen3:4b'], // suggestions; the model list also shows what Ollama has installed
  ollaya: ['decima', 'laya:multilingual', 'decima:small', 'pplx-decider', 'winnow:e4b'], // the same, for Ollaya; the last two want a GPU
};

const knownModel = (p: ProviderId, m: string | undefined) =>
  m && (isLocal(p) || MODELS[p].includes(m)) ? m : DEFAULT_SETTINGS.models[p];

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
    models: {
      claude: knownModel('claude', stored.models?.claude),
      deepseek: knownModel('deepseek', stored.models?.deepseek),
      ollama: knownModel('ollama', stored.models?.ollama),
      ollaya: knownModel('ollaya', stored.models?.ollaya),
    },
    sites: { ...d.sites, ...stored.sites },
  };
}

export async function saveSettings(s: Settings): Promise<void> {
  await chrome.storage.local.set({ [STORAGE_KEY]: s });
}
