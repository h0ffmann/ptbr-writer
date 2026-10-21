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

  const show = (p: ProviderId) => {
    key.value = maskKey(s.keys[p]);
    model.replaceChildren(...MODELS[p].map((m) => option(m, m)));
    model.value = s.models[p];
  };
  provider.value = s.provider;
  show(s.provider);
  provider.addEventListener('change', () => {
    s.provider = provider.value as ProviderId;
    show(s.provider);
    onChange(fields);
  });
  model.addEventListener('change', () => {
    s.models[provider.value as ProviderId] = model.value;
    onChange(fields);
  });
  container.append(labeled('Provedor', provider), labeled('Modelo', model), labeled('Chave', key));
  return fields;
}
