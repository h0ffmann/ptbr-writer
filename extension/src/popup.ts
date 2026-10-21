import { normalizeHost } from './hosts';
import { mountProviderFields } from './provider-fields';
import { loadSettings } from './settings';
import { LEVELS, disableSiteFlow, enableSiteFlow, saveProviderFields, setEnabled, setLevel } from './sites';
import type { Level, ProviderId } from './types';

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

const PROVIDER_NAMES: Record<ProviderId, string> = { claude: 'Claude', deepseek: 'DeepSeek' };

function button(id: string, text: string, onClick: () => void): HTMLButtonElement {
  const b = document.createElement('button');
  b.id = id;
  b.type = 'button';
  b.textContent = text;
  b.addEventListener('click', onClick);
  return b;
}

function section(root: HTMLElement): HTMLElement {
  const div = document.createElement('section');
  root.append(div);
  return div;
}

// The small settings screen: the main switch, the current site (level, on/off), then provider,
// model and key. Provider, model, level and the switch save at once; the key only on Salvar, so a
// half-pasted key is never stored. The full page holds the list of every site.
export async function render(root: HTMLElement): Promise<void> {
  root.replaceChildren();
  const line = (parent: HTMLElement, text: string) => {
    const p = document.createElement('p');
    p.textContent = text;
    parent.append(p);
  };
  const s = await loadSettings();

  const head = section(root);
  const enabled = Object.assign(document.createElement('input'), { id: 'enabled', type: 'checkbox', checked: s.enabled });
  enabled.addEventListener('change', () => void setEnabled(enabled.checked));
  const title = document.createElement('label');
  title.append(enabled, ' ptbr-writer ligado');
  head.append(title);

  const site = section(root);
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  const host = tab?.url ? hostOf(tab.url) : null;
  if (!tab?.url) line(site, 'Abra um site https para ativar a extensão.');
  else if (!host) line(site, 'Site não suportado.');
  else {
    const cfg = s.sites[host];
    const on = !!cfg?.enabled;
    line(site, host);
    if (cfg) {
      const level = document.createElement('select');
      level.id = 'site-level';
      level.append(...LEVELS.map((l) => Object.assign(document.createElement('option'), { value: l, textContent: l })));
      level.value = cfg.level;
      level.addEventListener('change', () => void setLevel(host, level.value as Level));
      const label = document.createElement('label');
      label.append('Nível ', level);
      site.append(label);
    }
    site.append(
      button('toggle-site', on ? 'Desativar neste site' : 'Ativar neste site', async () => {
        if (on) await disableSiteFlow(host);
        else await enableSiteFlow(host);
        await render(root);
      }),
    );
  }

  const provider = section(root);
  if (!s.keys[s.provider]) line(provider, `Sem chave do ${PROVIDER_NAMES[s.provider]}.`);
  const fields = mountProviderFields(provider, s, (f) => {
    void saveProviderFields({ provider: f.provider.value as ProviderId, model: f.model.value, key: '', enabled: enabled.checked });
  });
  provider.append(
    button('save-key', 'Salvar', async () => {
      await saveProviderFields({
        provider: fields.provider.value as ProviderId,
        model: fields.model.value,
        key: fields.key.value,
        enabled: enabled.checked,
      });
      await render(root);
    }),
  );

  root.append(button('open-options', 'Todas as configurações', () => void chrome.runtime.openOptionsPage()));
}

const app = typeof document !== 'undefined' ? document.getElementById('app') : null;
if (app && typeof chrome !== 'undefined' && chrome.tabs) void render(app);
