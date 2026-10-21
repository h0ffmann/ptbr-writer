// @vitest-environment happy-dom
import { afterEach, describe, expect, it, vi } from 'vitest';
import { overlayRoot, renderBadge, renderSuggestions } from '../src/content/overlay';
import type { Located } from '../src/types';

const erro: Located = {
  original: 'Foi decidido',
  replacement: 'A equipe decidiu',
  rule: 'PTW-1',
  severity: 'erro',
  occurrence: 1,
  start: 0,
  end: 12,
};
const aviso: Located = {
  original: 'hoje',
  replacement: 'nesta terça',
  rule: 'PTW-2',
  severity: 'aviso',
  occurrence: 1,
  start: 13,
  end: 17,
};

function editable(): HTMLElement {
  const el = document.createElement('div');
  el.setAttribute('contenteditable', 'true');
  el.textContent = 'Foi decidido hoje';
  document.body.appendChild(el);
  return el;
}

afterEach(() => {
  document.body.innerHTML = '';
  document.getElementById('ptbr-writer-root')?.remove();
});

describe('renderSuggestions', () => {
  it('draws one underline per item in the shadow root and nothing inside the editor', () => {
    const el = editable();
    const before = el.innerHTML;
    renderSuggestions(el, [erro, aviso], 0, vi.fn(), vi.fn());
    const root = document.getElementById('ptbr-writer-root')?.shadowRoot;
    expect(root).toBe(overlayRoot());
    expect(root?.querySelectorAll('.ptbr-underline-erro').length).toBe(1);
    expect(root?.querySelectorAll('.ptbr-underline-aviso').length).toBe(1);
    expect(el.innerHTML).toBe(before);
    expect(el.querySelector('[class^="ptbr-"]')).toBeNull();
  });

  it('also draws for a textarea', () => {
    const ta = document.createElement('textarea');
    ta.value = 'Foi decidido hoje';
    document.body.appendChild(ta);
    renderSuggestions(ta, [erro], 0, vi.fn(), vi.fn());
    expect(overlayRoot().querySelectorAll('.ptbr-underline-erro').length).toBe(1);
    expect(ta.value).toBe('Foi decidido hoje');
  });

  it('opens a card on click and calls onAccept with the item', () => {
    const el = editable();
    const onAccept = vi.fn();
    const onIgnore = vi.fn();
    renderSuggestions(el, [erro, aviso], 0, onAccept, onIgnore);
    const root = overlayRoot();
    (root.querySelector('.ptbr-underline-erro') as HTMLElement).click();
    const card = root.querySelector('.ptbr-card') as HTMLElement;
    expect(card.textContent).toContain('PTW-1');
    expect(card.querySelector('s')?.textContent).toBe('Foi decidido');
    expect(card.querySelector('strong')?.textContent).toBe('A equipe decidiu');
    expect(card.querySelector('[data-action="accept"]')?.textContent).toBe('Aceitar');
    expect(card.querySelector('[data-action="ignore"]')?.textContent).toBe('Ignorar');
    (card.querySelector('[data-action="accept"]') as HTMLElement).click();
    expect(onAccept).toHaveBeenCalledWith(erro);
    expect(onIgnore).not.toHaveBeenCalled();
    expect(root.querySelector('.ptbr-card')).toBeNull();
  });

  it('calls onIgnore from the card', () => {
    const el = editable();
    const onIgnore = vi.fn();
    renderSuggestions(el, [aviso], 0, vi.fn(), onIgnore);
    const root = overlayRoot();
    (root.querySelector('.ptbr-underline-aviso') as HTMLElement).click();
    (root.querySelector('[data-action="ignore"]') as HTMLElement).click();
    expect(onIgnore).toHaveBeenCalledWith(aviso);
  });

  it('keeps focus in the editor on mousedown over an underline', () => {
    const el = editable();
    renderSuggestions(el, [erro], 0, vi.fn(), vi.fn());
    const ev = new MouseEvent('mousedown', { bubbles: true, cancelable: true });
    overlayRoot().querySelector('.ptbr-underline-erro')?.dispatchEvent(ev);
    expect(ev.defaultPrevented).toBe(true);
  });

  it('replaces the previous underlines of the same editor', () => {
    const el = editable();
    renderSuggestions(el, [erro, aviso], 0, vi.fn(), vi.fn());
    renderSuggestions(el, [aviso], 0, vi.fn(), vi.fn());
    expect(overlayRoot().querySelectorAll('.ptbr-underline-erro').length).toBe(0);
    expect(overlayRoot().querySelectorAll('.ptbr-underline-aviso').length).toBe(1);
  });
});

