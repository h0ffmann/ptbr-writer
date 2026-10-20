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

<details>
<summary>níveis de rigor e limites</summary>

[`SKILL.md`](SKILL.md) é o prompt que o agente carrega. Invoque com `/ptbr-writter` ou descreva a
tarefa ("reescreva este runbook em português controlado"). Sem nível explícito, a skill classifica
o texto e declara o que assumiu. A skill não muda tese, autoria nem conclusões, e não busca fontes:
marca o que não foi checado.

| Nível | Status | Uso |
| :---- | :----- | :-- |
| `estrito` | IMPLEMENTADO | runbooks, procedimentos, saídas para agentes |
| `descritivo` | IMPLEMENTADO | documentação de sistema |
| `leve` | IMPLEMENTADO | comunicados internos |
| `científico` | IMPLEMENTADO | artigos, propostas, relatórios técnicos com fontes |
| `retórico` | PLANEJADO | texto que precisa convencer: proposta, justificativa, comunicado de decisão |
| `ensaístico` | PLANEJADO | ensaio: o autor pensa por escrito, em primeira pessoa, à maneira de *The Essays* (*Os Ensaios*), de Michel de Montaigne |
| `crônico` | PLANEJADO | crônica: o cotidiano observado de perto, em tom de conversa com o leitor |
| `anedótico` | PLANEJADO | anedota: um caso curto e concreto que ilustra uma ideia |
| `machado-writter` | PLANEJADO | o estilo de Machado de Assis (1839–1908): ironia, digressão, o narrador que conversa com o leitor. Obra em domínio público; o texto produzido cita Machado de Assis ([REUSE.md](REUSE.md)) |
| `saramago-writter` | PLANEJADO | o estilo de José Saramago (1922–2010): períodos longos, pontuação mínima, diálogo sem travessão. Obra protegida por direito autoral: imita o estilo, nunca copia trecho ([REUSE.md](REUSE.md)) |

