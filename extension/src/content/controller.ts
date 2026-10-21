import { addNeverFix } from '../never-fix';
import { ruleTitles } from '../rules';
import { levelFor, loadSettings, saveSettings } from '../settings';
import { SKILL } from '../skills';
import type { CheckError, CheckRequest, CheckResult, Level, Located } from '../types';
import { acceptSuggestion } from './apply';
import { forget, renderBadge, renderSuggestions, repositionAll } from './overlay';
import { textModel } from './text';
import { createWatcher } from './watcher';

const TITLES = ruleTitles(SKILL);
const TEXT_INPUTS = ['text', 'search', 'email', 'url'];

interface FieldState {
  located: Located[];
  offset: number;
  state: 'idle' | 'checking' | CheckError;
}

// The editable root a focus event belongs to, or null when it is not a text field.
export function editableRoot(target: EventTarget | null): HTMLElement | null {
  if (target instanceof HTMLTextAreaElement) return target;
  if (target instanceof HTMLInputElement) return TEXT_INPUTS.includes(target.type) ? target : null;
  if (!(target instanceof HTMLElement) || !target.isContentEditable) return null;
  let root = target;
  while (root.parentElement?.isContentEditable) root = root.parentElement;
  return root;
}

// Runs the content script on this page. It follows the settings: when this site's level changes,
// the fields start over with the new level; when the site or the extension is switched off, it
// stops and clears what it drew. Resolves to a function that undoes it all (for tests).
export async function startContent(): Promise<() => void> {
  const fields = new Map<HTMLElement, FieldState>();
  let level: Level | null = null;
  let watcher: ReturnType<typeof createWatcher> | null = null;

  const field = (el: HTMLElement) => {
    let f = fields.get(el);
    if (!f) fields.set(el, (f = { located: [], offset: 0, state: 'idle' }));
    return f;
  };

  const render = (el: HTMLElement) => {
    const f = field(el);
    const drop = (l: Located) => {
      f.located = f.located.filter((x) => x !== l);
      render(el);
    };
    // The accepted edit is the extension's own: the other suggestions stay, shifted by the change in
    // length, and the API is not called again until the user types.
    const accept = (l: Located) => {
      const start = l.start + f.offset;
      const end = l.end + f.offset;
      const before = textModel(el).text;
      watcher?.expectEdit(el, before.slice(0, start) + l.replacement + before.slice(end));
      if (!acceptSuggestion(el, l, f.offset)) {
        watcher?.expectEdit(el, null);
        drop(l);
        return;
      }
      const delta = l.replacement.length - (l.end - l.start);
      f.located = f.located
        .filter((x) => x !== l)
        .map((x) => (x.start >= l.end ? { ...x, start: x.start + delta, end: x.end + delta } : x));
      render(el);
    };
    // "Sempre ignorar" saves the phrase and drops the underline; the API is not called again.
    const neverFix = (l: Located) => {
      drop(l);
      void loadSettings()
        .then((s) => saveSettings({ ...s, neverFix: addNeverFix(s.neverFix, l.original) }))
        .catch(() => {});
    };
    renderSuggestions(el, f.located, f.offset, accept, drop, { titles: TITLES, onNeverFix: neverFix });
    renderBadge(el, f.state, f.located.length);
  };

  const attach = (target: EventTarget | null) => {
    const el = editableRoot(target);
    if (!watcher || !el || fields.has(el)) return;
    field(el);
    watcher.attach(el);
  };

  const stop = () => {
    for (const el of fields.keys()) {
      watcher?.detach(el);
      forget(el);
    }
    fields.clear();
    watcher = null;
  };

  const follow = async () => {
    const next = levelFor(location.hostname, await loadSettings());
    if (next === level) return;
    stop();
    level = next;
    if (!level) return;
    watcher = createWatcher({
      level,
      send: (r: CheckRequest) => chrome.runtime.sendMessage<CheckRequest, CheckResult>(r),
      onResult(el, located, offset) {
        Object.assign(field(el), { located, offset });
        render(el);
      },
      onState(el, state) {
        field(el).state = state;
        render(el);
      },
    });
    // The focused field gets no new focusin: before the script ran, or when the site was switched on.
    attach(document.activeElement);
  };

  await follow();
  const onChanged = (changes: Record<string, unknown>, area: string) => {
    if (area === 'local' && 'settings' in changes) void follow().catch(() => {});
  };
  chrome.storage.onChanged?.addListener(onChanged);

  const onFocus = (e: FocusEvent) => attach(e.composedPath()[0] ?? e.target);
  document.addEventListener('focusin', onFocus, true);

  let frame = 0;
  const schedule = () => {
    if (frame) return;
    frame = requestAnimationFrame(() => {
      frame = 0;
      for (const el of fields.keys()) {
        if (!el.isConnected) {
          watcher?.detach(el);
          fields.delete(el);
        }
      }
      repositionAll();
    });
  };
  window.addEventListener('scroll', schedule, { capture: true, passive: true });
  window.addEventListener('resize', schedule, { passive: true });

  return () => {
    stop();
    chrome.storage.onChanged?.removeListener(onChanged);
    document.removeEventListener('focusin', onFocus, true);
    window.removeEventListener('scroll', schedule, { capture: true });
    window.removeEventListener('resize', schedule);
  };
}
