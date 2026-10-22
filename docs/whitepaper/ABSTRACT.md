# Whitepaper do ptbr-writer: abstract para revisão

Rascunho para o autor revisar antes do PDF. Nível `científico`; destinatário: quem lê o registro no
Zenodo e decide se usa ou cita o ptbr-writer. As marcas são da skill `cientifico`: `(v)` é um fato
conferido num arquivo deste repositório no commit `dfe86ce` (2026-10-22), e `⚠` é o que não foi
medido. As marcas saem do PDF.

**Autores:** Matheus Hoffmann (ORCID 0009-0009-1056-7661) e Rob Kler Soares da Silva Junior
(@robkler, ORCID 0009-0003-3102-4996), nesta ordem. O nome e o ORCID do Rob vêm do registro do
marola no Zenodo (v0.2.2, [10.5281/zenodo.23266642](https://doi.org/10.5281/zenodo.23266642)) e
batem com o registro público dele no ORCID, lidos em 2026-10-22. O registro no Zenodo é um preprint.

**Para você decidir (CIE-9):** o título, e se a tese "uma sugestão de escrita é uma decisão" entra
como tese do artigo ou como escolha de projeto.

## Título (proposta)

**ptbr-writer: português técnico controlado para pessoas e agentes**

## Resumo

O ptbr-writer reescreve texto técnico em português brasileiro para uma leitura única: sujeito
explícito, uma proposição por frase e termos consistentes. A semente é o Português Técnico
Controlado (PTC), de kayquer: uma skill do Claude Code com regras de linguagem controlada em
português para um leitor que pode ser um modelo (v). O ptbr-writer adapta as 8 regras do PTC e
acrescenta 5, num total de 13 regras, PTW-1 a PTW-13 (v).
Cada pedido passa por um de 5 níveis de rigor, de `estrito` a `smart-note` (v). Para texto com
fontes, a skill `cientifico` acrescenta 12 regras, CIE-1 a CIE-12 (v). As mesmas regras chegam por
três caminhos: um plugin do Claude Code, um instalador para o OpenCode e uma extensão de navegador
para Chrome e Firefox (v). A extensão trata a revisão como uma decisão. Além de modelos que geram
texto (Claude, DeepSeek ou um modelo local pelo Ollama), ela aceita um modelo de decisão local,
servido pelo Ollaya, que diz qual regra cada frase quebra, com uma probabilidade, sem gerar texto
(v). As ferramentas de execução são binários Rust estáticos, fixados por sha256, e cada afirmação
não conferida leva a marca `⚠` (v). Este artigo descreve as regras, os níveis, a arquitetura e a
política de evidência. Ele não relata medida de acerto: os casos de regressão em `evals/` estão
escritos e não foram rodados, e a latência e o acerto dos modelos locais em pt-BR não foram
medidos ⚠. O código está sob a MPL-2.0 desde a v0.3.0 (v).

**Palavras-chave:** escrita técnica; português brasileiro; linguagem controlada; agentes de
programação; modelos de decisão; reprodutibilidade.

## Abstract

ptbr-writer rewrites technical text in Brazilian Portuguese for a single reading: explicit
subject, one proposition per sentence and consistent terms. Its seed is Português Técnico
Controlado (PTC), by kayquer: a Claude Code skill with controlled-language rules in Portuguese for
a reader that may be a model (v). ptbr-writer adapts the 8 rules of PTC and adds 5, for 13 rules in
total, PTW-1 to PTW-13 (v). Each
request goes through one of 5 levels of rigor, from `estrito` to `smart-note` (v). For text with
sources, the `cientifico` skill adds 12 rules, CIE-1 to CIE-12 (v). The same rules reach the writer
by three paths: a Claude Code plugin, an installer for OpenCode and a browser extension for Chrome
and Firefox (v). The extension treats a review as a decision. Besides models that generate text
(Claude, DeepSeek or a local model through Ollama), it accepts a local decision model, served by
Ollaya, which says which rule each sentence breaks, with a probability, and generates no text (v).
The runtime tools are static Rust binaries pinned by sha256, and each claim that was not checked
carries the mark `⚠` (v). This paper describes the rules, the levels, the architecture and the
evidence policy. It reports no measure of accuracy: the regression cases in `evals/` are written
and were not run, and the latency and accuracy of the local models on pt-BR were not measured ⚠.
The code is under MPL-2.0 since v0.3.0 (v).

**Keywords:** technical writing; Brazilian Portuguese; controlled language; coding agents;
decision models; reproducibility.

## Como cada afirmação foi conferida

| Afirmação | Fonte |
| :-- | :-- |
| o PTC é uma skill do Claude Code, para um leitor que pode ser um modelo | descrição e README de [kayquer/portugues-tecnico-controlado](https://github.com/kayquer/portugues-tecnico-controlado/tree/cfdffad0ed98cdebfb7f591481889090a4f4e65b) no commit `cfdffad`, lidos em 2026-10-22 ("Skill do Claude Code"; "Um modelo que consome uma instrução também não") |
| 8 regras do PTC | a mesma descrição ("com 8 regras próprias") e `site/data/awesome.tsv` |
| 13 regras PTW | `SKILL.md`: 13 cabeçalhos `### PTW-` |
| 5 níveis | `SKILL.md`, seção "Níveis de rigor": 5 itens |
| 12 regras CIE | `skills/cientifico/SKILL.md`: 12 cabeçalhos `### CIE-` |
| plugin, OpenCode, extensão | `plugins/ptbr-writer/`, `scripts/install-opencode.sh`, `extension/` |
| Ollaya: uma decisão por frase, sem gerar texto | `extension/src/decisions.ts` |
| binários fixados por sha256 | `plugins/ptbr-writer/bin/tools.lock` |
| `evals/` escritos e não rodados | `site/data/roadmap.tsv`, linha `evals` |
| MPL-2.0 desde a v0.3.0 | `LICENSE`, `CHANGELOG.md` (v0.3.0) |

## Pendências (⚠)

- A medida de acerto das regras em pt-BR: rodar os `evals/` por um harness.
- A latência e o acerto do Ollama e do Ollaya na extensão (#61).
- Se o Zenodo arquiva a release criada pelo `release.yml`: o webhook do Zenodo ainda não está
  ligado neste repositório.

## Procurado e não citado (CIE-3)

A única fonte externa lida para este abstract é o repositório do PTC, no commit `cfdffad`. O
registro no Zenodo o cita com a relação `isDerivedFrom`. O artigo completo vai citar também o
Brysbaert (2019), o ASD-STE100 e o Ollaya, cada um lido antes.
