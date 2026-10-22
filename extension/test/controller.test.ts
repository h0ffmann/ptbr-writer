// @vitest-environment happy-dom
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { startContent } from '../src/content/controller';
import { playCue } from '../src/content/sound';
import type { CheckRequest, Level } from '../src/types';

vi.mock('../src/content/sound', () => ({ playCue: vi.fn(), speak: vi.fn(() => true) }));

type Listener = (changes: Record<string, unknown>, area: string) => void;
let store: Record<string, unknown>;
let listeners: Listener[];
let dispose: (() => void) | undefined;
const send = vi.fn(async (_r: CheckRequest) => ({ ok: true, suggestions: [] }));

function site(enabled: boolean, level: Level = 'descritivo') {
  store.settings = { sites: { [location.hostname]: { enabled, level } } };
}
async function change(enabled: boolean, level?: Level) {
  site(enabled, level);
  for (const l of listeners) l({ settings: {} }, 'local');
  await vi.advanceTimersByTimeAsync(0);
}

beforeEach(() => {
  vi.useFakeTimers();
  store = {};
  listeners = [];
  send.mockClear();
  (globalThis as any).chrome = {
    storage: {
      local: { get: async () => store },
      onChanged: {
        addListener: (l: Listener) => listeners.push(l),
        removeListener: (l: Listener) => (listeners = listeners.filter((x) => x !== l)),
      },
    },
    runtime: { sendMessage: send },
  };
  document.body.innerHTML = '';
});
afterEach(() => {
  dispose?.();
  dispose = undefined;
  vi.useRealTimers();
  delete (globalThis as any).chrome;
});

function textarea(): HTMLTextAreaElement {
  const el = document.createElement('textarea');
  document.body.appendChild(el);
  return el;
}
const type = (el: HTMLTextAreaElement, v: string) => {
  el.value = v;
  el.dispatchEvent(new Event('input', { bubbles: true }));
};

describe('startContent', () => {
  it('sound on: a cue when suggestions appear and when an error shows; off: silence', async () => {
    const sug = { original: 'dois', replacement: '2', rule: 'PTW-7', severity: 'aviso', occurrence: 1 };
    const cues = vi.mocked(playCue);
    cues.mockClear();
    store.settings = { sound: true, sites: { [location.hostname]: { enabled: true, level: 'descritivo' } } };
    send.mockResolvedValueOnce({ ok: true, suggestions: [sug] } as any);
    dispose = await startContent();
    const el = textarea();
    el.focus();
    type(el, 'um dois três quatro');
    await vi.advanceTimersByTimeAsync(1000);
    expect(cues).toHaveBeenCalledWith('found');
    send.mockResolvedValueOnce({ ok: false, error: 'no-key' } as any);
    type(el, 'um dois três quatro cinco');
    await vi.advanceTimersByTimeAsync(1000);
    expect(cues).toHaveBeenCalledWith('error');
    dispose();
    dispose = undefined;

    cues.mockClear();
    site(true);
    send.mockResolvedValueOnce({ ok: true, suggestions: [sug] } as any);
    dispose = await startContent();
    const quiet = textarea();
    quiet.focus();
    type(quiet, 'um dois três quatro');
    await vi.advanceTimersByTimeAsync(1000);
    expect(send).toHaveBeenCalled();
    expect(cues).not.toHaveBeenCalled();
  });

  it('stops sending once the site is switched off', async () => {
    site(true);
    dispose = await startContent();
    const el = textarea();
    el.focus();
    type(el, 'um dois três quatro');
    await vi.advanceTimersByTimeAsync(1000);
    expect(send).toHaveBeenCalledTimes(1);
    await change(false);
    type(el, 'um dois três quatro cinco');
    await vi.advanceTimersByTimeAsync(2000);
    expect(send).toHaveBeenCalledTimes(1);
    expect(document.getElementById('ptbr-writer-root')?.shadowRoot?.querySelector('.ptbr-badge')).toBeFalsy();
  });

  it('attaches to a field focused before the script started', async () => {
    site(true);
    const el = textarea();
    el.focus();
    dispose = await startContent();
    type(el, 'um dois três quatro');
    await vi.advanceTimersByTimeAsync(1000);
    expect(send).toHaveBeenCalledTimes(1);
  });

  it('switching the extension off also stops it', async () => {
    site(true);
    dispose = await startContent();
    const el = textarea();
    el.focus();
    store.settings = { enabled: false, sites: { [location.hostname]: { enabled: true, level: 'leve' } } };
    for (const l of listeners) l({ settings: {} }, 'local');
    await vi.advanceTimersByTimeAsync(0);
    type(el, 'um dois três quatro');
    await vi.advanceTimersByTimeAsync(2000);
    expect(send).not.toHaveBeenCalled();
  });

  it('follows a new level', async () => {
    site(true, 'descritivo');
    dispose = await startContent();
    const el = textarea();
    el.focus();
    await change(true, 'leve');
    type(el, 'um dois três quatro');
    await vi.advanceTimersByTimeAsync(1000);
    expect(send.mock.calls.map((c) => c[0].level)).toEqual(['leve']);
  });

  it('starts working when the site is switched on', async () => {
    site(false);
    dispose = await startContent();
    const el = textarea();
    el.focus();
    type(el, 'um dois três quatro');
    await vi.advanceTimersByTimeAsync(2000);
    expect(send).not.toHaveBeenCalled();
    await change(true);
    type(el, 'um dois três quatro cinco');
    await vi.advanceTimersByTimeAsync(1000);
    expect(send).toHaveBeenCalledTimes(1);
  });
});

