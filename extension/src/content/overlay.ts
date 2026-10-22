import { NEVER_FIX_MAX_WORDS } from '../never-fix';
import type { CheckError, Located } from '../types';
import { rangeFor, textModel } from './text';

// Everything is drawn in one shadow root outside the editor: the editor's DOM is never touched.
const HOST_ID = 'ptbr-writer-root';

const STYLE = `
:host { all: initial; }
.ptbr-layer, .ptbr-mirror { position: absolute; top: 0; left: 0; }
.ptbr-underline { pointer-events: none; }
.ptbr-seg { position: absolute; box-sizing: border-box; border-bottom: 2px solid; }
.ptbr-hit { cursor: pointer; }
.ptbr-underline-erro .ptbr-seg { border-color: #d93025; }
.ptbr-underline-aviso .ptbr-seg { border-color: #e8a400; }
.ptbr-underline:hover .ptbr-seg { background: rgba(232, 164, 0, 0.15); }
.ptbr-underline-erro:hover .ptbr-seg { background: rgba(217, 48, 37, 0.12); }
.ptbr-mirror { visibility: hidden; overflow: hidden; box-sizing: border-box; border-style: solid; border-color: transparent; }
.ptbr-card, .ptbr-badge {
  pointer-events: auto;
  position: absolute;
  font: 13px/1.4 system-ui, -apple-system, "Segoe UI", sans-serif;
  color: #1f1f1f;
}
.ptbr-card {
  box-sizing: border-box;
  max-width: 320px;
  padding: 10px 12px;
  background: #fff;
  border: 1px solid #d0d0d0;
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.18);
}
.ptbr-rule { margin: 0 0 6px; font-size: 12px; color: #5f5f5f; }
.ptbr-change { margin: 0 0 8px; }
.ptbr-change s { color: #a0332b; }
.ptbr-change strong { color: #146c2e; }
.ptbr-actions { display: flex; gap: 6px; }
.ptbr-actions button {
  font: inherit;
  padding: 3px 10px;
  border-radius: 6px;
  border: 1px solid #c4c4c4;
  background: #f5f5f5;
  color: inherit;
  cursor: pointer;
}
.ptbr-actions button[data-action="accept"] { background: #146c2e; border-color: #146c2e; color: #fff; }
.ptbr-badge {
  box-sizing: border-box;
  min-width: 20px;
  height: 20px;
  padding: 0 6px;
  border-radius: 10px;
  background: #146c2e;
  color: #fff;
  font-size: 11px;
  line-height: 20px;
  text-align: center;
  white-space: nowrap;
  cursor: pointer;
}
.ptbr-badge[data-state="checking"] { background: #5f5f5f; }
.ptbr-badge[data-state="error"] { background: #a0332b; }
@media (prefers-color-scheme: dark) {
  .ptbr-card { background: #2b2b2b; border-color: #4a4a4a; color: #ececec; }
  .ptbr-rule { color: #b0b0b0; }
  .ptbr-change s { color: #ff8a80; }
  .ptbr-change strong { color: #81c995; }
  .ptbr-actions button { background: #3a3a3a; border-color: #555; }
}
`;

const BADGE_LABEL: Record<CheckError, string> = {
  'no-key': 'sem chave',
  auth: 'chave inválida',
  'rate-limit': 'limite atingido',
  provider: 'erro do provider',
  network: 'erro do provider',
  reload: 'recarregue a página',
};

interface Drawn {
  located: Located[];
  offset: number;
  onAccept: (l: Located) => void;
  onIgnore: (l: Located) => void;
  titles: Record<string, string>;
  onNeverFix?: (l: Located) => void;
  layer: HTMLElement;
  badge?: HTMLElement;
  // Where each underlined span is on screen (client coordinates), for the hover test.
  hits: { rect: DOMRect; l: Located }[];
}

const drawn = new Map<HTMLElement, Drawn>();
// `hover`: opened by resting the mouse on a word, so it closes when the mouse leaves. A card opened
// by a click (underline strip or badge) stays until Escape or a click outside.
let card: { el: HTMLElement; node: HTMLElement; l: Located; hover: boolean } | null = null;

const HOVER_MS = 200;
const LEAVE_MS = 300;
let openTimer: ReturnType<typeof setTimeout> | undefined;
let closeTimer: ReturnType<typeof setTimeout> | undefined;

