import { OLLAMA_ORIGINS, ollamaModels, ollamaPull } from './providers';
import { MODELS } from './settings';
import { maskKey } from './sites';
import type { ProviderId, Settings } from './types';

export interface ProviderFields {
  provider: HTMLSelectElement;
  model: HTMLSelectElement;
  key: HTMLInputElement;
}

const PROVIDERS: [ProviderId, string][] = [
  ['claude', 'Claude'],
  ['deepseek', 'DeepSeek'],
  ['ollama', 'Ollama + Qwen (local)'],
];

function option(value: string, text: string): HTMLOptionElement {
  return Object.assign(document.createElement('option'), { value, textContent: text });
}

function labeled(text: string, input: HTMLElement): HTMLLabelElement {
  const label = document.createElement('label');
  label.append(`${text} `, input);
  return label;
}

// Provider, model and key, shared by the popup and the options page. The model list and the key
// follow the chosen provider; the other provider's key stays stored. `s` is updated in place so a
// provider switched back shows what was just chosen.
export function mountProviderFields(
  container: HTMLElement,
  s: Settings,
  onChange: (f: ProviderFields) => void = () => {},
): ProviderFields {
  const provider = document.createElement('select');
  provider.id = 'provider';
  provider.append(...PROVIDERS.map(([id, name]) => option(id, name)));
  const model = document.createElement('select');
  model.id = 'model';
  const key = document.createElement('input');
  key.id = 'key';
  key.type = 'password';
  key.autocomplete = 'off';
  const fields = { provider, model, key };

  // Ollama's list: the suggested models (MODELS.ollama, the first one suggested) and the ones
  // already installed; one not installed is downloaded with Baixar.
  const pull = Object.assign(document.createElement('button'), { id: 'pull-model', type: 'button', textContent: 'Baixar' });
  const status = Object.assign(document.createElement('span'), { id: 'pull-status' });
  status.setAttribute('role', 'status');
  let installed: string[] = [];
  const models = (p: ProviderId) => (p === 'ollama' ? [...new Set([...MODELS.ollama, ...installed])] : MODELS[p]);
  const label = (p: ProviderId, m: string) =>
    p !== 'ollama'
      ? m
      : m + (m === MODELS.ollama[0] ? ' (sugerido)' : '') + (installed.includes(m) ? '' : ' (baixar)');
  const fill = (p: ProviderId) => {
    model.replaceChildren(...models(p).map((m) => option(m, label(p, m))));
    model.value = s.models[p];
    pull.hidden = p !== 'ollama' || installed.includes(model.value);
  };
  const refresh = async () => {
    installed = await ollamaModels();
    if (provider.value === 'ollama') fill('ollama');
  };

  const show = (p: ProviderId) => {
    key.value = maskKey(s.keys[p]);
    fill(p);
    if (p === 'ollama') void refresh();
    // Ollama runs on this machine and takes no key.
    key.disabled = p === 'ollama';
    key.placeholder = p === 'ollama' ? 'sem chave: just ext-ollama' : '';
  };
  provider.value = s.provider;
  show(s.provider);
  provider.addEventListener('change', async () => {
    const p = provider.value as ProviderId;
    // Asked in the click that picks Ollama: the browser shows its prompt only during a user gesture.
    if (p === 'ollama' && !(await chrome.permissions.request(OLLAMA_ORIGINS))) {
      provider.value = s.provider;
      return;
    }
    s.provider = p;
    show(s.provider);
    onChange(fields);
  });
  model.addEventListener('change', () => {
    s.models[provider.value as ProviderId] = model.value;
    pull.hidden = provider.value !== 'ollama' || installed.includes(model.value);
    onChange(fields);
  });
  pull.addEventListener('click', async () => {
    pull.disabled = true;
    status.textContent = `baixando ${model.value}…`;
    const ok = await ollamaPull(model.value);
    if (ok) await refresh();
    status.textContent = ok ? 'pronto' : 'falhou: o Ollama está no ar (just ext-ollama)?';
    pull.disabled = false;
  });
  const pullRow = document.createElement('p');
  pullRow.append(pull, ' ', status);
  container.append(labeled('Provedor', provider), labeled('Modelo', model), pullRow, labeled('Chave', key));
  return fields;
}