describe('accepting one of several suggestions', () => {
  const TEXT = 'Foi decidido que o deploy a nível de equipe vai ser realizado na sexta.';
  const suggestions = [
    { original: 'Foi decidido', replacement: 'A equipe decidiu', rule: 'PTW-1', severity: 'erro', occurrence: 1 },
    { original: 'a nível de', replacement: 'da', rule: 'PTW-5', severity: 'erro', occurrence: 1 },
    { original: 'vai ser realizado', replacement: 'acontece', rule: 'PTW-4', severity: 'erro', occurrence: 1 },
  ];
  const shadow = () => document.getElementById('ptbr-writer-root')!.shadowRoot!;
  const acceptFirst = () => {
    (shadow().querySelector('.ptbr-underline-erro') as HTMLElement).click();
    (shadow().querySelector('.ptbr-card [data-action="accept"]') as HTMLElement).click();
  };

  it('keeps the other suggestions, shifted, and does not call the API again', async () => {
    site(true);
    send.mockImplementation(async () => ({ ok: true, suggestions }) as never);
    dispose = await startContent();
    const el = textarea();
    el.focus();
    type(el, TEXT);
    await vi.advanceTimersByTimeAsync(1000);
    expect(shadow().querySelectorAll('.ptbr-underline-erro').length).toBe(3);

    acceptFirst();
    await vi.advanceTimersByTimeAsync(2000);
    expect(el.value).toBe('A equipe decidiu que o deploy a nível de equipe vai ser realizado na sexta.');
    expect(shadow().querySelectorAll('.ptbr-underline-erro').length).toBe(2);
    expect(shadow().querySelector('.ptbr-badge')?.textContent).toBe('2');

    acceptFirst();
    acceptFirst();
    await vi.advanceTimersByTimeAsync(2000);
    expect(el.value).toBe('A equipe decidiu que o deploy da equipe acontece na sexta.');
    expect(shadow().querySelectorAll('.ptbr-underline-erro').length).toBe(0);
    expect(send).toHaveBeenCalledTimes(1);
  });

  it('typing again after accepting clears and checks everything again', async () => {
    site(true);
    send.mockImplementation(async () => ({ ok: true, suggestions }) as never);
    dispose = await startContent();
    const el = textarea();
    el.focus();
    type(el, TEXT);
    await vi.advanceTimersByTimeAsync(1000);
    acceptFirst();
    type(el, el.value + ' Mais uma frase.');
    expect(shadow().querySelectorAll('.ptbr-underline-erro').length).toBe(0);
    await vi.advanceTimersByTimeAsync(1000);
    expect(send).toHaveBeenCalledTimes(2);
  });
});
