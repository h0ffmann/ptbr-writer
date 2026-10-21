import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { buildSystemPrompt } from '../src/rules';

const skill = readFileSync(fileURLToPath(new URL('../../SKILL.md', import.meta.url)), 'utf8');
const cientifico = readFileSync(fileURLToPath(new URL('../../skills/cientifico/SKILL.md', import.meta.url)), 'utf8');

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

  it('científico só embute as Regras do cientifico', () => {
    const prompt = buildSystemPrompt('científico', skill, cientifico);
    expect(prompt).toContain('### CIE-1');
    for (const h of ['## Formato de saída', '## Entrada', '## Fluxo']) expect(prompt).not.toContain(h);
  });

  it('falha se a seção Regras do cientifico some', () => {
    expect(() => buildSystemPrompt('científico', skill, '# x')).toThrow('cientifico/SKILL.md: seção Regras não encontrada');
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
