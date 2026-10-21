import { MODELS, loadSettings, saveSettings } from './settings';
import type { Level, Settings } from './types';

export const DEFAULT_SITE_HOSTS: ReadonlySet<string> = new Set(['web.whatsapp.com', 'mail.google.com', 'github.com']);
export const LEVELS: readonly Level[] = ['estrito', 'descritivo', 'leve', 'científico'];
export const NEW_SITE_LEVEL: Level = 'descritivo';
const MASK = '••••';

const origins = (host: string) => ({ origins: [`https://${host}/*`] });
const notify = (type: 'enable-site' | 'disable-site', host: string) => chrome.runtime.sendMessage({ type, host });

export function maskKey(key: string): string {
  return key ? MASK + key.slice(-4) : '';
}

/** What to store after a key field is saved: empty or still-masked input keeps the stored key. */
export function resolveKey(stored: string, typed: string): string {
  const value = typed.trim();
  if (!value || value.startsWith(MASK)) return stored;
  return value;
}

export async function enableSiteFlow(host: string): Promise<boolean> {
  const isDefault = DEFAULT_SITE_HOSTS.has(host);
  if (!isDefault && !(await chrome.permissions.request(origins(host)))) return false;
  const s = await loadSettings();
  const level = s.sites[host]?.level ?? NEW_SITE_LEVEL;
  await saveSettings({ ...s, sites: { ...s.sites, [host]: { enabled: true, level } } });
  if (!isDefault) await notify('enable-site', host);
  return true;
}

export async function disableSiteFlow(host: string): Promise<void> {
  const isDefault = DEFAULT_SITE_HOSTS.has(host);
  const s = await loadSettings();
  const level = s.sites[host]?.level ?? NEW_SITE_LEVEL;
  await saveSettings({ ...s, sites: { ...s.sites, [host]: { enabled: false, level } } });
  if (isDefault) return;
  await chrome.permissions.remove(origins(host));
  await notify('disable-site', host);
}

export async function removeSiteFlow(host: string): Promise<void> {
  if (DEFAULT_SITE_HOSTS.has(host)) return;
  const s = await loadSettings();
  const sites = { ...s.sites };
  delete sites[host];
  await saveSettings({ ...s, sites });
  await chrome.permissions.remove(origins(host));
  await notify('disable-site', host);
}

export async function setLevel(host: string, level: Level): Promise<void> {
  const s = await loadSettings();
  const site = s.sites[host];
  if (!site) return;
  await saveSettings({ ...s, sites: { ...s.sites, [host]: { ...site, level } } });
}

export async function setEnabled(enabled: boolean): Promise<void> {
  const s = await loadSettings();
  await saveSettings({ ...s, enabled });
}

/** Saves the chosen provider with its key and model; the other provider's key and model stay. */
export async function saveProviderFields(f: {
  provider: Settings['provider'];
  key: string;
  model: string;
  enabled: boolean;
}): Promise<void> {
  const s = await loadSettings();
  const p = f.provider;
  await saveSettings({
    ...s,
    enabled: f.enabled,
    provider: p,
    keys: { ...s.keys, [p]: resolveKey(s.keys[p], f.key) },
    models: { ...s.models, [p]: MODELS[p].includes(f.model) ? f.model : s.models[p] },
  });
}
