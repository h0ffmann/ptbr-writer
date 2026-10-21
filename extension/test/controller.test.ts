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
