import { dropOverlaps, locate } from '../suggestions';
import type { CheckError, CheckRequest, CheckResult, Level, Located } from '../types';
import { caretOffset, selectForSending, textModel, wordCount } from './text';

export interface WatcherOptions {
  level: Level;
  send: (r: CheckRequest) => Promise<CheckResult>;
  onResult: (el: HTMLElement, located: Located[], offset: number) => void;
  onState: (el: HTMLElement, s: 'idle' | 'checking' | CheckError) => void;
  delayMs?: number;
}

const MIN_WORDS = 3;

export function createWatcher(opts: WatcherOptions) {
  const keys = new WeakMap<HTMLElement, string>();
  const timers = new Map<HTMLElement, ReturnType<typeof setTimeout>>();
  const listeners = new Map<HTMLElement, () => void>();
  let seq = 0;

  const keyOf = (el: HTMLElement) => {
    let k = keys.get(el);
    if (!k) keys.set(el, (k = `f${++seq}`));
    return k;
  };
  const ignored = (el: HTMLElement) => el instanceof HTMLInputElement && el.type === 'password';

  async function check(el: HTMLElement) {
    timers.delete(el);
    const m = textModel(el);
    if (ignored(el) || wordCount(m.text) < MIN_WORDS) return opts.onState(el, 'idle');
    const sent = selectForSending(m.text, caretOffset(el, m));
    if (wordCount(sent.text) < MIN_WORDS) return opts.onState(el, 'idle');
    opts.onState(el, 'checking');
    let result: CheckResult;
    try {
      result = await opts.send({ type: 'check', key: keyOf(el), text: sent.text, level: opts.level });
    } catch {
      result = { ok: false, error: 'network' };
    }
    if (!listeners.has(el)) return;
    if (textModel(el).text !== m.text) return opts.onState(el, 'idle'); // field changed: stale
    if (!result.ok) {
      opts.onState(el, result.error);
      return;
    }
    // A span across a line break cannot be shown or applied in a contenteditable, whose breaks are synthetic.
    const located = locate(sent.text, result.suggestions).filter(
      (l) => !sent.text.slice(l.start, l.end).includes('\n'),
    );
    opts.onResult(el, dropOverlaps(located), sent.offset);
    opts.onState(el, 'idle');
  }

  // An exception inside check (a render that throws) shows as a provider error, never as an unhandled rejection.
  const run = (el: HTMLElement) =>
    check(el).catch(() => {
      try {
        opts.onState(el, 'provider');
      } catch {
        // nothing left to report to
      }
    });

  return {
    attach(el: HTMLElement) {
      if (listeners.has(el)) return;
      const onInput = () => {
        opts.onResult(el, [], 0);
        clearTimeout(timers.get(el));
        timers.set(el, setTimeout(() => run(el), opts.delayMs ?? 1000));
      };
      listeners.set(el, onInput);
      el.addEventListener('input', onInput);
    },
    detach(el: HTMLElement) {
      const l = listeners.get(el);
      if (l) el.removeEventListener('input', l);
      listeners.delete(el);
      clearTimeout(timers.get(el));
      timers.delete(el);
    },
  };
}
