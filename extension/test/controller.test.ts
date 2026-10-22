// @vitest-environment happy-dom
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { startContent } from '../src/content/controller';
import type { CheckRequest, Level } from '../src/types';

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
  send.mockReset().mockImplementation(async () => ({ ok: true, suggestions: [] }));
  (globalThis as any).chrome = {
    storage: {
      local: {
        get: async () => store,
        set: async (v: Record<string, unknown>) => {
          Object.assign(store, v);
          for (const l of listeners) l({ settings: {} }, 'local');
        },
      },
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
  it('a tab left with the old script after the extension reloads says to reload the page', async () => {
    site(true);
    send.mockImplementation(async () => {
      throw new Error('Extension context invalidated.');
    });
    dispose = await startContent();
    const el = textarea();
    el.focus();
    type(el, 'um dois três quatro');
    await vi.advanceTimersByTimeAsync(1000);
    const badge = document.getElementById('ptbr-writer-root')?.shadowRoot?.querySelector('.ptbr-badge');
    expect(badge?.textContent).toBe('recarregue a página');
  });

  it('any other send failure still shows a provider error', async () => {
    site(true);
    send.mockImplementation(async () => {
      throw new Error('boom');
    });
    dispose = await startContent();
    const el = textarea();
    el.focus();
    type(el, 'um dois três quatro');
    await vi.advanceTimersByTimeAsync(1000);
    const badge = document.getElementById('ptbr-writer-root')?.shadowRoot?.querySelector('.ptbr-badge');
    expect(badge?.textContent).toBe('erro do provider');
  });

  it('follows a new wait: no call before it, one call after it', async () => {
    site(true);
    dispose = await startContent();
    const el = textarea();
    el.focus();
    store.settings = { sites: { [location.hostname]: { enabled: true, level: 'descritivo' } }, delayMs: 2000 };
    for (const l of listeners) l({ settings: {} }, 'local');
    await vi.advanceTimersByTimeAsync(0);
    type(el, 'um dois três quatro');
    await vi.advanceTimersByTimeAsync(1000);
    expect(send).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(1000);
    expect(send).toHaveBeenCalledTimes(1);
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

  it('a save that only changes neverFix keeps the underlines and does not call the API again', async () => {
    site(true);
    send.mockImplementation(async () => ({ ok: true, suggestions }) as never);
    dispose = await startContent();
    const el = textarea();
    el.focus();
    type(el, TEXT);
    await vi.advanceTimersByTimeAsync(1000);
    expect(shadow().querySelectorAll('.ptbr-underline-erro').length).toBe(3);
    await chrome.storage.local.set({ settings: { ...(store.settings as object), neverFix: ['Kafka'] } });
    await vi.advanceTimersByTimeAsync(2000);
    expect(shadow().querySelectorAll('.ptbr-underline-erro').length).toBe(3);
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

describe('Sempre ignorar', () => {
  it('saves the original in neverFix, drops the underline and does not call the API again', async () => {
    site(true);
    send.mockImplementation(
      async () => ({ ok: true, suggestions: [{ original: 'a nível de', replacement: 'da', rule: 'PTW-5', severity: 'erro', occurrence: 1 }] }) as never,
    );
    dispose = await startContent();
    const el = textarea();
    el.focus();
    type(el, 'O deploy a nível de equipe sai na sexta.');
    await vi.advanceTimersByTimeAsync(1000);
    const shadow = document.getElementById('ptbr-writer-root')!.shadowRoot!;
    (shadow.querySelector('.ptbr-underline-erro') as HTMLElement).click();
    (shadow.querySelector('.ptbr-card [data-action="never"]') as HTMLElement).click();
    await vi.advanceTimersByTimeAsync(2000);
    expect((store.settings as { neverFix: string[] }).neverFix).toEqual(['a nível de']);
    expect(shadow.querySelectorAll('.ptbr-underline-erro').length).toBe(0);
    expect(send).toHaveBeenCalledTimes(1);
  });
});

describe('card titles', () => {
  it('a CIE card shows the title from cientifico/SKILL.md', async () => {
    site(true, 'científico');
    send.mockImplementation(
      async () => ({ ok: true, suggestions: [{ original: 'aumentou 40%', replacement: 'aumentou', rule: 'CIE-1', severity: 'erro', occurrence: 1 }] }) as never,
    );
    dispose = await startContent();
    const el = textarea();
    el.focus();
    type(el, 'O uso da ferramenta aumentou 40% no trimestre.');
    await vi.advanceTimersByTimeAsync(1000);
    const shadow = document.getElementById('ptbr-writer-root')!.shadowRoot!;
    (shadow.querySelector('.ptbr-underline-erro') as HTMLElement).click();
    expect(shadow.querySelector('.ptbr-card .ptbr-rule')?.textContent).toBe('CIE-1 · Nada inventado');
  });
});