function hitAt(x: number, y: number): { el: HTMLElement; d: Drawn; l: Located } | null {
  for (const [el, d] of drawn)
    for (const h of d.hits)
      if (x >= h.rect.left && x <= h.rect.right && y >= h.rect.top && y <= h.rect.bottom) return { el, d, l: h.l };
  return null;
}

// The words stay clickable for the editor (no pointer events on them), so hover is found by
// comparing the mouse position with the drawn rects.
function onMouseMove(e: Event) {
  const { clientX: x, clientY: y } = e as MouseEvent;
  // A card whose node left the page (the site rebuilt the overlay host) no longer counts.
  if (card && !card.node.isConnected) card = null;
  if (card && e.composedPath().includes(card.node)) {
    clearTimeout(closeTimer);
    return;
  }
  const hit = hitAt(x, y);
  clearTimeout(openTimer);
  if (hit && card?.l === hit.l) {
    clearTimeout(closeTimer);
    return;
  }
  if (hit) {
    openTimer = setTimeout(() => {
      if (card && !card.hover) return;
      openCard(hit.el, hit.l, hit.d, true);
    }, HOVER_MS);
    return;
  }
  if (card?.hover) {
    clearTimeout(closeTimer);
    closeTimer = setTimeout(() => {
      if (card?.hover) closeCard();
    }, LEAVE_MS);
  }
}

export function overlayRoot(doc: Document = document): ShadowRoot {
  let host = doc.getElementById(HOST_ID);
  if (!host || !host.shadowRoot) {
    host?.remove();
    host = doc.createElement('div');
    host.id = HOST_ID;
    const root = host.attachShadow({ mode: 'open' });
    const style = doc.createElement('style');
    style.textContent = STYLE;
    root.appendChild(style);
    (doc.documentElement ?? doc.body).appendChild(host);
    doc.addEventListener('mousedown', closeOnOutside, true);
    doc.addEventListener('keydown', closeOnEscape, true);
    doc.addEventListener('mousemove', onMouseMove, { capture: true, passive: true });
  }
  return host.shadowRoot!;
}

function closeOnOutside(e: Event) {
  if (card && !e.composedPath().includes(card.node)) closeCard();
}
function closeOnEscape(e: Event) {
  if (card && (e as KeyboardEvent).key === 'Escape') closeCard();
}
function closeCard() {
  clearTimeout(closeTimer);
  card?.node.remove();
  card = null;
}

function entry(el: HTMLElement): Drawn {
  const root = overlayRoot(el.ownerDocument);
  let d = drawn.get(el);
  if (!d || !root.contains(d.layer)) {
    const layer = el.ownerDocument.createElement('div');
    layer.className = 'ptbr-layer';
    root.appendChild(layer);
    d = { located: [], offset: 0, onAccept: () => {}, onIgnore: () => {}, titles: {}, layer, badge: d?.badge, hits: [] };
    drawn.set(el, d);
  }
  return d;
}

export function renderSuggestions(
  el: HTMLElement,
  located: Located[],
  offset: number,
  onAccept: (l: Located) => void,
  onIgnore: (l: Located) => void,
  extra: { titles?: Record<string, string>; onNeverFix?: (l: Located) => void } = {},
): void {
  const d = entry(el);
  Object.assign(d, { located, offset, onAccept, onIgnore, titles: extra.titles ?? {}, onNeverFix: extra.onNeverFix });
  if (card?.el === el) closeCard();
  draw(el, d);
}

export function renderBadge(el: HTMLElement, state: 'idle' | 'checking' | CheckError, count: number): void {
  const d = entry(el);
  if (!d.badge || !d.badge.isConnected) {
    d.badge = el.ownerDocument.createElement('div');
    d.badge.className = 'ptbr-badge';
    d.badge.addEventListener('mousedown', (e) => e.preventDefault());
    // The badge opens the first suggestion, the same card a click on its underline opens.
    d.badge.addEventListener('click', (e) => {
      e.stopPropagation();
      const now = drawn.get(el);
      if (now?.located.length) openCard(el, now.located[0], now);
    });
    overlayRoot(el.ownerDocument).appendChild(d.badge);
  }
  const b = d.badge;
  if (state === 'idle') {
    b.dataset.state = 'idle';
    b.textContent = String(count);
  } else if (state === 'checking') {
    b.dataset.state = 'checking';
    b.textContent = '…';
  } else {
    b.dataset.state = 'error';
    b.textContent = BADGE_LABEL[state];
  }
  b.title = `ptbr-writer: ${b.textContent}`;
  placeBadge(el, b);
}

