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
  const doc = el.ownerDocument;
  el.focus();
  sel.removeAllRanges();
  sel.addRange(range);
  // Editors that keep their own text state (WhatsApp Web) learn the selection from selectionchange,
  // which the browser fires later; fire it now so the paste below lands on the selected span.
  doc.dispatchEvent(new Event('selectionchange'));
  if (pasteInto(el, l.replacement)) return true;
  // An editor that does not take over paste: execCommand goes through its input pipeline (undo stack).
  return doc.execCommand('insertText', false, l.replacement);
}

// Offers the replacement as a plain-text paste. True when the editor handled it (prevented the
// default), as WhatsApp's does: it then writes the text through its own state, which a DOM insert
// would bypass and the editor would undo.
function pasteInto(el: HTMLElement, text: string): boolean {
  if (typeof DataTransfer === 'undefined' || typeof ClipboardEvent === 'undefined') return false;
  const data = new DataTransfer();
  data.setData('text/plain', text);
  const paste = new ClipboardEvent('paste', { clipboardData: data, bubbles: true, cancelable: true, composed: true });
  el.dispatchEvent(paste);
  return paste.defaultPrevented;
}
