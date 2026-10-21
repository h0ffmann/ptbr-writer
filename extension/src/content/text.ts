export interface TextModel {
  text: string;
  nodes: { node: Text; start: number }[];
}

export const MAX_CHARS = 4000;
const QUOTED = 'blockquote, .gmail_quote';

export function textModel(root: HTMLElement): TextModel {
  if (root instanceof HTMLTextAreaElement || root instanceof HTMLInputElement) {
    return { text: root.value, nodes: [] };
  }
  const doc = root.ownerDocument;
  const walker = doc.createTreeWalker(root, NodeFilter.SHOW_ELEMENT | NodeFilter.SHOW_TEXT, {
    acceptNode(n) {
      if (n.nodeType === Node.TEXT_NODE) return NodeFilter.FILTER_ACCEPT;
      return (n as Element).matches(QUOTED) ? NodeFilter.FILTER_REJECT : NodeFilter.FILTER_SKIP;
    },
  });
  let text = '';
  const nodes: TextModel['nodes'] = [];
  for (let n = walker.nextNode(); n; n = walker.nextNode()) {
    const t = n as Text;
    nodes.push({ node: t, start: text.length });
    text += t.data;
  }
  return { text, nodes };
}

// Offset of the selection focus within the model; 0 when outside it.
export function caretOffset(root: HTMLElement, m: TextModel): number {
  if (root instanceof HTMLTextAreaElement || root instanceof HTMLInputElement) {
    return root.selectionStart ?? 0;
  }
  const focus = root.ownerDocument.getSelection()?.focusNode;
  const offset = root.ownerDocument.getSelection()?.focusOffset ?? 0;
  const hit = m.nodes.find((x) => x.node === focus);
  return hit ? hit.start + offset : 0;
}

export function selectForSending(text: string, caret: number): { text: string; offset: number } {
  if (text.length <= MAX_CHARS) return { text, offset: 0 };
  const c = Math.min(Math.max(caret, 0), text.length);
  const before = text.lastIndexOf('\n', c - 1);
  const start = before < 0 ? 0 : before + 1;
  const after = text.indexOf('\n', c);
  const end = after < 0 ? text.length : after;
  if (end - start <= MAX_CHARS) return { text: text.slice(start, end), offset: start };
  const from = Math.max(start, Math.min(c - MAX_CHARS / 2, end - MAX_CHARS));
  return { text: text.slice(from, from + MAX_CHARS), offset: from };
}

export function rangeFor(m: TextModel, start: number, end: number): Range | null {
  if (start < 0 || end <= start || end > m.text.length) return null;
  const find = (pos: number, atEnd: boolean) =>
    m.nodes.find((x) => (atEnd ? pos > x.start : pos >= x.start) && pos <= x.start + x.node.data.length && (atEnd || pos < x.start + x.node.data.length));
  const a = find(start, false);
  const b = find(end, true);
  if (!a || !b) return null;
  const r = a.node.ownerDocument.createRange();
  r.setStart(a.node, start - a.start);
  r.setEnd(b.node, end - b.start);
  return r;
}

export function wordCount(text: string): number {
  return text.split(/\s+/).filter(Boolean).length;
}
