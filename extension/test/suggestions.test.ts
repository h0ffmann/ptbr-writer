import { describe, expect, it } from 'vitest';
import { applyToString, dropOverlaps, filterSuggestions, locate, parseSuggestions } from '../src/suggestions';
import type { Located, Suggestion } from '../src/types';

const sug = (original: string, occurrence = 1, replacement = 'x'): Suggestion => ({
  original,
  replacement,
  rule: 'PTW-1',
  severity: 'erro',
  occurrence,
});

describe('parseSuggestions', () => {
  it('tira cercas e preenche occurrence', () => {
    const raw =
      '```json\n[{"original":"Foi decidido","replacement":"A equipe decidiu","rule":"PTW-1","severity":"erro"}]\n```';
    expect(parseSuggestions(raw)).toEqual([
      { original: 'Foi decidido', replacement: 'A equipe decidiu', rule: 'PTW-1', severity: 'erro', occurrence: 1 },
    ]);
  });
  it('prosa com lista vazia', () => {
    expect(parseSuggestions('Aqui está: [] ok')).toEqual([]);
  });
  it('JSON inválido', () => {
    expect(parseSuggestions('não é json')).toEqual([]);
    expect(parseSuggestions('[{oops]')).toEqual([]);
  });
  it('descarta severity desconhecida e campos faltando', () => {
    const raw = JSON.stringify([
      { original: 'a', replacement: 'b', rule: 'R', severity: 'info' },
      { original: '', replacement: 'b', rule: 'R', severity: 'erro' },
      { original: 'a', replacement: 1, rule: 'R', severity: 'erro' },
      { original: 'a', replacement: 'b', rule: 'R', severity: 'aviso' },
    ]);
    expect(parseSuggestions(raw)).toHaveLength(1);
  });
  it('occurrence inválida vira 1', () => {
    const mk = (o: unknown) =>
      parseSuggestions(JSON.stringify([{ original: 'a', replacement: 'b', rule: 'R', severity: 'erro', occurrence: o }]))[0]
        .occurrence;
    expect(mk(0)).toBe(1);
    expect(mk(1.5)).toBe(1);
    expect(mk('2')).toBe(1);
    expect(mk(3)).toBe(3);
  });
});

describe('locate', () => {
  it('acha a ocorrência pedida', () => {
    const r = locate('a nível de x e a nível de y', [sug('a nível de', 2)]);
    expect(r[0].start).toBe(15);
    expect(r[0].end).toBe(25);
  });
  it('descarta ausente, igual e ocorrência inexistente', () => {
    expect(locate('foo bar', [sug('baz')])).toEqual([]);
    expect(locate('foo bar', [sug('foo', 1, 'foo')])).toEqual([]);
    expect(locate('foo bar', [sug('foo', 2)])).toEqual([]);
  });
  it('ordena por start', () => {
    const r = locate('foo bar', [sug('bar'), sug('foo')]);
    expect(r.map((l) => l.start)).toEqual([0, 4]);
    expect(r.map((l) => l.original)).toEqual(['foo', 'bar']);
  });
  it('conta em UTF-16 com emoji', () => {
    const r = locate('Oi 👋 foi decidido', [sug('foi decidido')]);
    expect(r[0].start).toBe(6);
  });
  it('casa em NFC e devolve posições do texto original (NFD)', () => {
    const nfd = 'ola é muito bom'; // "é" decomposto
    const r = locate(nfd, [sug('é muito')]);
    expect(r).toHaveLength(1);
    expect(r[0].start).toBe(4);
    expect(nfd.slice(r[0].start, r[0].end)).toBe('é muito');
    const nfc = locate('ola é muito bom', [sug('é muito')]);
    expect(nfc[0].start).toBe(4);
    expect(nfc[0].end).toBe(11);
  });
});

describe('dropOverlaps', () => {
  it('mantém o primeiro de cada sobreposição', () => {
    const l = (start: number, end: number): Located => ({ ...sug('x'), start, end });
    const r = dropOverlaps([l(0, 12), l(4, 20), l(20, 25)]);
    expect(r.map((x) => [x.start, x.end])).toEqual([[0, 12], [20, 25]]);
  });
});

describe('applyToString', () => {
  it('aplica a troca', () => {
    const l: Located = { ...sug('Foi decidido', 1, 'A equipe decidiu'), start: 0, end: 12 };
    expect(applyToString('Foi decidido hoje', l)).toBe('A equipe decidiu hoje');
  });
});

describe('filterSuggestions', () => {
  const text = 'bomd ia tudo bem com vc. O relatório está pronto';
  const mk = (original: string, replacement: string, severity: 'erro' | 'aviso'): Suggestion => ({
    original,
    replacement,
    rule: 'PTW-1',
    severity,
    occurrence: 1,
  });
  const aviso = mk('bomd ia tudo bem com vc.', 'Bom dia, tudo bem com você.', 'aviso');
  const erro = mk('vc', 'você', 'erro');
  const on = { rewrite: true, neverFix: [] as string[] };

  it('erro dentro de aviso: fica só o aviso', () => {
    expect(filterSuggestions(text, [aviso, erro], on)).toEqual([aviso]);
  });
  it('só erros: fica só o erro', () => {
    expect(filterSuggestions(text, [aviso, erro], { ...on, rewrite: false })).toEqual([erro]);
  });
  it('aviso não achado: o erro fica', () => {
    const lost = mk('bomd  ia', 'Bom dia', 'aviso');
    expect(filterSuggestions(text, [lost, erro], on)).toEqual([erro]);
  });
  it('lista: some o erro e o aviso que trocam a entrada', () => {
    const keep = mk('O relatório está pronto', 'O relatório está pronto, vc', 'aviso');
    const drop = mk('bomd ia tudo bem com vc.', 'Bom dia, tudo bem.', 'aviso');
    expect(filterSuggestions(text, [erro], { rewrite: true, neverFix: ['vc'] })).toEqual([]);
    expect(filterSuggestions(text, [drop], { rewrite: true, neverFix: ['vc'] })).toEqual([]);
    expect(filterSuggestions(text, [keep], { rewrite: true, neverFix: ['vc'] })).toEqual([keep]);
  });
  it('sobreposição parcial: fica só o primeiro, sem erro', () => {
    const a = mk(text.slice(0, 10), 'A', 'aviso');
    const e = mk(text.slice(5, 15), 'E', 'erro');
    expect(filterSuggestions(text, [a, e], on)).toEqual([a]);
  });
});