describe('renderBadge', () => {
  it('shows the label of an error', () => {
    const el = editable();
    renderBadge(el, 'rate-limit', 0);
    expect(overlayRoot().querySelector('.ptbr-badge')?.textContent).toBe('limite atingido');
  });

  it('shows the other labels and the pending count', () => {
    const el = editable();
    const label = () => overlayRoot().querySelector('.ptbr-badge')?.textContent;
    renderBadge(el, 'no-key', 0);
    expect(label()).toBe('sem chave');
    renderBadge(el, 'auth', 0);
    expect(label()).toBe('chave inválida');
    renderBadge(el, 'provider', 0);
    expect(label()).toBe('erro do provider');
    renderBadge(el, 'network', 0);
    expect(label()).toBe('erro do provider');
    renderBadge(el, 'idle', 2);
    expect(label()).toBe('2');
    expect(overlayRoot().querySelectorAll('.ptbr-badge').length).toBe(1);
  });
});

describe('hit strip', () => {
  it('only a 4 px strip at the bottom of a segment takes the mouse', () => {
    const el = editable();
    vi.spyOn(el, 'getBoundingClientRect').mockReturnValue(new DOMRect(0, 0, 400, 100));
    vi.spyOn(Range.prototype, 'getClientRects').mockReturnValue([new DOMRect(10, 10, 80, 18)] as never);
    const onAccept = vi.fn();
    renderSuggestions(el, [erro], 0, onAccept, vi.fn());
    const seg = overlayRoot().querySelector('.ptbr-seg') as HTMLElement;
    expect(seg).toBeTruthy();
    expect(seg.style.pointerEvents).toBe('none');
    const hit = seg.querySelector('.ptbr-hit') as HTMLElement;
    expect(hit.style.pointerEvents).toBe('auto');
    expect(hit.style.height).toBe('4px');
    expect(hit.style.bottom).toBe('0px');
    const down = new MouseEvent('mousedown', { bubbles: true, cancelable: true });
    hit.dispatchEvent(down);
    expect(down.defaultPrevented).toBe(true);
    hit.click();
    expect(overlayRoot().querySelector('.ptbr-card')).not.toBeNull();
    vi.restoreAllMocks();
  });
});

