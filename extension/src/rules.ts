import type { Level } from './types';

function section(skill: string, heading: string, name: string, file = "SKILL.md"): string {
  const lines = skill.split('\n');
  const start = lines.findIndex((l) => l.trimEnd() === heading);
  if (start < 0) throw new Error(`${file}: seção ${name} não encontrada`);
  let end = lines.findIndex((l, i) => i > start && l.startsWith('## '));
  if (end < 0) end = lines.length;
  return lines.slice(start, end).join('\n').trim();
}

export function buildSystemPrompt(level: Level, skill: string, cientifico: string): string {
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
    ].join('\n'),
  );
  return parts.join('\n\n');
}
