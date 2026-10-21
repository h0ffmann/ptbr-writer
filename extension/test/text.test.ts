// @vitest-environment happy-dom
import { describe, expect, it } from 'vitest';
import { rangeFor, selectForSending, textModel, wordCount } from '../src/content/text';

function editable(html: string): HTMLElement {
  const el = document.createElement('div');
  el.setAttribute('contenteditable', 'true');
  el.innerHTML = html;
  document.body.appendChild(el);
  return el;
}

describe('textModel', () => {
  it('skips quoted subtrees', () => {
    const el = editable('<p>Oi</p><div class="gmail_quote">antigo</div><blockquote>velho</blockquote>');
    expect(textModel(el).text).toBe('Oi');
  });
  it('separates blocks with a synthetic newline that belongs to no node', () => {
    const el = editable('<div>a b</div><div>c d</div>');
    const m = textModel(el);
    expect(m.text).toBe('a b\nc d');
    expect(m.nodes.map((n) => n.start)).toEqual([0, 4]);
  });
  it('a <br> is a newline too', () => {
    expect(textModel(editable('<p>x<br>y</p>')).text).toBe('x\ny');
  });
  it('no newline before the first text, nor inside inline elements', () => {
    expect(textModel(editable('<div><p>um <b>dois</b></p></div>')).text).toBe('um dois');
  });
  it('uses value for textarea and input', () => {
    const ta = document.createElement('textarea');
    ta.value = 'a\nb';
    expect(textModel(ta)).toEqual({ text: 'a\nb', nodes: [] });
    const inp = document.createElement('input');
    inp.value = 'x y';
    expect(textModel(inp).text).toBe('x y');
  });
});

describe('selectForSending', () => {
  it('sends the whole text when it fits', () => {
    expect(selectForSending('a'.repeat(4000), 10)).toEqual({ text: 'a'.repeat(4000), offset: 0 });
  });
  it('sends the paragraph containing the caret', () => {
    const p1 = 'a'.repeat(1500);
    const p2 = 'b'.repeat(2000);
    const p3 = 'c'.repeat(1500);
    const text = [p1, p2, p3].join('\n');
    expect(text.length).toBeGreaterThan(4000);
    const start = p1.length + 1;
    expect(selectForSending(text, start + 10)).toEqual({ text: p2, offset: start });
  });
  it('windows a paragraph over 4000 around the caret', () => {
    const text = 'x'.repeat(10000);
    expect(selectForSending(text, 5000)).toEqual({ text: text.slice(3000, 7000), offset: 3000 });
    expect(selectForSending(text, 100)).toEqual({ text: text.slice(0, 4000), offset: 0 });
    expect(selectForSending(text, 9900)).toEqual({ text: text.slice(6000, 10000), offset: 6000 });
  });
  it('with the caret at 0, sends the first paragraph even when it is empty', () => {
    const text = '\n' + 'a '.repeat(2500);
    expect(selectForSending(text, 0)).toEqual({ text: '', offset: 0 });
  });
  it('sends a contenteditable paragraph', () => {
    const m = textModel(editable('<div>' + 'a '.repeat(1500) + '</div><div>c d</div><div>' + 'e '.repeat(1500) + '</div>'));
    expect(m.text.length).toBeGreaterThan(4000);
    const at = m.text.indexOf('c d');
    expect(selectForSending(m.text, at + 1)).toEqual({ text: 'c d', offset: at });
  });
  it('windows inside a later paragraph', () => {
    const text = 'a\n' + 'y'.repeat(6000);
    const r = selectForSending(text, 2 + 100);
    expect(r.offset).toBe(2);
    expect(r.text).toBe(text.slice(2, 4002));
  });
});

describe('rangeFor', () => {
  it('maps UTF-16 offsets after an emoji', () => {
    const el = editable('<p>😀 olá mundo</p>');
    const m = textModel(el);
    const start = m.text.indexOf('mundo');
    expect(rangeFor(m, start, start + 5)?.toString()).toBe('mundo');
  });
  it('spans several text nodes', () => {
    const el = editable('<p>um <b>dois</b> três</p>');
    const m = textModel(el);
    expect(rangeFor(m, 1, 9)?.toString()).toBe('m dois t');
  });
  it('returns null across a block separator', () => {
    const m = textModel(editable('<div>a b</div><div>c d</div>'));
    expect(rangeFor(m, 2, 5)).toBeNull();
    expect(rangeFor(m, 3, 4)).toBeNull();
    expect(rangeFor(m, 4, 7)?.toString()).toBe('c d');
  });
  it('returns null out of bounds', () => {
    const m = textModel(editable('<p>oi</p>'));
    expect(rangeFor(m, 5, 9)).toBeNull();
  });
});

describe('wordCount', () => {
  it('counts whitespace-separated words', () => {
    expect(wordCount('')).toBe(0);
    expect(wordCount('  um  dois\ntrês ')).toBe(3);
  });
});
