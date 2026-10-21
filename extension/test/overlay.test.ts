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
