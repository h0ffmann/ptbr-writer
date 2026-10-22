import { LOCAL_ORIGINS, localModels, localPull } from './providers';
import { MODELS, isLocal } from './settings';
import { maskKey, setOllayaStyle, setSound } from './sites';
import type { ProviderId, Settings } from './types';

export interface ProviderFields {
  provider: HTMLSelectElement;
  model: HTMLSelectElement;
  key: HTMLInputElement;
}

const PROVIDERS: [ProviderId, string][] = [
  ['claude', 'Claude'],
  ['deepseek', 'DeepSeek'],
  ['ollama', 'Ollama (local)'],
  ['ollaya', 'Ollaya (local, só aponta)'],
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

  // A local server's list (Ollama, Ollaya): the suggested models (MODELS[p], the first one suggested)
  // and the ones already installed; one not installed is downloaded with Baixar.
  const pull = Object.assign(document.createElement('button'), { id: 'pull-model', type: 'button', textContent: 'Baixar' });
  const status = Object.assign(document.createElement('span'), { id: 'pull-status' });
  status.setAttribute('role', 'status');
  let installed: string[] = [];
  const models = (p: ProviderId) => (isLocal(p) ? [...new Set([...MODELS[p], ...installed])] : MODELS[p]);
  const label = (p: ProviderId, m: string) =>
    !isLocal(p)
      ? m
      : m + (m === MODELS[p][0] ? ' (sugerido)' : '') + (installed.includes(m) ? '' : ' (baixar)');
  const fill = (p: ProviderId) => {
    model.replaceChildren(...models(p).map((m) => option(m, label(p, m))));
    model.value = s.models[p];
    pull.hidden = !isLocal(p) || installed.includes(model.value);
  };
  const refresh = async () => {
    const p = provider.value as ProviderId;
    if (!isLocal(p)) return;
    const got = await localModels(p);
    // A switch while the list was loading: this list belongs to the other server.
    if (provider.value !== p) return;
    installed = got;
    fill(p);
  };

  // Ollaya + Ollama: a feature flag beside Ollama. On, Ollaya picks the style and Ollama writes in it;
  // Ollaya down, Ollama writes alone. Saved on the click, like the on/off switch.
  const style = Object.assign(document.createElement('input'), { id: 'ollaya-style', type: 'checkbox', checked: s.ollayaStyle });
  const styleRow = labeled('Ollaya escolhe o estilo (conciso, socrático, amigável)', style);
  style.addEventListener('change', async () => {
    // The permission for Ollaya, asked in this click as for the provider itself.
    if (style.checked && !(await chrome.permissions.request(LOCAL_ORIGINS.ollaya))) style.checked = false;
    s.ollayaStyle = style.checked;
    await setOllayaStyle(style.checked);
  });

  const show = (p: ProviderId) => {
    styleRow.hidden = p !== 'ollama';
    key.value = maskKey(s.keys[p]);
    // The other server's models are not this one's.
    installed = [];
    fill(p);
    if (isLocal(p)) void refresh();
    // Ollama and Ollaya run on this machine and take no key.
    key.disabled = isLocal(p);
    key.placeholder = isLocal(p) ? `sem chave: just ext-${p}` : '';
  };
  provider.value = s.provider;
  show(s.provider);
  provider.addEventListener('change', async () => {
    const p = provider.value as ProviderId;
    // Asked in the click that picks a local server: the browser shows its prompt only during a user gesture.
    if (isLocal(p) && !(await chrome.permissions.request(LOCAL_ORIGINS[p]))) {
      provider.value = s.provider;
      return;
    }
    s.provider = p;
    show(s.provider);
    onChange(fields);
  });
  model.addEventListener('change', () => {
    s.models[provider.value as ProviderId] = model.value;
    pull.hidden = !isLocal(provider.value as ProviderId) || installed.includes(model.value);
    onChange(fields);
  });
  pull.addEventListener('click', async () => {
    pull.disabled = true;
    status.textContent = `baixando ${model.value}…`;
    const p = provider.value as 'ollama' | 'ollaya';
    const ok = await localPull(p, model.value);
    if (ok) await refresh();
    status.textContent = ok ? 'pronto' : `falhou: o ${p === 'ollama' ? 'Ollama' : 'Ollaya'} está no ar (just ext-${p})?`;
    pull.disabled = false;
  });
  const pullRow = document.createElement('p');
  pullRow.append(pull, ' ', status);
  // Sound, off by default: a cue on each event and 🔊 on the card, with the system's own pt-BR voice.
  // No server. Saved on the click, like the on/off switch.
  const sound = Object.assign(document.createElement('input'), { id: 'sound', type: 'checkbox', checked: s.sound });
  sound.addEventListener('change', async () => {
    s.sound = sound.checked;
    await setSound(sound.checked);
  });
  const soundRow = labeled('🔊 Som: efeitos e ler em voz alta (voz do sistema, sem servidor)', sound);
  container.append(labeled('Provedor', provider), labeled('Modelo', model), pullRow, styleRow, labeled('Chave', key), soundRow);
  return fields;
}
