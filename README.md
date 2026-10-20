<div align="center">

<img alt="Uma pena de escrever, numa linha só: desenho original" src="site/static/favicon.svg" width="160"/>

# ptbr-writter

<sub><a href="https://h0ffmann.github.io/ptbr-writter/">o site</a> · <a href="SKILL.md">SKILL.md</a> · <a href="#regras">regras</a> · <a href="#roadmap">roadmap</a> · <a href="#awesome-ptbr-writter">awesome</a> · <a href="#desenvolvimento">desenvolvimento</a> · <a href="#licença">licença</a></sub>

[![site](https://github.com/h0ffmann/ptbr-writter/actions/workflows/site.yml/badge.svg)](https://github.com/h0ffmann/ptbr-writter/actions/workflows/site.yml)
[![license: MIT](https://img.shields.io/badge/license-MIT-1682D4?style=flat-square)](#licença)
[![skill: OpenCode](https://img.shields.io/badge/skill-OpenCode-1682D4?style=flat-square)](SKILL.md)
[![star me](https://img.shields.io/github/stars/h0ffmann/ptbr-writter?style=social&label=star%20me)](https://github.com/h0ffmann/ptbr-writter)
[![site: Gleam + Lustre](https://img.shields.io/badge/site-Gleam%20%2B%20Lustre-FFAFF3?style=flat-square)](#o-site)

Uma skill que reescreve texto técnico em português brasileiro para leitura única: sujeito explícito,
uma proposição por frase, termos consistentes. Feita para OpenCode e Claude Code, e pensada para
texto que humanos e agentes de IA vão ler. O objetivo: outro repositório instala a skill e mantém
sua documentação técnica em PT-BR e EN gastando menos tokens.
</div>

## a skill

[`SKILL.md`](SKILL.md) é o prompt que o agente carrega. Invoque com `/ptbr-writter` ou descreva a
tarefa ("reescreva este runbook em português controlado"). Sem nível explícito, a skill classifica
o texto e declara o que assumiu. A skill não muda tese, autoria nem conclusões, e não busca fontes:
marca o que não foi checado.

| Nível       | Uso                                         |
| :---------- | :------------------------------------------ |
| `estrito`   | runbooks, procedimentos, saídas para agentes |
| `descritivo` | documentação de sistema                    |
| `leve`      | comunicados internos                        |
| `científico` | artigos, propostas, relatórios técnicos com fontes |

## regras

O texto completo de cada regra está [no site](https://h0ffmann.github.io/ptbr-writter/).

| id | regra | origem | criador |
| :- | :---- | :----- | :------ |
| PTW-1 | Quem faz, aparece | PTC-1, [kayquer/portugues-tecnico-controlado](https://github.com/kayquer/portugues-tecnico-controlado) | [Kayque Rotondo](https://github.com/kayquer) |
| PTW-2 | Uma proposição por frase | PTC-2, [kayquer/portugues-tecnico-controlado](https://github.com/kayquer/portugues-tecnico-controlado) | [Kayque Rotondo](https://github.com/kayquer) |
| PTW-3 | Modalidade e quantidade explícitas | PTC-3, [kayquer/portugues-tecnico-controlado](https://github.com/kayquer/portugues-tecnico-controlado) | [Kayque Rotondo](https://github.com/kayquer) |
| PTW-4 | Verbo pleno | PTC-4, [kayquer/portugues-tecnico-controlado](https://github.com/kayquer/portugues-tecnico-controlado) | [Kayque Rotondo](https://github.com/kayquer) |
| PTW-5 | Sintaxe plana | PTC-5, [kayquer/portugues-tecnico-controlado](https://github.com/kayquer/portugues-tecnico-controlado) | [Kayque Rotondo](https://github.com/kayquer) |
| PTW-6 | Termos congelados | PTC-6, [kayquer/portugues-tecnico-controlado](https://github.com/kayquer/portugues-tecnico-controlado) | [Kayque Rotondo](https://github.com/kayquer) |
| PTW-7 | Números, datas e unidades | PTC-7, [kayquer/portugues-tecnico-controlado](https://github.com/kayquer/portugues-tecnico-controlado) | [Kayque Rotondo](https://github.com/kayquer) |
| PTW-8 | Ortografia PT-BR vigente | PTC-8, [kayquer/portugues-tecnico-controlado](https://github.com/kayquer/portugues-tecnico-controlado) | [Kayque Rotondo](https://github.com/kayquer) |
| PTW-9 | Toda afirmação tem marca | ouro-paper: toda afirmação tem marca, [marola-dev/marola-research](https://github.com/marola-dev/marola-research)<br>AGENTS.md: Audience and evidence, [h0ffmann/ww3-gpu](https://github.com/h0ffmann/ww3-gpu) | [Matheus Hoffmann](https://github.com/h0ffmann) |
| PTW-10 | Nenhuma referência inventada | ouro-paper: nunca inventar referência, [marola-dev/marola-research](https://github.com/marola-dev/marola-research) | [Matheus Hoffmann](https://github.com/h0ffmann) |
| PTW-11 | Número com unidade e origem | ouro-paper: todo número tem seu comando, [marola-dev/marola-research](https://github.com/marola-dev/marola-research)<br>AGENTS.md: Audience and evidence, [h0ffmann/ww3-gpu](https://github.com/h0ffmann/ww3-gpu) | [Matheus Hoffmann](https://github.com/h0ffmann) |
| PTW-12 | Sem alegação de pioneirismo | .vale/styles/research/PriorityClaims, [marola-dev/marola-research](https://github.com/marola-dev/marola-research) | [Matheus Hoffmann](https://github.com/h0ffmann) |
| PTW-13 | Fato separado de análise | ouro-paper: fatos e análise separados, [marola-dev/marola-research](https://github.com/marola-dev/marola-research) | [Matheus Hoffmann](https://github.com/h0ffmann) |

A saída tem quatro partes: nível e destinatário assumidos; tabela de violações (regra, trecho
original, reescrita); texto final; nota sobre o que foi mantido de propósito. No nível `científico`
há uma quinta: a lista de `⚠`. Os casos de regressão estão em [`evals/cientifico.md`](evals/cientifico.md).

A origem de cada regra está em [`site/data/origins.tsv`](site/data/origins.tsv), que o site também
mostra; [`SKILL.md`](SKILL.md) diz em que commit as regras científicas foram lidas.

## roadmap

Prontidão para produção. A fonte é [`site/data/roadmap.tsv`](site/data/roadmap.tsv); o
[site](https://h0ffmann.github.io/ptbr-writter/#roadmap) a lê no build.

| Peça | Status | O que falta ou o que há |
| :--- | :----- | :---------------------- |
| [`SKILL.md`](SKILL.md) | em andamento | regras PTW e níveis de rigor; as regras ainda estão marcadas como em desenvolvimento |
| [`evals/`](evals/cientifico.md) | em andamento | um caso de regressão por regra científica, escrito e ainda não rodado por um harness ⚠ |
| plugin | planejado | empacotar a skill como plugin do Claude Code e do OpenCode, instalável com um comando |
| distribuição | planejado | release com tag e instalação por URL em outros repositórios, sem copiar o `SKILL.md` à mão |

## awesome-ptbr-writter

Outros projetos para quem escreve texto técnico em português, cada um achado numa lista curada. A
fonte é [`site/data/awesome.tsv`](site/data/awesome.tsv).

| Repositório | O que dá | Achado em |
| :---------- | :------- | :-------- |
| [fabricioctelles/skills](https://github.com/fabricioctelles/skills) | skill "humanizar" para agentes: remove padrões de texto de IA em pt-BR | [awesome-jev](https://github.com/heyjunpenn/awesome-jev) |
| [vale-cli/vale](https://github.com/vale-cli/vale) | linter de prosa com estilos próprios, base para regras pt-BR | [awesome-scientific-writing](https://github.com/writing-resources/awesome-scientific-writing) |
| [hunspell/hunspell](https://github.com/hunspell/hunspell) | verificação ortográfica com dicionários pt-BR | [awesome-scientific-writing](https://github.com/writing-resources/awesome-scientific-writing) |
| [ajdavidl/Portuguese-NLP](https://github.com/ajdavidl/Portuguese-NLP) | índice de recursos de PLN para português | [awesome-nlp](https://github.com/keon/awesome-nlp) |

## o site

Gleam no alvo JavaScript e [Lustre](https://lustre.build), pré-renderizado para HTML estático. O
leitor não recebe JavaScript. Todo fato da página (regras, níveis, formato de saída, exemplos de
uso, limites, receitas do `just`) é lido de `SKILL.md` e do `justfile` por `site/content`, um gerador em
Gleam que escreve `site/src/ptbr_writter_site/content.gleam`. Nada em `site/src` é digitado à mão.
As regras visuais estão em [`DESIGN.md`](DESIGN.md).

Duas tabelas guardam o que nenhum outro arquivo guarda: [`site/data/awesome.tsv`](site/data/awesome.tsv)
(a seção awesome-ptbr-writter: repositórios relacionados, cada um com a lista curada onde foi
achado) e [`site/data/roadmap.tsv`](site/data/roadmap.tsv) (a prontidão para produção: SKILL.md,
evals, plugin, distribuição). O build falha se uma linha estiver malformada.

[`.github/workflows/site.yml`](.github/workflows/site.yml) publica no GitHub Pages: um pull request
só compila; um push na `main` compila e publica.

```console
git submodule update --init --depth 1 forks/picasso   # uma vez, raso
just site             # site/dist: formata, testa, gera o conteúdo, pré-renderiza
just site-serve       # compila e serve em http://localhost:8000
just pins             # o pin do flake.lock == o submódulo forks/picasso
```

<details>
<summary>Detalhes do ambiente</summary>

| Tipo       | Programa |
| :--------- | :-----: |
| Linguagem  | [Gleam 1.19.1](https://gleam.run/) no alvo JavaScript; [Lustre](https://lustre.build) do Hex |
| Runtime    | [Node.js 24](https://nodejs.org/) (`gleam test`, `gleam run` para a pré-renderização) |
| Scripting  | [just](https://github.com/casey/just), python 3 |
| Origem     | lab `labs/gleam-website` do [picasso](https://github.com/h0ffmann/picasso), pelo input `gleam-website` do flake |

</details>

O `flake.lock` fixa o `gleam-website` na mesma revisão do submódulo `forks/picasso`. Se as duas
diferirem, `just pins` falha.

## desenvolvimento

Para desenvolver a skill com OpenCode e um Qwen local servido pelo `llama.cpp`.

Requisitos:

- [Nix](https://nixos.org/download/) com flakes habilitados
- driver NVIDIA e GPU com ~24 GB de VRAM (padrão: Qwen3.6-27B Q4_K_M, contexto 131072)
- OpenCode instalado e no `PATH`

```console
nix develop           # entra no shell de desenvolvimento
just                  # lista as receitas
just quality          # justfile, flake, configuração e pins
just opencode-qwen    # sobe o Qwen3.6-27B e abre o OpenCode
just opencode-status  # o servidor está saudável?
just opencode-stop    # para o servidor na porta 8080
just opencode-logs    # acompanha o log do llama-server
just nix-check        # valida flake.nix e flake.lock sem compilar
```

A configuração do OpenCode está em `.opencode.json` (provider `llamacpp`, modelo `qwen3.6-27b`).

## estrutura

```text
SKILL.md                a skill
.opencode.json          OpenCode -> llamacpp/qwen3.6-27b
justfile                receitas
flake.nix, flake.lock   shell reprodutível e pins
DESIGN.md               regras visuais do site
evals/                  casos de regressão das regras (ainda sem harness)
forks/picasso/          submódulo: toolchain do site
site/
  content/              gerador: SKILL.md + justfile + data/ -> content.gleam
  data/                 awesome.tsv e roadmap.tsv
  src/                  o site em Gleam + Lustre
  static/               favicon e CSS
  test/                 testes
.github/workflows/site.yml
```

## CI

Runners hospedados pelo GitHub. [`site.yml`](.github/workflows/site.yml):

* **build** — em todo pull request e push: `just quality` (justfile, flake, pins), depois `just site`:
  o site é formatado, testado, gerado e pré-renderizado em `site/dist`. Um pull request não publica.
* **deploy** — só em push na `main`: publica `site/dist` no GitHub Pages.

## créditos

O layout deste README vem do [h0ffmann/picasso](https://github.com/h0ffmann/picasso). A stack e a
disciplina do site vêm de [h0ffmann/closed-source](https://github.com/h0ffmann/closed-source) e
[h0ffmann/ww3-gpu](https://github.com/h0ffmann/ww3-gpu). Lustre (MIT) e a stdlib do Gleam
(Apache-2.0) rodam só no build; a página publicada não leva o código deles. As fontes vêm do Google Fonts (SIL OFL).
As regras PTW-9 a PTW-13 adaptam a skill `ouro-paper` e o estilo `.vale/styles/research` de
[marola-dev/marola-research](https://github.com/marola-dev/marola-research) (`041cedd`) e a seção
"Audience and evidence" de [h0ffmann/ww3-gpu](https://github.com/h0ffmann/ww3-gpu) (`319e4e2`).

## licença

MIT. Veja [LICENSE](LICENSE).

<sub>ptbr-writter é um fork conceitual e independente do <a href="https://github.com/kayquer/portugues-tecnico-controlado">Português Técnico Controlado (PTC)</a>, de <a href="https://github.com/kayquer">kayquer</a> (MIT). A metodologia e as regras originais são de lá.</sub>