Um nível PLANEJADO ainda não está na [`SKILL.md`](SKILL.md), então não custa contexto a quem instala
a skill; a fonte da lista é [`site/data/planned.tsv`](site/data/planned.tsv). Talvez venha um
orwell-writer, como a skill [orwell-writing](https://github.com/tamdogood/builder-essential-skills/tree/1be9984/skills/orwell-writing) (as seis regras de Orwell com o ASD-STE100), mas feito para
os grandes escritores da língua portuguesa: um nível por autor, para
imitar estilos idiossincráticos de escrita. Os primeiros são Machado de Assis e José Saramago.

> [!WARNING]
> Um nível de autor imita um estilo, nunca copia texto, e respeita os direitos do autor. Obra em
> domínio público, de quem morreu há muito tempo, como a de Machado de Assis, pode servir de base, e
> o autor é citado sempre que o texto for usado. Obra protegida, como a de José Saramago, serve só
> de modelo de estilo. As regras estão em [`REUSE.md`](REUSE.md).

</details>

## regras

<details>
<summary>PTW-1 a PTW-13, com origem e criador</summary>

O texto completo de cada regra está [no site](https://h0ffmann.github.io/ptbr-writter/).

| id | regra | origem | criador |
| :- | :---- | :----- | :------ |
| PTW-1 | Quem faz, aparece | PTC-1, [kayquer/portugues-tecnico-controlado](https://github.com/kayquer/portugues-tecnico-controlado) | [@kayquer](https://github.com/kayquer) |
| PTW-2 | Uma proposição por frase | PTC-2, [kayquer/portugues-tecnico-controlado](https://github.com/kayquer/portugues-tecnico-controlado) | [@kayquer](https://github.com/kayquer) |
| PTW-3 | Modalidade e quantidade explícitas | PTC-3, [kayquer/portugues-tecnico-controlado](https://github.com/kayquer/portugues-tecnico-controlado) | [@kayquer](https://github.com/kayquer) |
| PTW-4 | Verbo pleno | PTC-4, [kayquer/portugues-tecnico-controlado](https://github.com/kayquer/portugues-tecnico-controlado) | [@kayquer](https://github.com/kayquer) |
| PTW-5 | Sintaxe plana | PTC-5, [kayquer/portugues-tecnico-controlado](https://github.com/kayquer/portugues-tecnico-controlado) | [@kayquer](https://github.com/kayquer) |
| PTW-6 | Termos congelados | PTC-6, [kayquer/portugues-tecnico-controlado](https://github.com/kayquer/portugues-tecnico-controlado) | [@kayquer](https://github.com/kayquer) |
| PTW-7 | Números, datas e unidades | PTC-7, [kayquer/portugues-tecnico-controlado](https://github.com/kayquer/portugues-tecnico-controlado) | [@kayquer](https://github.com/kayquer) |
| PTW-8 | Ortografia PT-BR vigente | PTC-8, [kayquer/portugues-tecnico-controlado](https://github.com/kayquer/portugues-tecnico-controlado) | [@kayquer](https://github.com/kayquer) |
| PTW-9 | Toda afirmação tem marca | ouro-paper: toda afirmação tem marca, [marola-dev/marola-research](https://github.com/marola-dev/marola-research)<br>AGENTS.md: Audience and evidence, [h0ffmann/ww3-gpu](https://github.com/h0ffmann/ww3-gpu) | [@h0ffmann](https://github.com/h0ffmann) |
| PTW-10 | Nenhuma referência inventada | ouro-paper: nunca inventar referência, [marola-dev/marola-research](https://github.com/marola-dev/marola-research) | [@h0ffmann](https://github.com/h0ffmann) |
| PTW-11 | Número com unidade e origem | ouro-paper: todo número tem seu comando, [marola-dev/marola-research](https://github.com/marola-dev/marola-research)<br>AGENTS.md: Audience and evidence, [h0ffmann/ww3-gpu](https://github.com/h0ffmann/ww3-gpu) | [@h0ffmann](https://github.com/h0ffmann) |
| PTW-12 | Sem alegação de pioneirismo | .vale/styles/research/PriorityClaims, [marola-dev/marola-research](https://github.com/marola-dev/marola-research) | [@h0ffmann](https://github.com/h0ffmann) |
| PTW-13 | Fato separado de análise | ouro-paper: fatos e análise separados, [marola-dev/marola-research](https://github.com/marola-dev/marola-research) | [@h0ffmann](https://github.com/h0ffmann) |

A saída tem quatro partes: nível e destinatário assumidos; tabela de violações (regra, trecho
original, reescrita); texto final; nota sobre o que foi mantido de propósito. No nível `científico`
há uma quinta: a lista de `⚠`. Os casos de regressão estão em [`evals/cientifico.md`](evals/cientifico.md).

A origem de cada regra está em [`site/data/origins.tsv`](site/data/origins.tsv), que o site também
mostra. Os commits lidos estão em [créditos](#créditos).

</details>

## a skill científica

[`skills/cientifico/SKILL.md`](skills/cientifico/SKILL.md) é uma skill própria para texto científico com fontes: aplica PTW-1 a PTW-13 e acrescenta as regras CIE. O corpo da skill só tem instruções; os créditos ficam aqui.

| id | regra | origem |
| :- | :---- | :----- |
| CIE-1 | Fonte lida antes de citada | ouro-paper: regras duras, [marola-dev/marola-research](https://github.com/marola-dev/marola-research) @ 041cedd<br>AGENTS.md: invariantes, [h0ffmann/closed-source](https://github.com/h0ffmann/closed-source) @ 563c339 |
| CIE-2 | Procurado e não citado | ouro-paper: passo 3, [marola-dev/marola-research](https://github.com/marola-dev/marola-research) @ 041cedd |
| CIE-3 | DOI é link | AGENTS.md: regras duras de citação, [marola-dev/marola-research](https://github.com/marola-dev/marola-research) @ 041cedd |
| CIE-4 | Fato que envelhece leva data | AGENTS.md: Audience and evidence, [h0ffmann/ww3-gpu](https://github.com/h0ffmann/ww3-gpu) @ 319e4e2 |
| CIE-5 | Código é `arquivo:linha` num commit | AGENTS.md: Audience and evidence; Third-party issues and pull requests, [h0ffmann/ww3-gpu](https://github.com/h0ffmann/ww3-gpu) @ 319e4e2 |
| CIE-6 | Issue e PR de terceiros não são fonte | AGENTS.md: Third-party issues and pull requests, [h0ffmann/ww3-gpu](https://github.com/h0ffmann/ww3-gpu) @ 319e4e2 |
| CIE-7 | Plano não é medição | ouro-paper: regras duras (sem resultado antes de paridade, tipos de afirmação), [marola-dev/marola-research](https://github.com/marola-dev/marola-research) @ 041cedd<br>AGENTS.md: Audience and evidence, [h0ffmann/ww3-gpu](https://github.com/h0ffmann/ww3-gpu) @ 319e4e2 |
| CIE-8 | Fonte privada fica privada | ouro-paper: regras duras; AGENTS.md: regra 8, [marola-dev/marola-research](https://github.com/marola-dev/marola-research) @ 041cedd |
| CIE-9 | Humano decide | AGENTS.md: invariantes 1 e 3, [h0ffmann/closed-source](https://github.com/h0ffmann/closed-source) @ 563c339<br>AGENTS.md: invariante 1, [ouroboric-dev/ouroboric](https://github.com/ouroboric-dev/ouroboric) @ 2858c11 |
| CIE-10 | Voz ativa, frase curta, número | .vale/styles/fabric: ActiveVoice, ShortSentences, PlainWords, NumbersOverAdjectives, [h0ffmann/closed-source](https://github.com/h0ffmann/closed-source) @ 563c339<br>.vale/styles/research, [marola-dev/marola-research](https://github.com/marola-dev/marola-research) @ 041cedd |
| CIE-11 | Figura carrega a evidência | AGENTS.md: Figures, [h0ffmann/ww3-gpu](https://github.com/h0ffmann/ww3-gpu) @ 319e4e2 |

O estilo `research` de [ouroboric-dev/ouroboric](https://github.com/ouroboric-dev/ouroboric) @ 2858c11 (`docs/4-ways-of-working/writing-styles.md`: afirmação atribuída, fato separado de análise) sustenta a mesma separação de PTW-13. Os autores são [@h0ffmann](https://github.com/h0ffmann).

## o linter saramaguiano

<details>
<summary>a pontuação de Saramago, em regras de Vale</summary>

O estilo de Saramago pode ser decodificado: a prosa dele abre mão de quase toda pontuação e fica com
a vírgula e o ponto. [`lint/saramago/`](lint/saramago) traduz isso em quatro regras de
[Vale](https://vale.sh), que valem para o nível `saramago-writter`:

| Regra | O que sinaliza | O que fazer |
| :---- | :------------- | :---------- |
| `Travessao` | travessão (`—`) | a fala segue no período, depois de uma vírgula, com maiúscula |
| `Aspas` | aspas (`"`, `“ ”`, `« »`) | a fala e o pensamento se marcam pela maiúscula depois da vírgula |
| `Entonacao` | `?` e `!` | a entonação fica nas palavras |
| `Pausa` | `;`, `:` e reticências | só vírgula e ponto |

```console
just saramago texto.md   # aponta cada marca que Saramago não usaria
just saramago-check      # autoteste: o exemplo saramaguiano passa, o convencional dispara as 4 regras
```

Os dois exemplos em `lint/saramago/fixtures/` foram escritos para este repositório; nenhum é trecho
de Saramago ([REUSE.md](REUSE.md)). As regras descrevem a pontuação, não o vocabulário nem a
sintaxe: um texto pode passar no linter e não soar como Saramago ⚠.

</details>

## a skill de retórica

<details>
<summary>status, epígrafe e base</summary>

[`skills/retorica/SKILL.md`](skills/retorica/SKILL.md), em desenvolvimento e ainda sem regras. A
epígrafe é de Robert M. Pirsig, *Zen and the Art of Motorcycle Maintenance* (1974), e se dirige ao
Phaedrus dele; o texto foi conferido em duas reproduções públicas, não na edição impressa ⚠.

Base: o *Fédon* e o *Fedro*, de Platão, e o Phaedrus de Pirsig. Cada regra futura vai citar seu
trecho aqui, não na skill: Platão pela numeração de Stephanus, Pirsig pelo capítulo. A fazer: ler
as três obras e anotar cada edição; propor as regras numa nota revisada pelo dono; escrever um caso
de regressão por regra em `evals/`.

</details>

## roadmap

Prontidão para produção. A fonte é [`site/data/roadmap.tsv`](site/data/roadmap.tsv); o
[site](https://h0ffmann.github.io/ptbr-writter/#roadmap) a lê no build.

| Peça | Status | O que falta ou o que há |
| :--- | :----- | :---------------------- |
| [`SKILL.md`](SKILL.md) | em andamento | regras PTW e níveis de rigor; as regras ainda estão marcadas como em desenvolvimento |
| [`evals/`](evals/cientifico.md) | em andamento | um caso de regressão por regra científica, escrito e ainda não rodado por um harness ⚠ |
| [skill científica](skills/cientifico/SKILL.md) | em andamento | skill própria para texto científico, com regras CIE; casos de regressão ainda não escritos |
| plugin | planejado | empacotar a skill como plugin do Claude Code e do OpenCode, instalável com um comando |
| distribuição | planejado | release com tag e instalação por URL em outros repositórios, sem copiar o `SKILL.md` à mão |
| [retórica](skills/retorica/SKILL.md) | planejado | skill de retórica a partir do *Fédon* e do *Fedro*, de Platão, e do Phaedrus de Robert M. Pirsig; só o esqueleto existe, sem regras |

## awesome-ptbr-writter

Outros projetos para quem escreve texto técnico em português. O primeiro é o PTC original, de onde
este fork vem; os outros foram achados numa lista curada. A fonte é
[`site/data/awesome.tsv`](site/data/awesome.tsv).

| Repositório | O que dá | Achado em |
| :---------- | :------- | :-------- |
| [kayquer/portugues-tecnico-controlado](https://github.com/kayquer/portugues-tecnico-controlado) | o original: 8 regras de português técnico controlado, inspiradas no ASD-STE100, com modo bilíngue EN/PT | a origem deste fork |
| [fabricioctelles/skills](https://github.com/fabricioctelles/skills) | skill "humanizar" para agentes: remove padrões de texto de IA em pt-BR | [awesome-jev](https://github.com/heyjunpenn/awesome-jev) |
| [vale-cli/vale](https://github.com/vale-cli/vale) | linter de prosa com estilos próprios, base para regras pt-BR | [awesome-scientific-writing](https://github.com/writing-resources/awesome-scientific-writing) |
| [languagetool-org/languagetool](https://github.com/languagetool-org/languagetool) | corretor gramatical e de estilo com suporte a pt-BR | [awesome-selfhosted](https://github.com/awesome-selfhosted/awesome-selfhosted) |
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
lint/saramago/          linter de prosa saramaguiano (Vale)
skills/retorica/        skill de retórica, em desenvolvimento (Fédon, Fedro, Pirsig)
forks/picasso/          submódulo: toolchain do site
skills/cientifico/      skill científica (regras CIE)
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

A ideia de um nível por autor vem da skill [orwell-writing](https://github.com/tamdogood/builder-essential-skills/tree/1be9984/skills/orwell-writing), de @tamdogood (MIT). O layout deste README vem do [h0ffmann/picasso](https://github.com/h0ffmann/picasso). A stack e a
disciplina do site vêm de [h0ffmann/closed-source](https://github.com/h0ffmann/closed-source) e
[h0ffmann/ww3-gpu](https://github.com/h0ffmann/ww3-gpu). Lustre (MIT) e a stdlib do Gleam
(Apache-2.0) rodam só no build; a página publicada não leva o código deles. As fontes vêm do Google Fonts (SIL OFL).
As regras PTW-9 a PTW-13 adaptam a skill `ouro-paper` e o estilo `.vale/styles/research` de
[marola-dev/marola-research](https://github.com/marola-dev/marola-research) (`041cedd`) e a seção
"Audience and evidence" de [h0ffmann/ww3-gpu](https://github.com/h0ffmann/ww3-gpu) (`319e4e2`).

## licença

MIT. Veja [LICENSE](LICENSE).

<sub>ptbr-writter é um fork conceitual e independente do <a href="https://github.com/kayquer/portugues-tecnico-controlado">Português Técnico Controlado (PTC)</a>, de <a href="https://github.com/kayquer">kayquer</a> (MIT). A metodologia e as regras originais são de lá.</sub>
