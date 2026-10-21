import { normalizeHost } from './hosts';
import { mountProviderFields } from './provider-fields';
import { NEVER_FIX_MAX_WORDS, addNeverFix } from './never-fix';
import { loadSettings } from './settings';
import { mountTextFields } from './text-fields';
import {
  DEFAULT_SITE_HOSTS,
  LEVELS,
  disableSiteFlow,
  enableSiteFlow,
  removeSiteFlow,
  saveProviderFields,
  saveTextSettings,
  setLevel,
} from './sites';
import type { Level, ProviderId } from './types';

const el = <T extends HTMLElement>(root: ParentNode, sel: string) => root.querySelector(sel) as T;

function button(text: string, onClick: () => void): HTMLButtonElement {
  const b = document.createElement('button');
  b.type = 'button';
  b.textContent = text;
  b.addEventListener('click', onClick);
  return b;
}

export async function render(root: HTMLElement): Promise<void> {
  const s = await loadSettings();
  const field = (label: string, input: string) => `<label>${label} ${input}</label>`;
  root.innerHTML = `
    <h1>ptbr-writer</h1>
    <p>${field('<input id="enabled" type="checkbox">', 'Ativado')}</p>
    <div id="provider-fields"></div>
    <p><button id="save" type="button">Salvar</button> <span id="status" role="status"></span></p>
    <div id="text-fields"></div>
    <h2>Nunca corrigir</h2>
    <ul id="never-fix"></ul>
    <p><input id="new-never-fix" type="text" placeholder="até 3 palavras"> <button id="add-never-fix" type="button">Adicionar</button>
    <span id="never-fix-error" role="alert"></span></p>
    <h2>Sites</h2>
    <table id="sites"><thead><tr><th>Site</th><th>Nível</th><th>Ativo</th><th></th></tr></thead><tbody></tbody></table>
    <p><input id="new-host" type="text" placeholder="exemplo.com"> <button id="add-site" type="button">Adicionar site</button>
    <span id="site-error" role="alert"></span></p>`;
  el<HTMLInputElement>(root, '#enabled').checked = s.enabled;
  const { provider, model, key } = mountProviderFields(el(root, '#provider-fields'), s);
  mountTextFields(el(root, '#text-fields'), s);
  const list = el(root, '#never-fix');
  for (const entry of s.neverFix) {
    const li = document.createElement('li');
    li.append(entry + ' ', button('Remover', async () => {
      const cur = (await loadSettings()).neverFix;
      await saveTextSettings({ neverFix: cur.filter((e) => e !== entry) });
      await render(root);
    }));
    list.append(li);
  }
  el<HTMLButtonElement>(root, '#add-never-fix').addEventListener('click', async () => {
    const typed = el<HTMLInputElement>(root, '#new-never-fix').value.trim();
    if (!typed || typed.split(/\s+/).length > NEVER_FIX_MAX_WORDS) {
      el(root, '#never-fix-error').textContent = 'Use de 1 a 3 palavras.';
      return;
    }
    // Reloaded here: another tab may have added entries since this page was drawn.
    const cur = (await loadSettings()).neverFix;
    await saveTextSettings({ neverFix: addNeverFix(cur, typed) });
    await render(root);
  });
  const body = el<HTMLTableSectionElement>(root, '#sites tbody');
  for (const [host, cfg] of Object.entries(s.sites)) {
    const row = body.insertRow();
    row.dataset.host = host;
    row.insertCell().textContent = host;
    const select = document.createElement('select');
    for (const l of LEVELS) { const o = document.createElement('option'); o.value = l; o.textContent = l; select.append(o); };
    select.value = cfg.level;
    select.addEventListener('change', () => void setLevel(host, select.value as Level));
    row.insertCell().append(select);
    const toggle = document.createElement('input');
    toggle.type = 'checkbox';
    toggle.checked = cfg.enabled;
    toggle.addEventListener('change', async () => {
      if (toggle.checked) await enableSiteFlow(host);
      else await disableSiteFlow(host);
      await render(root);
    });
    row.insertCell().append(toggle);
    const last = row.insertCell();
    if (!DEFAULT_SITE_HOSTS.has(host)) {
      last.append(button('Remover', async () => { await removeSiteFlow(host); await render(root); }));
    }
  }
  el<HTMLButtonElement>(root, '#save').addEventListener('click', async () => {
    await saveProviderFields({
      provider: provider.value as ProviderId,
      enabled: el<HTMLInputElement>(root, '#enabled').checked,
      key: key.value,
      model: model.value,
    });
    await render(root);
    el(root, '#status').textContent = 'Salvo.';
  });
  el<HTMLButtonElement>(root, '#add-site').addEventListener('click', async () => {
    const err = el(root, '#site-error');
    const host = normalizeHost(el<HTMLInputElement>(root, '#new-host').value);
    if (!host) {
      err.textContent = 'Site inválido. Use um domínio como exemplo.com.';
      return;
    }
    if (!(await enableSiteFlow(host))) {
      err.textContent = 'Permissão negada.';
      return;
    }
    await render(root);
  });
}

const app = typeof document !== 'undefined' ? document.getElementById('app') : null;
if (app && typeof chrome !== 'undefined' && chrome.storage) void render(app);
