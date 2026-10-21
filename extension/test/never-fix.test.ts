import { describe, expect, it } from 'vitest';
import { NEVER_FIX_MAX, addNeverFix, containsEntry } from '../src/never-fix';

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
