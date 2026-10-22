// @vitest-environment happy-dom
import { afterEach, describe, expect, it, vi } from 'vitest';
import { acceptSuggestion } from '../src/content/apply';
import type { Located } from '../src/types';

const passive: Located = {
  original: 'Foi decidido',
  replacement: 'A equipe decidiu',
  rule: 'PTW-1',
  severity: 'erro',
  occurrence: 1,
  start: 0,
  end: 12,
};

afterEach(() => {
  document.body.innerHTML = '';
  vi.restoreAllMocks();
});

describe('acceptSuggestion', () => {
  it('replaces the span of a textarea and fires one input event', () => {
    const ta = document.createElement('textarea');
    ta.value = 'Foi decidido hoje';
    document.body.appendChild(ta);
    const inputs = vi.fn();
    ta.addEventListener('input', inputs);
    expect(acceptSuggestion(ta, passive, 0)).toBe(true);
    expect(ta.value).toBe('A equipe decidiu hoje');
    expect(inputs).toHaveBeenCalledTimes(1);
  });

  it('applies the offset of the sent text', () => {
    const inp = document.createElement('input');
    inp.value = 'Ontem. Foi decidido hoje';
    document.body.appendChild(inp);
    expect(acceptSuggestion(inp, passive, 7)).toBe(true);
    expect(inp.value).toBe('Ontem. A equipe decidiu hoje');
  });

  it('returns false and changes nothing when the span changed', () => {
    const ta = document.createElement('textarea');
    ta.value = 'Foi decidido hoje';
    document.body.appendChild(ta);
    ta.value = 'Foi resolvido hoje';
    const inputs = vi.fn();
    ta.addEventListener('input', inputs);
    expect(acceptSuggestion(ta, passive, 0)).toBe(false);
    expect(ta.value).toBe('Foi resolvido hoje');
    expect(inputs).not.toHaveBeenCalled();
  });

  it('selects the original in a contenteditable and inserts the replacement', () => {
    const el = document.createElement('div');
    el.setAttribute('contenteditable', 'true');
    el.innerHTML = '<p>Foi <b>decidido</b> hoje</p>';
    document.body.appendChild(el);
    let selected = '';
    const exec = vi.fn(() => {
      selected = document.getSelection()?.toString() ?? '';
      return true;
    });
    (document as unknown as { execCommand: typeof exec }).execCommand = exec;
    expect(acceptSuggestion(el, passive, 0)).toBe(true);
    expect(exec).toHaveBeenCalledWith('insertText', false, 'A equipe decidiu');
    expect(selected).toBe('Foi decidido');
  });

  it('an editor that handles paste itself (WhatsApp) gets the replacement as a paste, not execCommand', () => {
    const el = document.createElement('div');
    el.setAttribute('contenteditable', 'true');
    el.innerHTML = '<p>Foi <b>decidido</b> hoje</p>';
    document.body.appendChild(el);
    let pasted = '';
    let selected = '';
    el.addEventListener('paste', (e) => {
      selected = document.getSelection()?.toString() ?? '';
      pasted = (e as ClipboardEvent).clipboardData?.getData('text/plain') ?? '';
      e.preventDefault();
    });
    const exec = vi.fn(() => true);
    (document as unknown as { execCommand: typeof exec }).execCommand = exec;
    expect(acceptSuggestion(el, passive, 0)).toBe(true);
    expect(pasted).toBe('A equipe decidiu');
    expect(selected).toBe('Foi decidido');
    expect(exec).not.toHaveBeenCalled();
  });
  it('does not call execCommand when the contenteditable text changed', () => {
    const el = document.createElement('div');
    el.setAttribute('contenteditable', 'true');
    el.textContent = 'Foi resolvido hoje';
    document.body.appendChild(el);
    const exec = vi.fn(() => true);
    (document as unknown as { execCommand: typeof exec }).execCommand = exec;
    expect(acceptSuggestion(el, passive, 0)).toBe(false);
    expect(exec).not.toHaveBeenCalled();
  });
});
