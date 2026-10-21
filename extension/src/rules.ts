import type { Level, Tone } from './types';

export interface PromptOptions {
  tone: Tone;
  rewrite: boolean;
  neverFix: readonly string[];
}

const TONE_LINES: Record<Tone, string> = {
  informal: 'Tom informal: mantenha a voz e as gírias da pessoa; só corrija e esclareça.',
  neutro: 'Tom neutro: claro e direto, nem coloquial nem cerimonioso.',
  formal: 'Tom formal: registro profissional, sem gíria.',
};

// Lê os cabeçalhos `### PTW-n: Título` e devolve { 'PTW-n': 'Título' }.
export function ruleTitles(skill: string): Record<string, string> {
  const titles: Record<string, string> = {};
  for (const line of skill.split('\n')) {
    const m = /^### ([A-Z]+-\d+): (.+?)\s*$/.exec(line);
    if (m) titles[m[1]] = m[2];
  }
  return titles;
}

function section(skill: string, heading: string, name: string, file = "SKILL.md"): string {
  const lines = skill.split('\n');
  const start = lines.findIndex((l) => l.trimEnd() === heading);
  if (start < 0) throw new Error(`${file}: seção ${name} não encontrada`);
  let end = lines.findIndex((l, i) => i > start && l.startsWith('## '));
  if (end < 0) end = lines.length;
  return lines.slice(start, end).join('\n').trim();
}

export function buildSystemPrompt(level: Level, skill: string, cientifico: string, opts: PromptOptions): string {
  const parts = [
    'Você revisa texto em português do Brasil segundo as regras abaixo. Aponte só trechos que quebram uma regra.',
    section(skill, '## Níveis de rigor', 'Níveis de rigor'),
    section(skill, '## Regras (em desenvolvimento)', 'Regras'),
  ];
  if (level === 'científico') parts.push(section(cientifico, "## Regras", "Regras", "cientifico/SKILL.md"));
  parts.push(
    `Nível pedido: \`${level}\`. Aplique as regras com o rigor desse nível.`,
    [
      'Este formato substitui qualquer outro formato de saída citado nas regras acima.',
      'Formato da resposta: responda só com um array JSON, sem texto fora dele. Cada item é um objeto',
      '{"original", "replacement", "rule", "severity", "occurrence"}:',
      '- original: cópia exata do trecho do texto, sem alterar um caractere;',
      '- replacement: o trecho reescrito;',
      '- rule: o código da regra quebrada (por exemplo PTW-3);',
      '- severity: "erro" ou "aviso";',
      '- occurrence: qual ocorrência do trecho no texto, contando a partir de 1.',
      'Texto sem problemas: responda []. Não reescreva nada fora dos trechos apontados.',
      'Severidade: `erro` para as regras PTW-8 e PTW-14 a PTW-16; `aviso` só para a PTW-17.',
      'Um `aviso` cobre a frase inteira e já traz todas as correções dela; nenhum `erro` cai dentro de um `aviso`.',
      'Copie o original letra por letra, sem normalizar espaços nem aspas.',
      TONE_LINES[opts.tone],
      ...(opts.rewrite ? [] : ['Só erros: não devolva nenhum aviso; não reorganize frases.']),
      ...(opts.neverFix.length ? [`Palavras da pessoa: nunca troque estas palavras: ${opts.neverFix.join(', ')}.`] : []),
    ].join('\n'),
  );
  return parts.join('\n\n');
}
