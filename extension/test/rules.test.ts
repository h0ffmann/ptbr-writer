import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { buildSystemPrompt } from '../src/rules';

const skill = readFileSync('../SKILL.md', 'utf8');
const cientifico = readFileSync('../skills/cientifico/SKILL.md', 'utf8');

describe('buildSystemPrompt', () => {
  it('inclui PTW-1 a PTW-13', () => {
    const prompt = buildSystemPrompt('estrito', skill, cientifico);
    for (let n = 1; n <= 13; n++) expect(prompt).toContain(`### PTW-${n}`);
  });

  it('exclui Uso, Smart note e Formato de saída', () => {
    const prompt = buildSystemPrompt('estrito', skill, cientifico);
    expect(prompt).not.toContain('## Uso');
    expect(prompt).not.toContain('## Smart note');
    expect(prompt).not.toContain('## Formato de saída');
  });

  it('científico embute CIE', () => {
    expect(buildSystemPrompt('científico', skill, cientifico)).toContain('### CIE-1');
    expect(buildSystemPrompt('leve', skill, cientifico)).not.toContain('### CIE-1');
  });

  it('não vaza o front matter do científico', () => {
    expect(buildSystemPrompt('científico', skill, cientifico)).not.toContain('name: cientifico');
  });

  it('declara o nível', () => {
    expect(buildSystemPrompt('leve', skill, cientifico)).toContain('`leve`');
  });

  it('falha se a seção some', () => {
    const semRegras = skill.replace('## Regras (em desenvolvimento)', '## Outra');
    expect(() => buildSystemPrompt('leve', semRegras, cientifico)).toThrow('SKILL.md: seção Regras não encontrada');
    const semNiveis = skill.replace('## Níveis de rigor', '## Outra');
    expect(() => buildSystemPrompt('leve', semNiveis, cientifico)).toThrow('SKILL.md: seção Níveis de rigor não encontrada');
  });
});
