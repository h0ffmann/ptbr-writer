import { DELAYS, TONES } from './settings';
import { saveTextSettings } from './sites';
import type { Settings, Tone } from './types';

function select(id: string, options: [string, string][], value: string): HTMLSelectElement {
  const sel = document.createElement('select');
  sel.id = id;
  for (const [v, text] of options) {
    const o = document.createElement('option');
    o.value = v;
    o.textContent = text;
    sel.append(o);
  }
  sel.value = value;
  return sel;
}

function labelled(text: string, sel: HTMLSelectElement): HTMLLabelElement {
  const label = document.createElement('label');
  label.append(text + ' ', sel);
  return label;
}

// The tone, rewrite and wait selects, shared by the popup and the options page. Each saves at once.
export function mountTextFields(parent: HTMLElement, s: Settings): void {
  const tone = select('tone', TONES.map((t) => [t, t]), s.tone);
  const rewrite = select('rewrite', [['false', 'Só erros'], ['true', 'Erros e reescrita']], String(s.rewrite));
  const delay = select('delay', DELAYS.map((d) => [String(d), `${d / 1000} s`]), String(s.delayMs));
  tone.addEventListener('change', () => void saveTextSettings({ tone: tone.value as Tone }));
  rewrite.addEventListener('change', () => void saveTextSettings({ rewrite: rewrite.value === 'true' }));
  delay.addEventListener('change', () => void saveTextSettings({ delayMs: Number(delay.value) as Settings['delayMs'] }));
  parent.append(labelled('Tom', tone), labelled('Correção', rewrite), labelled('Espera', delay));
}