// Removes everything drawn for an editor: its underlines, its card and its badge.
export function forget(el: HTMLElement): void {
  const d = drawn.get(el);
  if (!d) return;
  d.layer.remove();
  d.badge?.remove();
  if (card?.el === el) closeCard();
  drawn.delete(el);
}

// Redraws every editor; called on scroll and resize. Forgets editors that left the page.
export function repositionAll(): void {
  for (const [el, d] of drawn) {
    if (!el.isConnected) {
      forget(el);
      continue;
    }
    draw(el, d);
    if (d.badge) placeBadge(el, d.badge);
  }
  if (card) placeCard(card.node, card.el);
}

function draw(el: HTMLElement, d: Drawn) {
  const doc = el.ownerDocument;
  d.layer.replaceChildren();
  d.hits = [];
  const clip = el.getBoundingClientRect();
  const rectsOf = rectSource(el);
  for (const l of d.located) {
    const u = doc.createElement('div');
    u.className = `ptbr-underline ptbr-underline-${l.severity}`;
    u.title = l.rule;
    for (const r of rectsOf(l.start + d.offset, l.end + d.offset)) {
      if (!visible(r, clip)) continue;
      d.hits.push({ rect: r, l });
      const seg = doc.createElement('div');
      seg.className = 'ptbr-seg';
      place(seg, r, doc);
      seg.style.width = `${r.width}px`;
      seg.style.height = `${r.height}px`;
      // Only a thin strip at the bottom takes the mouse: the word itself stays clickable for the editor.
      seg.style.pointerEvents = 'none';
      const hit = doc.createElement('div');
      hit.className = 'ptbr-hit';
      Object.assign(hit.style, { position: 'absolute', left: '0px', right: '0px', bottom: '0px', height: '4px', pointerEvents: 'auto' });
      seg.appendChild(hit);
      u.appendChild(seg);
    }
    u.addEventListener('mousedown', (e) => e.preventDefault());
    u.addEventListener('click', (e) => {
      e.stopPropagation();
      openCard(el, l, d);
    });
    d.layer.appendChild(u);
  }
}

function openCard(el: HTMLElement, l: Located, d: Drawn, hover = false) {
  closeCard();
  const doc = el.ownerDocument;
  const node = doc.createElement('div');
  node.className = 'ptbr-card';
  node.setAttribute('role', 'dialog');
  const rule = doc.createElement('p');
  rule.className = 'ptbr-rule';
  const title = d.titles[l.rule];
  const rewritten = l.severity === 'aviso' && l.rule === 'PTW-17';
  rule.textContent = rewritten ? 'Reescrita' : title ? `${l.rule} · ${title}` : l.rule;
  const change = doc.createElement('p');
  change.className = 'ptbr-change';
  const s = doc.createElement('s');
  s.textContent = l.original;
  const strong = doc.createElement('strong');
  strong.textContent = l.replacement;
  change.append(s, ' → ', strong);
  const actions = doc.createElement('div');
  actions.className = 'ptbr-actions';
  const button = (action: string, label: string, run: () => void) => {
    const b = doc.createElement('button');
    b.type = 'button';
    b.dataset.action = action;
    b.textContent = label;
    b.addEventListener('mousedown', (e) => e.preventDefault());
    b.addEventListener('click', (e) => {
      e.stopPropagation();
      closeCard();
      run();
    });
    return b;
  };
  actions.append(button('accept', 'Aceitar', () => d.onAccept(l)), button('ignore', 'Ignorar', () => d.onIgnore(l)));
  if (d.onNeverFix && l.original.trim().split(/\s+/).length <= NEVER_FIX_MAX_WORDS) {
    const never = d.onNeverFix;
    actions.append(button('never', 'Sempre ignorar', () => never(l)));
  }
  node.append(rule, change, actions);
  node.addEventListener('mousedown', (e) => e.preventDefault());
  overlayRoot(doc).appendChild(node);
  card = { el, node, l, hover };
  node.dataset.start = String(l.start + d.offset);
  node.dataset.end = String(l.end + d.offset);
  placeCard(node, el);
}

