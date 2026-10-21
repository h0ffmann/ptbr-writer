import type { Located } from '../types';
import { rangeFor, textModel } from './text';

// Replaces one suggestion in the editor. Returns false, changing nothing, when the span is no longer `original`.
export function acceptSuggestion(el: HTMLElement, l: Located, offset: number): boolean {
  const start = l.start + offset;
  const end = l.end + offset;
  if (el instanceof HTMLTextAreaElement || el instanceof HTMLInputElement) {
    if (el.value.slice(start, end) !== l.original) return false;
    el.setRangeText(l.replacement, start, end);
    el.dispatchEvent(new InputEvent('input', { bubbles: true }));
    return true;
  }
  const m = textModel(el);
  if (m.text.slice(start, end) !== l.original) return false;
  const range = rangeFor(m, start, end);
  const sel = el.ownerDocument.getSelection();
  if (!range || !sel) return false;
  el.focus();
  sel.removeAllRanges();
  sel.addRange(range);
  // execCommand goes through the editor's own input pipeline (undo stack, React/Lexical state).
  return el.ownerDocument.execCommand('insertText', false, l.replacement);
}
