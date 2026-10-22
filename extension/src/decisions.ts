import { STYLES, section } from './rules';
import type { Level, Style, Suggestion } from './types';

// Ollaya answers typed questions with probabilities and never writes text, so a rule broken is a
// decision: for each sentence, "which rule does it break?", with `nenhuma` as one of the answers.
// An answer other than `nenhuma` becomes an underline with no replacement (the card has no Aceitar).
// The sentence is the state and each option is only `PTW-n: title`: a decision model reads a short
// question and short options (docs/SURVEY_DECISION_MODEL.md).

const NONE = 'nenhuma';
// shortcut: one threshold for every model, not tuned on pt-BR (⚠ não medido); tune per model with evals/.
export const THRESHOLD = 0.5;
// One request per sentence: a field longer than this is checked in its first sentences.
const MAX_SENTENCES = 32;

// `### PTW-1: Quem faz, aparece` → { 'PTW-1': 'Quem faz, aparece' }, from the Regras section of a skill.
export function rulesOf(skill: string, heading: string, file?: string): Record<string, string> {
  const out: Record<string, string> = {};
  for (const m of section(skill, heading, 'Regras', file).matchAll(/^### ([A-Z]+-\d+): (.+)$/gm)) out[m[1]] = m[2];
  return out;
}

// The sentences of `text`, as they appear in it: split after . ! ? … and at line breaks.
export function sentences(text: string): string[] {
  return text
    .split(/(?<=[.!?…])\s+|\n+/)
    .map((s) => s.trim())
    .filter((s) => s.split(/\s+/).length >= 3)
    .slice(0, MAX_SENTENCES);
}

export interface Ask {
  sentence: string;
  occurrence: number; // 1-based, among the identical sentences of the text
  body: Record<string, unknown>;
}

// One /api/decide body per sentence of `text`.
export function decideAsks(model: string, level: Level, text: string, skill: string, cientifico: string): Ask[] {
  const rules = rulesOf(skill, '## Regras (em desenvolvimento)');
  if (level === 'científico') Object.assign(rules, rulesOf(cientifico, '## Regras', 'cientifico/SKILL.md'));
  const criteria = { [NONE]: 'nenhuma regra quebrada', ...rules };
  const seen = new Map<string, number>();
  return sentences(text).map((sentence) => {
    const occurrence = (seen.get(sentence) ?? 0) + 1;
    seen.set(sentence, occurrence);
    const question = { type: 'choice', instructions: `Qual regra de escrita técnica (nível ${level}) esta frase quebra?`, criteria };
    return { sentence, occurrence, body: { model, state: sentence, questions: { regra: question }, keep_alive: '10m' } };
  });
}

// The underline for one sentence from its /api/decide response: its most likely rule, when that is
// not `nenhuma` and passes THRESHOLD.
export function readDecision(json: unknown, ask: Ask): Suggestion | null {
  const a = (json as { answers?: { regra?: { choice?: unknown; probabilities?: Record<string, unknown> } } } | null)?.answers
    ?.regra;
  if (typeof a?.choice !== 'string' || a.choice === NONE) return null;
  const p = a.probabilities?.[a.choice];
  if (typeof p !== 'number' || p < THRESHOLD) return null;
  return {
    original: ask.sentence,
    replacement: '',
    detectOnly: true,
    rule: `${a.choice} (${Math.round(p * 100)}%)`,
    severity: 'aviso',
    occurrence: ask.occurrence,
  };
}

// Ollaya + Ollama: before Ollama writes, Ollaya reads the whole field and picks the conversation
// style of the rewrite, one of STYLES (rules.ts); their descriptions are the options the model weighs.
export function styleBody(model: string, text: string): Record<string, unknown> {
  const question = { type: 'choice', instructions: 'Qual estilo de conversa a reescrita deste texto deve seguir?', criteria: STYLES };
  return { model, state: text, questions: { estilo: question }, keep_alive: '10m' };
}

// The style Ollaya chose; null when the reply is not one of STYLES.
export function readStyle(json: unknown): Style | null {
  const c = (json as { answers?: { estilo?: { choice?: unknown } } } | null)?.answers?.estilo?.choice;
  return typeof c === 'string' && Object.keys(STYLES).includes(c) ? (c as Style) : null;
}