function placeCard(node: HTMLElement, el: HTMLElement) {
  const rects = rectSource(el)(Number(node.dataset.start), Number(node.dataset.end));
  const r = rects[rects.length - 1] ?? el.getBoundingClientRect();
  const win = el.ownerDocument.defaultView;
  const h = node.offsetHeight;
  const w = node.offsetWidth;
  const vh = win?.innerHeight ?? Infinity;
  const vw = win?.innerWidth ?? Infinity;
  // Below the text when it fits, else above it: a composer at the bottom of the window (WhatsApp)
  // would push the card off screen.
  const top = r.bottom + 4 + h > vh && r.top - 4 - h >= 0 ? r.top - 4 - h : r.bottom + 4;
  const left = Math.max(4, Math.min(r.left, vw - w - 4));
  node.style.left = `${left + (win?.scrollX ?? 0)}px`;
  node.style.top = `${top + (win?.scrollY ?? 0)}px`;
}

function placeBadge(el: HTMLElement, b: HTMLElement) {
  const r = el.getBoundingClientRect();
  const win = el.ownerDocument.defaultView;
  b.style.display = r.width || r.height ? '' : 'none';
  // Bottom-right corner, inside the field.
  b.style.left = `${r.right - 28 + (win?.scrollX ?? 0)}px`;
  b.style.top = `${r.bottom - 26 + (win?.scrollY ?? 0)}px`;
}

function place(node: HTMLElement, r: DOMRect, doc: Document) {
  const win = doc.defaultView;
  node.style.left = `${r.left + (win?.scrollX ?? 0)}px`;
  node.style.top = `${r.top + (win?.scrollY ?? 0)}px`;
}

function visible(r: DOMRect, clip: DOMRect): boolean {
  return r.width > 0 && r.bottom > clip.top && r.top < clip.bottom && r.right > clip.left && r.left < clip.right;
}

// Client rects of [start, end) in the editor's text.
function rectSource(el: HTMLElement): (start: number, end: number) => DOMRect[] {
  if (el instanceof HTMLTextAreaElement || el instanceof HTMLInputElement) return (s, e) => fieldRects(el, s, e);
  const m = textModel(el);
  return (s, e) => {
    const r = rangeFor(m, s, e);
    return r ? Array.from(r.getClientRects()) : [];
  };
}

// A textarea or input has no Range over its text: a hidden mirror div with the same box and font
// lays out the same text, and the rects of the mirror's Range stand in for the field's.
const COPIED = [
  'fontFamily', 'fontSize', 'fontWeight', 'fontStyle', 'fontVariant', 'fontStretch', 'lineHeight',
  'letterSpacing', 'wordSpacing', 'textTransform', 'textIndent', 'textAlign', 'tabSize', 'direction',
  'paddingTop', 'paddingRight', 'paddingBottom', 'paddingLeft',
  'borderTopWidth', 'borderRightWidth', 'borderBottomWidth', 'borderLeftWidth',
] as const;

function fieldRects(el: HTMLTextAreaElement | HTMLInputElement, start: number, end: number): DOMRect[] {
  const doc = el.ownerDocument;
  const root = overlayRoot(doc);
  const mirror = doc.createElement('div');
  mirror.className = 'ptbr-mirror';
  const cs = doc.defaultView?.getComputedStyle(el);
  if (cs) for (const p of COPIED) mirror.style[p] = cs[p];
  const box = el.getBoundingClientRect();
  const borderX = parseFloat(cs?.borderLeftWidth ?? '0') + parseFloat(cs?.borderRightWidth ?? '0');
  const borderY = parseFloat(cs?.borderTopWidth ?? '0') + parseFloat(cs?.borderBottomWidth ?? '0');
  // clientWidth excludes the scrollbar, so the mirror wraps where the field wraps.
  mirror.style.width = `${el.clientWidth + borderX}px`;
  mirror.style.height = `${el.clientHeight + borderY}px`;
  mirror.style.whiteSpace = el instanceof HTMLInputElement ? 'pre' : 'pre-wrap';
  mirror.style.overflowWrap = 'break-word';
  place(mirror, box, doc);
  const text = doc.createTextNode(el.value + '​');
  mirror.appendChild(text);
  root.appendChild(mirror);
  mirror.scrollTop = el.scrollTop;
  mirror.scrollLeft = el.scrollLeft;
  let rects: DOMRect[] = [];
  if (start >= 0 && end > start && end <= el.value.length) {
    const r = doc.createRange();
    r.setStart(text, start);
    r.setEnd(text, end);
    rects = Array.from(r.getClientRects());
  }
  mirror.remove();
  return rects;
}
