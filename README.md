<div align="center">

<img alt="Uma pena de escrever, numa linha só: desenho original" src="site/static/favicon.svg" width="160"/>

# ptbr-writter

<sub><a href="#instalar">instalar</a> · <a href="#usar">usar</a> · <a href="#níveis">níveis</a> · <a href="#regras">regras</a> · <a href="#outras-skills">outras skills</a> · <a href="#versões-e-release">versões</a> · <a href="#qualidade-do-plugin">qualidade</a> · <a href="#roadmap">roadmap</a> · <a href="#desenvolvimento">desenvolvimento</a> · <a href="#créditos">créditos</a> · <a href="https://h0ffmann.github.io/ptbr-writter/">o site</a></sub>

[![site](https://github.com/h0ffmann/ptbr-writter/actions/workflows/site.yml/badge.svg)](https://github.com/h0ffmann/ptbr-writter/actions/workflows/site.yml)
[![license: MIT](https://img.shields.io/badge/license-MIT-1682D4?style=flat-square)](#licença)
[![plugin: Claude Code](https://img.shields.io/badge/plugin-Claude%20Code-1682D4?style=flat-square)](#instalar)
[![release](https://img.shields.io/github/v/release/h0ffmann/ptbr-writter?style=flat-square&label=release)](https://github.com/h0ffmann/ptbr-writter/releases/tag/v0.1.0)
[![star me](https://img.shields.io/github/stars/h0ffmann/ptbr-writter?style=social&label=star%20me)](https://github.com/h0ffmann/ptbr-writter)

Reescreve texto técnico em português brasileiro para leitura única: sujeito explícito, uma
proposição por frase, termos consistentes. Um plugin do Claude Code, para texto que humanos e agentes
leem. Objetivo: manter a documentação PT-BR e EN de um repositório gastando menos tokens.
</div>

## instalar

ptbr-writter é um plugin do Claude Code. A versão oficial é a
**[v0.1.0](https://github.com/h0ffmann/ptbr-writter/releases/tag/v0.1.0)**: instale sempre por uma
tag, e o plugin só muda quando você trocar a tag.

### no seu Claude Code

```console
/plugin marketplace add h0ffmann/ptbr-writter#v0.1.0
/plugin install ptbr-writter@ptbr-writter
```

Abra uma sessão nova e as skills aparecem em [usar](#usar).

### num repositório

Para que todo mundo que abre o repositório use a mesma versão, ponha no `.claude/settings.json`
dele e faça o commit:

```json
{
  "extraKnownMarketplaces": {
    "ptbr-writter": {
      "source": { "source": "github", "repo": "h0ffmann/ptbr-writter", "ref": "v0.1.0" }
    }
  },
  "enabledPlugins": { "ptbr-writter@ptbr-writter": true }
}
```

O Claude Code oferece o plugin, nessa versão, a quem confia na pasta do repositório.

### atualizar

As versões estão em [releases](https://github.com/h0ffmann/ptbr-writter/releases), e o que muda em
cada uma, no [`CHANGELOG.md`](CHANGELOG.md).

- No seu Claude Code: `/plugin marketplace remove ptbr-writter` e o `add` de novo, com a tag nova.
- Num repositório: troque o `ref` num PR.

<details>
<summary>seguir o <code>main</code>, sem tag, e o hook que oferece o plugin</summary>

Sem a tag (`/plugin marketplace add h0ffmann/ptbr-writter`), o marketplace lê o `main`. Mesmo assim,
o plugin só muda para quem já instalou quando o `version` do
[`plugin.json`](plugins/ptbr-writter/.claude-plugin/plugin.json) sobe, ou seja, numa release; um
commit sem release não chega. Atualize com `/plugin marketplace update ptbr-writter` e
`/plugin update ptbr-writter@ptbr-writter`.

Um repositório pode lembrar quem ainda não instalou o plugin: copie
[`skills/ptbr-routing/`](skills/ptbr-routing/SKILL.md) para `.claude/skills/ptbr-routing/` e
acrescente o hook ao `.claude/settings.json`. Ele fica calado depois da instalação ou com
`PTBR_PLUGIN_OPTOUT=1`.

```json
"hooks": {
  "SessionStart": [{ "hooks": [{ "type": "command", "timeout": 5,
    "command": "bash \"$CLAUDE_PROJECT_DIR/.claude/skills/ptbr-routing/scripts/session-check.sh\"" }] }]
}
```

</details>

## usar

| Comando | Faz |
| :------ | :-- |
| `/ptbr-writter:ptbr-writter` | reescreve no nível pedido, ou classifica o texto e declara o nível |
| `/ptbr-writter:ptbr` | o mesmo, com nome curto |
| `/ptbr-writter:cientifico` | texto científico com fontes: artigo, proposta, relatório (regras CIE) |

Ou descreva a tarefa: "reescreva este runbook em português controlado". A skill não muda tese,
autoria nem conclusões e não busca fontes: marca o que não checou. A saída tem nível e
destinatário assumidos, a tabela de violações (regra, trecho, reescrita), o texto final e o que foi
mantido de propósito.

## níveis

| Nível | Status | Uso |
| :---- | :----- | :-- |
| `estrito` | IMPLEMENTADO | runbooks, procedimentos, saídas para agentes |
| `descritivo` | IMPLEMENTADO | documentação de sistema |
| `leve` | IMPLEMENTADO | comunicados internos |
| `científico` | IMPLEMENTADO | artigos, propostas, relatórios técnicos com fontes |
| `smart-note` | IMPLEMENTADO | notas para um segundo cérebro; combina com outro nível (`smart-note + estrito`) |
| `retórico` | PLANEJADO | texto que precisa convencer: proposta, justificativa, comunicado de decisão |
| `ensaístico` | PLANEJADO | ensaio em primeira pessoa, à maneira de *Os Ensaios*, de Montaigne |
| `crônico` | PLANEJADO | crônica: o cotidiano observado de perto, em tom de conversa |
| `anedótico` | PLANEJADO | anedota: um caso curto e concreto que ilustra uma ideia |
| `machado-writter` | PLANEJADO | o estilo de Machado de Assis; domínio público, sempre citado ([REUSE.md](REUSE.md)) |
| `saramago-writter` | PLANEJADO | o estilo de José Saramago; obra protegida: só o estilo, nunca um trecho ([REUSE.md](REUSE.md)) |

Um nível PLANEJADO não está na `SKILL.md` e não custa contexto. A lista vem de
[`site/data/planned.tsv`](site/data/planned.tsv).

> [!WARNING]
> Um nível de autor imita o estilo e nunca copia texto. As regras de citação e de domínio público
> estão em [`REUSE.md`](REUSE.md).

**smart-note** entrega o texto `RAW`, pronto para colar, e/ou a `nota` indexável: uma ideia por
nota, front matter (`id`, `type`, `status`, `source`, `tags`) e links `[[id-slug]]`. Sem pedido,
entrega as duas. Regras na seção `## Smart note` da [`SKILL.md`](SKILL.md).

## regras

O texto de cada regra está na [`SKILL.md`](SKILL.md) e [no site](https://h0ffmann.github.io/ptbr-writter/#regras).
A origem vem de [`site/data/origins.tsv`](site/data/origins.tsv).

| id | regra | origem | criador |
| :- | :---- | :----- | :------ |
| PTW-1 | Quem faz, aparece | PTC-1, [kayquer/portugues-tecnico-controlado](https://github.com/kayquer/portugues-tecnico-controlado) | [@kayquer](https://github.com/kayquer) |
| PTW-2 | Uma proposição por frase | PTC-2, idem | [@kayquer](https://github.com/kayquer) |
| PTW-3 | Modalidade e quantidade explícitas | PTC-3, idem | [@kayquer](https://github.com/kayquer) |
| PTW-4 | Verbo pleno | PTC-4, idem | [@kayquer](https://github.com/kayquer) |
| PTW-5 | Sintaxe plana | PTC-5, idem | [@kayquer](https://github.com/kayquer) |
| PTW-6 | Termos congelados | PTC-6, idem | [@kayquer](https://github.com/kayquer) |
| PTW-7 | Números, datas e unidades | PTC-7, idem | [@kayquer](https://github.com/kayquer) |
| PTW-8 | Ortografia PT-BR vigente | PTC-8, idem | [@kayquer](https://github.com/kayquer) |
| PTW-9 | Toda afirmação tem marca | ouro-paper, [marola-dev/marola-research](https://github.com/marola-dev/marola-research); Audience and evidence, [h0ffmann/ww3-gpu](https://github.com/h0ffmann/ww3-gpu) | [@h0ffmann](https://github.com/h0ffmann) |
| PTW-10 | Nenhuma referência inventada | ouro-paper, [marola-dev/marola-research](https://github.com/marola-dev/marola-research) | [@h0ffmann](https://github.com/h0ffmann) |
| PTW-11 | Número com unidade e origem | ouro-paper, [marola-dev/marola-research](https://github.com/marola-dev/marola-research); Audience and evidence, [h0ffmann/ww3-gpu](https://github.com/h0ffmann/ww3-gpu) | [@h0ffmann](https://github.com/h0ffmann) |
| PTW-12 | Sem alegação de pioneirismo | `.vale/styles/research/PriorityClaims`, [marola-dev/marola-research](https://github.com/marola-dev/marola-research) | [@h0ffmann](https://github.com/h0ffmann) |
| PTW-13 | Fato separado de análise | ouro-paper, [marola-dev/marola-research](https://github.com/marola-dev/marola-research) | [@h0ffmann](https://github.com/h0ffmann) |

## outras skills

- **[`cientifico`](skills/cientifico/SKILL.md)**: texto científico com fontes. Aplica PTW-1 a
  PTW-13 e acrescenta CIE-1 a CIE-12 (nada inventado, fonte lida antes de citada, DOI como link,
  código como `arquivo:linha`, humano decide). A origem de cada regra está abaixo.
- **[`ptbr-routing`](skills/ptbr-routing/SKILL.md)**: escolhe a skill e o nível antes de carregar
  qualquer outra, para que só uma entre no contexto. Fica fora do plugin; o repositório a copia
  para `.claude/skills/`. O nome não colide com o `routing` do marola-devkit.

  ```console
  $ just route "transforme em notas do Obsidian este comunicado"
  ptbr-writter · smart-note + leve
  ```

- **[`retorica`](skills/retorica/SKILL.md)**: em desenvolvimento, sem regras, fora do plugin. Base:
  o *Fédon* e o *Fedro*, de Platão, e o Phaedrus de Robert M. Pirsig.
- **[linter saramaguiano](lint/saramago)**: quatro regras de [Vale](https://vale.sh) para o nível
  `saramago-writter`: sem travessão, sem aspas, sem `?` e `!`, sem `;`, `:` e reticências.
  `just saramago texto.md`.

<details>
<summary>origem das regras CIE</summary>

Base: a skill `scientific-writing` de [K-Dense-AI/scientific-agent-skills](https://github.com/K-Dense-AI/scientific-agent-skills/tree/a1043ef544e3a100c28e70a736b6cf32554d0677/skills/scientific-writing) @ a1043ef (MIT), adaptada sem copiar texto. Citação pedida: Kassis, T., Agarwal, V., He, Y., Patel, D., & Brueckner, A. M. (2026). *Scientific Agent Skills: A Library of Procedural Knowledge for Research Agents*. arXiv:2609.00065. <https://doi.org/10.48550/arXiv.2609.00065>

| id | regra | origem |
| :- | :---- | :----- |
| CIE-1 | Nada inventado | `scientific-writing` (No fabrication); ouro-paper, [marola-dev/marola-research](https://github.com/marola-dev/marola-research) @ 041cedd; AGENTS.md invariante 2, [h0ffmann/closed-source](https://github.com/h0ffmann/closed-source) @ 563c339 |
| CIE-2 | Fonte lida antes de citada | `scientific-writing` (Evidence binding); ouro-paper |
| CIE-3 | Procurado e não citado | ouro-paper, passo 3 |
| CIE-4 | DOI é link, fato que envelhece leva data | AGENTS.md, marola-research; Audience and evidence, [h0ffmann/ww3-gpu](https://github.com/h0ffmann/ww3-gpu) @ 319e4e2 |
| CIE-5 | Código é `arquivo:linha` num commit | AGENTS.md, ww3-gpu |
| CIE-6 | Issue e PR de terceiros não são fonte | AGENTS.md, ww3-gpu |
| CIE-7 | Fidelidade científica | `scientific-writing` (Scientific fidelity); ouro-paper; ww3-gpu |
| CIE-8 | Confidencial fica confidencial | `scientific-writing` (Confidentiality); ouro-paper |
| CIE-9 | Humano decide | `scientific-writing`; closed-source invariantes 1 e 3; [ouroboric-dev/ouroboric](https://github.com/ouroboric-dev/ouroboric) @ 2858c11 invariante 1 |
| CIE-10 | Número completo | `scientific-writing` (Numbers and units); ouro-paper |
| CIE-11 | Estilo | `scientific-writing` (Language review); `artigo`, [labdaps/labskills](https://github.com/labdaps/labskills/tree/246c1646a51a5a754aedf786fef0544d528064e7/skills/artigo) @ 246c164; estilos Vale de closed-source e marola-research |
| CIE-12 | Figura e tabela carregam a evidência | `scientific-writing` (figures_tables); ww3-gpu |

</details>

## versões e release

Código e plugin têm uma versão só: o `version` do
[`plugin.json`](plugins/ptbr-writter/.claude-plugin/plugin.json), o cabeçalho do
[`CHANGELOG.md`](CHANGELOG.md) e a tag `vX.Y.Z`.

```console
just release 0.2.0   # 1ª vez: abre o PR chore/release-v0.2.0 (plugin.json + CHANGELOG)
just release 0.2.0   # depois do merge, no main: cria a tag; release.yml publica a release
```

Quem instalou o plugin só vê uma mudança quando a versão sobe: o Claude Code guarda a cópia pelo
número da versão. Por isso a release está amarrada ao que o plugin entrega:

- `just release-due` lista os arquivos do plugin (`SKILL.md`, `skills/cientifico/`, os manifestos)
  que mudaram desde a última tag. Num push no `main`, o CI mostra um aviso quando há algum.
- `just release` recusa uma versão sem mudança no plugin e põe esses arquivos na entrada do
  `CHANGELOG.md`, para você descrever.
- Uma tag que não bate com o `plugin.json` não vira release.

Quem instalou pelo `main` recebe a versão nova no merge do PR de release; quem pinou, quando troca
o `ref`. O site publica do `main` a cada merge.

## qualidade do plugin

Dois checks, cada um quando faz sentido ([`plugin.yml`](.github/workflows/plugin.yml)):

| Check | Quando | Custo | O que garante |
| :---- | :----- | :---- | :------------ |
| `plugin-smoke` | PR ou push que muda o que o plugin entrega | zero: sem login, sem tokens | instala do checkout num Claude Code vazio: manifestos válidos, as três skills em arquivos reais com `description`, custo fixo abaixo de 300 tokens por sessão |
| `eval` | só à mão: Actions → plugin → Run workflow | créditos de API, com teto (`max_cost_usd`, padrão US$ 2) | os casos de [`plugins/ptbr-writter/evals/`](plugins/ptbr-writter/evals), com e sem o plugin, avaliados por um juiz (Haiku) |

Para o `eval` no CI com um plano Max:

1. Em claude.ai → Settings → Billing, vincule uma organização do Console ao plano. O Max 5x traz
   US$ 100 por mês de créditos de API e o Max 20x, US$ 200. A oferta aparece depois de 7 dias no plano.
2. No Console dessa organização, crie uma API key.
3. No GitHub, guarde a chave como secret `ANTHROPIC_API_KEY` (Settings → Secrets and variables →
   Actions).

Sem crédito comprado na organização, uma chamada sem saldo para, e nada é cobrado no plano. Não
está claro se o eval no Actions conta como uso do Claude Code, que os créditos não cobrem ⚠: confira
na página de uso do Console depois da primeira execução. Localmente, `just plugin-eval` roda os
mesmos casos no seu login (um caso custou cerca de US$ 0,09).

## roadmap

Fonte: [`site/data/roadmap.tsv`](site/data/roadmap.tsv).

| Peça | Status | O que falta ou o que há |
| :--- | :----- | :---------------------- |
| [`SKILL.md`](SKILL.md) | em andamento | regras PTW e níveis de rigor; as regras ainda estão marcadas como em desenvolvimento |
| [`evals/`](evals/cientifico.md) | em andamento | casos de regressão escritos, ainda não rodados por um harness ⚠ |
| [skill científica](skills/cientifico/SKILL.md) | em andamento | regras CIE; casos em `evals/cientifico-cie.md`, ainda não rodados ⚠ |
| [roteamento](skills/ptbr-routing/SKILL.md) | feito | `route` escolhe a skill e o nível antes de carregar uma skill |
| [plugin](plugins/ptbr-writter/.claude-plugin/plugin.json) | em andamento | plugin do Claude Code pronto; falta o OpenCode |
| [distribuição](CHANGELOG.md) | feito | release por tag com `just release`; a v0.1.0 é a versão oficial, instalada por tag |
| [retórica](skills/retorica/SKILL.md) | planejado | só o esqueleto, sem regras |

## awesome-ptbr-writter

Outros projetos para quem escreve texto técnico em português. Fonte:
[`site/data/awesome.tsv`](site/data/awesome.tsv).

| Repositório | O que dá | Achado em |
| :---------- | :------- | :-------- |
| [kayquer/portugues-tecnico-controlado](https://github.com/kayquer/portugues-tecnico-controlado) | o original: 8 regras de português técnico controlado, inspiradas no ASD-STE100 | a origem deste fork |
| [fabricioctelles/skills](https://github.com/fabricioctelles/skills) | skill "humanizar": remove padrões de texto de IA em pt-BR | [awesome-jev](https://github.com/heyjunpenn/awesome-jev) |
| [vale-cli/vale](https://github.com/vale-cli/vale) | linter de prosa com estilos próprios | [awesome-scientific-writing](https://github.com/writing-resources/awesome-scientific-writing) |
| [languagetool-org/languagetool](https://github.com/languagetool-org/languagetool) | corretor gramatical e de estilo com pt-BR | [awesome-selfhosted](https://github.com/awesome-selfhosted/awesome-selfhosted) |
| [hunspell/hunspell](https://github.com/hunspell/hunspell) | ortografia com dicionários pt-BR | [awesome-scientific-writing](https://github.com/writing-resources/awesome-scientific-writing) |
| [ajdavidl/Portuguese-NLP](https://github.com/ajdavidl/Portuguese-NLP) | índice de recursos de PLN para português | [awesome-nlp](https://github.com/keon/awesome-nlp) |

## desenvolvimento

Sem o Claude Code (OpenCode, ou outro agente que lê skills): copie [`SKILL.md`](SKILL.md) e, se
quiser, [`skills/cientifico/`](skills/cientifico/SKILL.md) para a pasta de skills do agente.

Requisitos: [Nix](https://nixos.org/download/) com flakes. Para rodar a skill com um Qwen local:
GPU NVIDIA com ~24 GB e OpenCode no `PATH`.

```console
nix develop           # o shell de desenvolvimento
just quality          # tudo o que o CI checa: pins, linter, route, plugin, README
just release-due      # o plugin mudou desde a última tag?
just site             # o site em site/dist
just route "..."      # skill e nível de um pedido
just opencode-qwen    # sobe o Qwen3.6-27B e abre o OpenCode
```

```text
SKILL.md                a skill principal (níveis e regras PTW)
skills/                 cientifico, ptbr (alias), ptbr-routing, retorica
plugins/ptbr-writter/   o plugin: plugin.json e links para as skills
.claude-plugin/         marketplace.json
evals/                  casos de regressão (ainda sem harness)
lint/saramago/          linter saramaguiano (Vale)
site/                   o site (Gleam + Lustre); fatos lidos de SKILL.md, justfile e site/data/
forks/picasso/          submódulo: toolchain do site, pinado no flake.lock
```

CI: [`site.yml`](.github/workflows/site.yml) roda `just quality` e `just site` em todo PR e publica
o site do `main`; [`plugin.yml`](.github/workflows/plugin.yml) instala o plugin quando ele muda e
roda o eval sob demanda; [`release.yml`](.github/workflows/release.yml) publica uma release por tag.

## créditos

- Regras PTW-1 a PTW-8: o PTC de [@kayquer](https://github.com/kayquer) (MIT).
- PTW-9 a PTW-13: a skill `ouro-paper` e o estilo `.vale/styles/research` de
  [marola-dev/marola-research](https://github.com/marola-dev/marola-research) (`041cedd`) e
  "Audience and evidence" de [h0ffmann/ww3-gpu](https://github.com/h0ffmann/ww3-gpu) (`319e4e2`).
- Skill científica: `scientific-writing` de K-Dense-AI (`a1043ef`, MIT, Copyright (c) 2025 K-Dense
  Inc.) e `artigo` de labdaps/labskills (`246c164`, MIT, Copyright (c) 2026 LABDAPS), em palavras
  próprias.
- smart-note: *How to Take Smart Notes*, de Sönke Ahrens; o Zettelkasten de Niklas Luhmann; o
  fichário de Robert M. Pirsig para *Lila* ⚠.
- Retórica: a epígrafe é de Pirsig, *Zen and the Art of Motorcycle Maintenance* (1974), conferida em
  duas reproduções públicas, não na edição impressa ⚠.
- Nível por autor: a skill [orwell-writing](https://github.com/tamdogood/builder-essential-skills/tree/1be9984/skills/orwell-writing), de @tamdogood (MIT).
- Plugin e roteamento no molde de [marola-dev/marola-devkit](https://github.com/marola-dev/marola-devkit).
  Layout do README: [h0ffmann/picasso](https://github.com/h0ffmann/picasso). Stack do site:
  [h0ffmann/closed-source](https://github.com/h0ffmann/closed-source) e ww3-gpu; Lustre (MIT) e a
  stdlib do Gleam (Apache-2.0) só no build; fontes do Google Fonts (SIL OFL); bandeira e assinatura
  de Machado de Assis do Wikimedia Commons, em domínio público.

## licença

MIT. Veja [LICENSE](LICENSE).

<sub>ptbr-writter é um fork conceitual e independente do <a href="https://github.com/kayquer/portugues-tecnico-controlado">Português Técnico Controlado (PTC)</a>, de <a href="https://github.com/kayquer">kayquer</a> (MIT). A metodologia e as regras originais são de lá.</sub>
