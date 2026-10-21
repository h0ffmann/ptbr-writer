import { normalizeHost } from './hosts';
import { loadSettings } from './settings';
import { disableSiteFlow, enableSiteFlow } from './sites';

// The host the extension can run on: an https page whose host the options page would accept
// (no localhost, no IP address).
export function hostOf(url: string | undefined): string | null {
  if (!url) return null;
  try {
    const u = new URL(url);
    return u.protocol === 'https:' ? normalizeHost(u.hostname) : null;
  } catch {
    return null;
  }
}

export async function render(root: HTMLElement): Promise<void> {
  root.replaceChildren();
  const line = (text: string) => {
    const p = document.createElement('p');
    p.textContent = text;
    root.append(p);
  };
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  if (!tab?.url) return line('Site sem acesso.');
  const host = hostOf(tab.url);
  if (!host) return line('Site não suportado.');
  const s = await loadSettings();
  const site = s.sites[host];
  const on = !!site?.enabled;
  line(host);
  line(on ? `Ativo, nível ${site.level}` : 'Desativado');
  const b = document.createElement('button');
  b.id = 'toggle-site';
  b.type = 'button';
  b.textContent = on ? 'Desativar neste site' : 'Ativar neste site';
  b.addEventListener('click', async () => {
    if (on) await disableSiteFlow(host);
    else await enableSiteFlow(host);
    await render(root);
  });
  root.append(b);
}

const app = typeof document !== 'undefined' ? document.getElementById('app') : null;
if (app && typeof chrome !== 'undefined' && chrome.tabs) void render(app);
