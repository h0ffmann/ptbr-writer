// @vitest-environment happy-dom
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createWatcher } from '../src/content/watcher';
import type { CheckRequest, CheckResult } from '../src/types';


function field(value: string, type?: string): HTMLInputElement | HTMLTextAreaElement {
  const el = type ? document.createElement('input') : document.createElement('textarea');
  if (type) (el as HTMLInputElement).type = type;
  el.value = value;
  document.body.appendChild(el);
  return el;
}
const type = (el: HTMLInputElement | HTMLTextAreaElement, v: string) => {
  el.value = v;
  el.dispatchEvent(new Event('input', { bubbles: true }));
};

beforeEach(() => vi.useFakeTimers());
afterEach(() => vi.useRealTimers());

function setup(send: (r: CheckRequest) => Promise<CheckResult>) {
  const onResult = vi.fn();
  const onState = vi.fn();
  const w = createWatcher({ level: 'descritivo', send, onResult, onState });
  return { w, onResult, onState };
}

describe('watcher', () => {
  it('debounces 3 inputs into 1 send, 1000 ms after the last', async () => {
    const send = vi.fn(async (_r: CheckRequest) => ({ ok: true, suggestions: [] }) as CheckResult);
    const { w } = setup(send);
    const el = field('um dois três');
    w.attach(el);
    type(el, 'um dois três a');
    await vi.advanceTimersByTimeAsync(500);
    type(el, 'um dois três ab');
    await vi.advanceTimersByTimeAsync(500);
    type(el, 'um dois três abc');
    await vi.advanceTimersByTimeAsync(999);
    expect(send).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(1);
    expect(send).toHaveBeenCalledTimes(1);
    const req = send.mock.calls[0][0];
    expect(req).toMatchObject({ type: 'check', text: 'um dois três abc', level: 'descritivo' });
    expect(typeof req.key).toBe('string');
  });

  it('uses a stable key per field', async () => {
    const send = vi.fn(async (_r: CheckRequest) => ({ ok: true, suggestions: [] }) as CheckResult);
    const { w } = setup(send);
    const a = field('um dois três');
    const b = field('um dois três');
    w.attach(a);
    w.attach(b);
    type(a, 'um dois três 1');
    await vi.advanceTimersByTimeAsync(1000);
    type(a, 'um dois três 2');
    await vi.advanceTimersByTimeAsync(1000);
    type(b, 'um dois três 3');
    await vi.advanceTimersByTimeAsync(1000);
    const keys = send.mock.calls.map((c) => c[0].key);
    expect(keys[0]).toBe(keys[1]);
    expect(keys[2]).not.toBe(keys[0]);
  });

  it('ignores passwords and texts under 3 words', async () => {
    const send = vi.fn(async (_r: CheckRequest) => ({ ok: true, suggestions: [] }) as CheckResult);
    const { w } = setup(send);
    const pw = field('uma senha bem longa', 'password');
    const short = field('');
    w.attach(pw);
    w.attach(short);
    type(pw, 'uma senha bem longa x');
    type(short, 'dois termos');
    await vi.advanceTimersByTimeAsync(3000);
    expect(send).not.toHaveBeenCalled();
  });

  it('clears suggestions on every input', () => {
    const { w, onResult } = setup(async () => ({ ok: true, suggestions: [] }));
    const el = field('');
    w.attach(el);
    type(el, 'a');
    expect(onResult).toHaveBeenCalledWith(el, [], 0);
  });

  it('discards a response when the field changed meanwhile', async () => {
    let resolve!: (r: CheckResult) => void;
    const send = vi.fn((_r: CheckRequest) => new Promise<CheckResult>((r) => (resolve = r)));
    const { w, onResult } = setup(send);
    const el = field('um dois três');
    w.attach(el);
    type(el, 'a casa é bonito');
    await vi.advanceTimersByTimeAsync(1000);
    el.value = 'outro texto bem diferente'; // changed without a new input event
    resolve({
      ok: true,
      suggestions: [{ original: 'casa', replacement: 'casas', rule: 'R', severity: 'erro', occurrence: 1 }],
    });
    await vi.advanceTimersByTimeAsync(0);
    expect(onResult).toHaveBeenCalledTimes(1); // only the clearing call
    expect(onResult).toHaveBeenLastCalledWith(el, [], 0);
  });

  it('applies a fresh response as located suggestions with the offset', async () => {
    const send = vi.fn(async (_r: CheckRequest) => ({
      ok: true,
      suggestions: [{ original: 'casa', replacement: 'lar', rule: 'R', severity: 'aviso', occurrence: 1 }],
    }) as CheckResult);
    const { w, onResult, onState } = setup(send);
    const el = field('um dois três');
    w.attach(el);
    type(el, 'a casa é bonita');
    await vi.advanceTimersByTimeAsync(1000);
    const last = onResult.mock.calls.at(-1)!;
    expect(last[0]).toBe(el);
    expect(last[1]).toMatchObject([{ original: 'casa', start: 2, end: 6 }]);
    expect(last[2]).toBe(0);
    expect(onState.mock.calls.map((c) => c[1])).toEqual(['checking', 'idle']);
  });

  it('reports an error state', async () => {
    const send = vi.fn(async (_r: CheckRequest) => ({ ok: false, error: 'auth' }) as CheckResult);
    const { w, onState, onResult } = setup(send);
    const el = field('um dois três');
    w.attach(el);
    type(el, 'a casa é bonita');
    await vi.advanceTimersByTimeAsync(1000);
    expect(onState).toHaveBeenLastCalledWith(el, 'auth');
    expect(onResult).toHaveBeenCalledTimes(1);
  });

  it('detach stops pending checks', async () => {
    const send = vi.fn(async (_r: CheckRequest) => ({ ok: true, suggestions: [] }) as CheckResult);
    const { w } = setup(send);
    const el = field('um dois três');
    w.attach(el);
    type(el, 'a casa é bonita');
    w.detach(el);
    await vi.advanceTimersByTimeAsync(2000);
    expect(send).not.toHaveBeenCalled();
  });
});