describe('card placement and badge', () => {
  const box = (top: number, bottom: number) =>
    ({ top, bottom, left: 10, right: 400, width: 390, height: bottom - top, x: 10, y: top, toJSON() {} }) as DOMRect;
  const cardHeight = (h: number) => {
    const d = Object.getOwnPropertyDescriptor(HTMLElement.prototype, 'offsetHeight');
    Object.defineProperty(HTMLElement.prototype, 'offsetHeight', {
      configurable: true,
      get() {
        return (this as HTMLElement).classList.contains('ptbr-card') ? h : 0;
      },
    });
    return () => d && Object.defineProperty(HTMLElement.prototype, 'offsetHeight', d);
  };
  const card = () => overlayRoot().querySelector('.ptbr-card') as HTMLElement | null;

  it('opens the card above the text when there is no room below (composer at the bottom)', () => {
    const restore = cardHeight(100);
    Object.defineProperty(window, 'innerHeight', { configurable: true, value: 768 });
    const el = editable();
    el.getBoundingClientRect = () => box(700, 740);
    renderSuggestions(el, [erro], 0, vi.fn(), vi.fn());
    (overlayRoot().querySelector('.ptbr-underline-erro') as HTMLElement).click();
    expect(card()?.style.top).toBe('596px');
    restore();
  });

  it('keeps the card below the text when it fits', () => {
    const restore = cardHeight(100);
    Object.defineProperty(window, 'innerHeight', { configurable: true, value: 768 });
    const el = editable();
    el.getBoundingClientRect = () => box(100, 120);
    renderSuggestions(el, [erro], 0, vi.fn(), vi.fn());
    (overlayRoot().querySelector('.ptbr-underline-erro') as HTMLElement).click();
    expect(card()?.style.top).toBe('124px');
    restore();
  });

  it('clicking the badge opens the card of the first suggestion', () => {
    const el = editable();
    renderSuggestions(el, [erro, aviso], 0, vi.fn(), vi.fn());
    renderBadge(el, 'idle', 2);
    (overlayRoot().querySelector('.ptbr-badge') as HTMLElement).click();
    expect(card()?.textContent).toContain('A equipe decidiu');
  });

  it('clicking the badge with no suggestions opens nothing', () => {
    const el = editable();
    renderSuggestions(el, [], 0, vi.fn(), vi.fn());
    renderBadge(el, 'idle', 0);
    (overlayRoot().querySelector('.ptbr-badge') as HTMLElement).click();
    expect(card()).toBeNull();
  });
});

describe('hover', () => {
  const rect = (left: number, top: number, width: number, height: number) =>
    ({ left, top, width, height, right: left + width, bottom: top + height, x: left, y: top, toJSON() {} }) as DOMRect;
  const move = (x: number, y: number, target: EventTarget = document.body) =>
    target.dispatchEvent(new MouseEvent('mousemove', { clientX: x, clientY: y, bubbles: true, composed: true }));
  const card = () => overlayRoot().querySelector('.ptbr-card') as HTMLElement | null;

  // The word "Foi decidido" sits at x 10..90, y 10..30 of a field at 0..400 x 0..100.
  function setup() {
    vi.useFakeTimers();
    vi.spyOn(Range.prototype, 'getClientRects').mockReturnValue([rect(10, 10, 80, 20)] as unknown as DOMRectList);
    const el = editable();
    el.getBoundingClientRect = () => rect(0, 0, 400, 100);
    renderSuggestions(el, [erro], 0, vi.fn(), vi.fn());
    return el;
  }
  afterEach(() => {
    vi.useRealTimers();
    vi.restoreAllMocks();
  });

  it('resting the mouse on an underlined word opens its card', () => {
    setup();
    move(40, 20);
    vi.advanceTimersByTime(150);
    expect(card()).toBeNull();
    vi.advanceTimersByTime(100);
    expect(card()?.querySelector('strong')?.textContent).toBe('A equipe decidiu');
  });

  it('the hover card closes when the mouse leaves, and stays while the mouse is on the card', () => {
    setup();
    move(40, 20);
    vi.advanceTimersByTime(250);
    move(40, 60, card()!);
    vi.advanceTimersByTime(1000);
    expect(card()).not.toBeNull();
    move(300, 90);
    vi.advanceTimersByTime(350);
    expect(card()).toBeNull();
  });

  it('a card opened by a click does not close when the mouse leaves', () => {
    setup();
    (overlayRoot().querySelector('.ptbr-underline-erro') as HTMLElement).click();
    move(300, 90);
    vi.advanceTimersByTime(1000);
    expect(card()).not.toBeNull();
  });

  it('the mouse over plain text opens nothing', () => {
    setup();
    move(300, 90);
    vi.advanceTimersByTime(1000);
    expect(card()).toBeNull();
  });
});
