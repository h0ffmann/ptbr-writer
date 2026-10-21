import { levelFor, loadSettings } from '../settings';
import type { CheckError, CheckRequest, CheckResult, Located } from '../types';
import { acceptSuggestion } from './apply';
import { renderBadge, renderSuggestions, repositionAll } from './overlay';
import { createWatcher } from './watcher';

const TEXT_INPUTS = ['text', 'search', 'email', 'url'];

interface FieldState {
  located: Located[];
  offset: number;
  state: 'idle' | 'checking' | CheckError;
}

// The editable root a focus event belongs to, or null when it is not a text field.
function editableRoot(target: EventTarget | null): HTMLElement | null {
  if (target instanceof HTMLTextAreaElement) return target;
  if (target instanceof HTMLInputElement) return TEXT_INPUTS.includes(target.type) ? target : null;
  if (!(target instanceof HTMLElement) || !target.isContentEditable) return null;
  let root = target;
  while (root.parentElement?.isContentEditable) root = root.parentElement;
  return root;
}

async function start() {
  const level = levelFor(location.hostname, await loadSettings());
  if (!level) return;

  const fields = new Map<HTMLElement, FieldState>();
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
    // On success the editor's input event clears the field; the watcher checks again after the debounce.
    const accept = (l: Located) => {
      if (!acceptSuggestion(el, l, f.offset)) drop(l);
    };
    renderSuggestions(el, f.located, f.offset, accept, drop);
    renderBadge(el, f.state, f.located.length);
  };

  const watcher = createWatcher({
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

  document.addEventListener(
    'focusin',
    (e) => {
      const el = editableRoot(e.composedPath()[0] ?? e.target);
      if (!el || fields.has(el)) return;
      field(el);
      watcher.attach(el);
    },
    true,
  );

  let frame = 0;
  const schedule = () => {
    if (frame) return;
    frame = requestAnimationFrame(() => {
      frame = 0;
      for (const el of fields.keys()) {
        if (!el.isConnected) {
          watcher.detach(el);
          fields.delete(el);
        }
      }
      repositionAll();
    });
  };
  window.addEventListener('scroll', schedule, { capture: true, passive: true });
  window.addEventListener('resize', schedule, { passive: true });
}

void start();
