<div align="center">

<img alt="Uma pena de escrever, numa linha só: desenho original" src="site/static/favicon.svg" width="160"/>

# ptbr-writter

<sub><a href="https://h0ffmann.github.io/ptbr-writter/">o site</a> · <a href="SKILL.md">SKILL.md</a> · <a href="#regras">regras</a> · <a href="#desenvolvimento">desenvolvimento</a> · <a href="#licença">licença</a></sub>

[![site](https://github.com/h0ffmann/ptbr-writter/actions/workflows/site.yml/badge.svg)](https://github.com/h0ffmann/ptbr-writter/actions/workflows/site.yml)
[![license: MIT](https://img.shields.io/badge/license-MIT-1682D4?style=flat-square)](#licença)
[![skill: OpenCode](https://img.shields.io/badge/skill-OpenCode-1682D4?style=flat-square)](SKILL.md)
[![site: Gleam + Lustre](https://img.shields.io/badge/site-Gleam%20%2B%20Lustre-FFAFF3?style=flat-square)](#o-site)

Uma skill que reescreve texto técnico em português brasileiro para leitura única: sujeito explícito,
uma proposição por frase, termos consistentes. Feita para OpenCode e Claude Code, e pensada para
texto que humanos e agentes de IA vão ler.
</div>

## a skill

[`SKILL.md`](SKILL.md) é o prompt que o agente carrega. Invoque com `/ptbr-writter` ou descreva a
tarefa ("reescreva este runbook em português controlado"). Sem nível explícito, a skill classifica
o texto e declara o que assumiu.

| Nível       | Uso                                         |
| :---------- | :------------------------------------------ |
| `estrito`   | runbooks, procedimentos, saídas para agentes |
| `descritivo` | documentação de sistema                    |
| `leve`      | comunicados internos                        |

## regras

O texto completo de cada regra está [no site](https://h0ffmann.github.io/ptbr-writter/).

| id    | regra                       |
| :---- | :-------------------------- |
| PTW-1 | Quem faz, aparece           |
| PTW-2 | Uma proposição por frase    |
| PTW-3 | Modalidade e quantidade explícitas |
| PTW-4 | Verbo pleno                 |
| PTW-5 | Sintaxe plana               |
| PTW-6 | Termos congelados           |
| PTW-7 | Números, datas e unidades   |
| PTW-8 | Ortografia PT-BR vigente    |

A saída tem quatro partes: nível e destinatário assumidos; tabela de violações (regra, trecho
original, reescrita); texto final; nota sobre o que foi mantido de propósito.

## o site

Gleam no alvo JavaScript e [Lustre](https://lustre.build), pré-renderizado para HTML estático. O
leitor não recebe JavaScript. Todo fato da página (regras, níveis, formato de saída, exemplos de
uso, receitas do `just`) é lido de `SKILL.md` e do `justfile` por `site/content`, um gerador em
Gleam que escreve `site/src/ptbr_writter_site/content.gleam`. Nada em `site/src` é digitado à mão.
As regras visuais estão em [`DESIGN.md`](DESIGN.md).

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
forks/picasso/          submódulo: toolchain do site
site/
  content/              gerador: SKILL.md + justfile -> content.gleam
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

## licença

MIT. Veja [LICENSE](LICENSE).

<sub>ptbr-writter é um fork conceitual e independente do <a href="https://github.com/kayquer/portugues-tecnico-controlado">Português Técnico Controlado (PTC)</a>, de <a href="https://github.com/kayquer">kayquer</a> (MIT). A metodologia e as regras originais são de lá.</sub>
