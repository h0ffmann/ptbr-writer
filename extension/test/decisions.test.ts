import { describe, expect, it } from 'vitest';
import { THRESHOLD, decideAsks, readDecision, readStyle, rulesOf, sentences, styleBody } from '../src/decisions';
import { CIENTIFICO, SKILL } from '../src/skills';

const TEXT = 'Foi decidido que o deploy será feito. A equipe valida o build hoje.\nFoi decidido que o deploy será feito.';

describe('rulesOf', () => {
  it('reads PTW-1 to PTW-13 with their titles', () => {
    const r = rulesOf(SKILL, '## Regras (em desenvolvimento)');
    expect(Object.keys(r)).toEqual(Array.from({ length: 13 }, (_, i) => `PTW-${i + 1}`));
    expect(r['PTW-1']).toBe('Quem faz, aparece');
  });
});

describe('sentences', () => {
  it('splits after . ! ? and at line breaks, and drops fragments under 3 words', () => {
    expect(sentences('Oi. Isto é uma frase! E esta outra?\nok sim')).toEqual(['Isto é uma frase!', 'E esta outra?']);
  });
});

describe('decideAsks', () => {
  it('one body per sentence: the sentence is the state, one choice question, nenhuma first', () => {
    const asks = decideAsks('decima', 'estrito', TEXT, SKILL, CIENTIFICO);
    expect(asks.map((a) => [a.sentence, a.occurrence])).toEqual([
      ['Foi decidido que o deploy será feito.', 1],
      ['A equipe valida o build hoje.', 1],
      ['Foi decidido que o deploy será feito.', 2],
    ]);
    const body = asks[1].body as any;
    expect(body.model).toBe('decima');
    expect(body.state).toBe('A equipe valida o build hoje.');
    expect(body.questions.regra.type).toBe('choice');
    expect(Object.keys(body.questions.regra.criteria)[0]).toBe('nenhuma');
    expect(Object.keys(body.questions.regra.criteria)).toHaveLength(14);
  });
  it('científico adds the CIE rules', () => {
    const body = decideAsks('decima', 'científico', TEXT, SKILL, CIENTIFICO)[0].body as any;
    expect(body.questions.regra.criteria['CIE-1']).toBeTruthy();
  });
});

describe('readDecision', () => {
  const ask = decideAsks('decima', 'estrito', TEXT, SKILL, CIENTIFICO)[2];
  const reply = (choice: string, p: number) => ({ answers: { regra: { type: 'choice', choice, probabilities: { [choice]: p } } } });
  it('a rule above the threshold becomes a detection at its occurrence', () => {
    expect(readDecision(reply('PTW-1', 0.82), ask)).toEqual({
      original: 'Foi decidido que o deploy será feito.',
      replacement: '',
      detectOnly: true,
      rule: 'PTW-1 (82%)',
      severity: 'aviso',
      occurrence: 2,
    });
  });
  it('nenhuma, a weak answer or an unexpected body give nothing', () => {
    expect(readDecision(reply('nenhuma', 0.9), ask)).toBeNull();
    expect(readDecision(reply('PTW-1', THRESHOLD - 0.01), ask)).toBeNull();
    expect(readDecision(null, ask)).toBeNull();
    expect(readDecision({ answers: { regra: { choice: 3 } } }, ask)).toBeNull();
  });
});

describe('style (Ollaya + Ollama)', () => {
  it('asks one choice question over the whole field, the styles as options', () => {
    const body = styleBody('decima', TEXT) as any;
    expect(body.state).toBe(TEXT);
    expect(Object.keys(body.questions.estilo.criteria)).toEqual(['conciso', 'socrático', 'amigável']);
  });
  it('reads the chosen style; anything else is null', () => {
    expect(readStyle({ answers: { estilo: { choice: 'socrático' } } })).toBe('socrático');
    expect(readStyle({ answers: { estilo: { choice: 'toString' } } })).toBeNull();
    expect(readStyle(null)).toBeNull();
  });
});
