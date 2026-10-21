import { describe, expect, it } from 'vitest';
import { NEVER_FIX_MAX, addNeverFix, breaksEntry, containsEntry } from '../src/never-fix';

describe('addNeverFix', () => {
  it('trims the ends', () => {
    expect(addNeverFix([], '  Kafka ')).toEqual(['Kafka']);
  });
  it('does not repeat, ignoring case', () => {
    expect(addNeverFix(['Kafka'], 'kafka')).toEqual(['Kafka']);
  });
  it('ignores empty input', () => {
    expect(addNeverFix([], '   ')).toEqual([]);
  });
  it('ignores input with more than 3 words', () => {
    expect(addNeverFix([], 'um dois três quatro')).toEqual([]);
    expect(addNeverFix([], 'um dois três')).toEqual(['um dois três']);
  });
  it('keeps the 200 most recent', () => {
    const list = Array.from({ length: NEVER_FIX_MAX }, (_, i) => `w${i}`);
    const out = addNeverFix(list, 'novo');
    expect(out).toHaveLength(NEVER_FIX_MAX);
    expect(out[0]).toBe('w1');
    expect(out[out.length - 1]).toBe('novo');
  });
});

describe('containsEntry', () => {
  it('whole word, ignoring case', () => {
    expect(containsEntry('use a api nova', 'API')).toBe(true);
  });
  it('does not match inside a word', () => {
    expect(containsEntry('a capital', 'api')).toBe(false);
  });
  it('tells accents apart', () => {
    expect(containsEntry('ação rápida', 'acao')).toBe(false);
    expect(containsEntry('ação rápida', 'ação')).toBe(true);
  });
});

describe('breaksEntry', () => {
  it('a change of case alone breaks the entry', () => {
    expect(breaksEntry('maria', 'Maria', 'maria')).toBe(true);
    expect(breaksEntry('use o github', 'use o GitHub', 'github')).toBe(true);
  });
  it('keeps the text as written, whatever the case of the entry', () => {
    expect(breaksEntry('maria', 'maria,', 'maria')).toBe(false);
    expect(breaksEntry('o GitHub caiu', 'o GitHub caiu.', 'github')).toBe(false);
  });
  it('needs a whole word in the replacement', () => {
    expect(breaksEntry('api', 'apis', 'api')).toBe(true);
  });
  it('an original without the entry breaks nothing', () => {
    expect(breaksEntry('a capital', 'A capital', 'api')).toBe(false);
  });
});
