import { afterEach, describe, expect, it } from 'vitest';
import { DEFAULT_SETTINGS, MODELS, levelFor, loadSettings, saveSettings } from '../src/settings';
import type { Settings } from '../src/types';

function stub(initial: Record<string, unknown>) {
  const store: Record<string, unknown> = { ...initial };
  (globalThis as { chrome?: unknown }).chrome = {
    storage: {
      local: {
        get: async (k: string) => (k in store ? { [k]: store[k] } : {}),
        set: async (o: Record<string, unknown>) => void Object.assign(store, o),
      },
    },
  };
  return store;
}

afterEach(() => {
  delete (globalThis as { chrome?: unknown }).chrome;
});

describe('levelFor', () => {
  it('default sites', () => {
    expect(levelFor('web.whatsapp.com', DEFAULT_SETTINGS)).toBe('leve');
    expect(levelFor('mail.google.com', DEFAULT_SETTINGS)).toBe('descritivo');
    expect(levelFor('github.com', DEFAULT_SETTINGS)).toBe('descritivo');
    expect(levelFor('bank.example', DEFAULT_SETTINGS)).toBeNull();
  });
  it('disabled extension', () => {
    expect(levelFor('github.com', { ...DEFAULT_SETTINGS, enabled: false })).toBeNull();
  });
  it('disabled site', () => {
    const s: Settings = { ...DEFAULT_SETTINGS, sites: { 'github.com': { enabled: false, level: 'leve' } } };
    expect(levelFor('github.com', s)).toBeNull();
  });
  it('allowed site', () => {
    const s: Settings = {
      ...DEFAULT_SETTINGS,
      sites: { ...DEFAULT_SETTINGS.sites, 'bank.example': { enabled: true, level: 'descritivo' } },
    };
    expect(levelFor('bank.example', s)).toBe('descritivo');
  });
});

describe('defaults', () => {
  it('values', () => {
    expect(DEFAULT_SETTINGS.enabled).toBe(true);
    expect(DEFAULT_SETTINGS.provider).toBe('claude');
    expect(DEFAULT_SETTINGS.keys).toEqual({ claude: '', deepseek: '' });
    expect(DEFAULT_SETTINGS.models.claude).toBe('claude-haiku-5-5');
  });
});

describe('loadSettings', () => {
  it('empty storage gives defaults', async () => {
    stub({});
    expect(await loadSettings()).toEqual(DEFAULT_SETTINGS);
  });
  it('keeps default keys and models missing from storage', async () => {
    stub({ settings: { keys: { claude: 'sk-test' }, models: { deepseek: 'deepseek-v4-pro' } } });
    const s = await loadSettings();
    expect(s.keys).toEqual({ claude: 'sk-test', deepseek: '' });
    expect(s.models).toEqual({ claude: DEFAULT_SETTINGS.models.claude, deepseek: 'deepseek-v4-pro' });
  });
  it('a stored model outside the list falls back to the provider default', async () => {
    stub({ settings: { models: { claude: 'claude-typo', deepseek: 'deepseek-chat' } } });
    expect((await loadSettings()).models).toEqual(DEFAULT_SETTINGS.models);
  });
  it('every default model is in its provider list', () => {
    expect(MODELS.claude).toEqual(['claude-haiku-5-5', 'claude-sonnet-5-5', 'claude-opus-5-5']);
    expect(MODELS.deepseek).toEqual(['deepseek-flash', 'deepseek-v4-pro']);
    expect(MODELS.claude[0]).toBe(DEFAULT_SETTINGS.models.claude);
    expect(MODELS.deepseek[0]).toBe(DEFAULT_SETTINGS.models.deepseek);
  });
  it('keeps the default sites when storage has one host', async () => {
    stub({ settings: { sites: { 'bank.example': { enabled: true, level: 'estrito' } } } });
    const s = await loadSettings();
    expect(Object.keys(s.sites).sort()).toEqual(
      ['bank.example', 'github.com', 'mail.google.com', 'web.whatsapp.com'].sort(),
    );
    expect(levelFor('bank.example', s)).toBe('estrito');
  });
  it('stored top-level values win', async () => {
    stub({ settings: { enabled: false, provider: 'deepseek' } });
    const s = await loadSettings();
    expect(s.enabled).toBe(false);
    expect(s.provider).toBe('deepseek');
  });
  it('saveSettings round-trips', async () => {
    stub({});
    const s: Settings = { ...DEFAULT_SETTINGS, provider: 'deepseek' };
    await saveSettings(s);
    expect(await loadSettings()).toEqual(s);
  });
});
