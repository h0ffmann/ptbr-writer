<div align="center">

<img alt="Uma pena de escrever, numa linha só: desenho original" src="site/static/favicon.svg" width="160"/>

# ptbr-writter

<sub><a href="https://h0ffmann.github.io/ptbr-writter/">o site</a> · <a href="SKILL.md">SKILL.md</a> · <a href="#regras">regras</a> · <a href="#roadmap">roadmap</a> · <a href="#awesome-ptbr-writter">awesome</a> · <a href="#desenvolvimento">desenvolvimento</a> · <a href="#licença">licença</a></sub>

[![site](https://github.com/h0ffmann/ptbr-writter/actions/workflows/site.yml/badge.svg)](https://github.com/h0ffmann/ptbr-writter/actions/workflows/site.yml)
[![license: MIT](https://img.shields.io/badge/license-MIT-1682D4?style=flat-square)](#licença)
[![skill: OpenCode](https://img.shields.io/badge/skill-OpenCode-1682D4?style=flat-square)](SKILL.md)
[![star me](https://img.shields.io/github/stars/h0ffmann/ptbr-writter?style=social&label=star%20me)](https://github.com/h0ffmann/ptbr-writter)
[![site: Gleam + Lustre](https://img.shields.io/badge/site-Gleam%20%2B%20Lustre-FFAFF3?style=flat-square)](https://h0ffmann.github.io/ptbr-writter/)

Uma skill que reescreve texto técnico em português brasileiro para leitura única: sujeito explícito,
uma proposição por frase, termos consistentes. Feita para OpenCode e Claude Code, e pensada para
texto que humanos e agentes de IA vão ler. O objetivo: outro repositório instala a skill e mantém
sua documentação técnica em PT-BR e EN gastando menos tokens.
</div>

## a skill

<details>
<summary>níveis de rigor e limites</summary>

[`SKILL.md`](SKILL.md) é o prompt que o agente carrega. Invoque com `/ptbr-writter` (alias `/ptbr`) ou descreva a
tarefa ("reescreva este runbook em português controlado"). Sem nível explícito, a skill classifica
o texto e declara o que assumiu. A skill não muda tese, autoria nem conclusões, e não busca fontes:
marca o que não foi checado.

| Nível | Status | Uso |
| :---- | :----- | :-- |
| `estrito` | IMPLEMENTADO | runbooks, procedimentos, saídas para agentes |
| `descritivo` | IMPLEMENTADO | documentação de sistema |
| `leve` | IMPLEMENTADO | comunicados internos |
| `científico` | IMPLEMENTADO | artigos, propostas, relatórios técnicos com fontes |
| `smart-note` | IMPLEMENTADO | notas para um segundo cérebro (2nd-brain); combina com outro nível e entrega o texto `RAW` e/ou a `nota` indexável, com tags e links |
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

<details>
<summary>regras CIE, base e origem</summary>

[`skills/cientifico/SKILL.md`](skills/cientifico/SKILL.md) é uma skill própria para texto científico com fontes. A base de estrutura e de orientação é a skill `scientific-writing` de [K-Dense-AI/scientific-agent-skills](https://github.com/K-Dense-AI/scientific-agent-skills/tree/a1043ef544e3a100c28e70a736b6cf32554d0677/skills/scientific-writing) @ a1043ef (MIT, "Copyright (c) 2025 K-Dense Inc."): as ideias foram adaptadas ao português e a textos técnicos (artigo, proposta, relatório), sem copiar texto. A skill aplica PTW-1 a PTW-13 e acrescenta as regras CIE. O corpo da skill só tem instruções; os créditos ficam aqui.

Como pede o autor da base: Kassis, T., Agarwal, V., He, Y., Patel, D., & Brueckner, A. M. (2026). *Scientific Agent Skills: A Library of Procedural Knowledge for Research Agents*. arXiv:2609.00065. <https://doi.org/10.48550/arXiv.2609.00065>

Referência em português usada, só pelas ideias, parafraseadas: a skill `artigo` de [labdaps/labskills](https://github.com/labdaps/labskills/tree/246c1646a51a5a754aedf786fef0544d528064e7/skills/artigo) @ 246c164 (MIT, "Copyright (c) 2026 LABDAPS"), a lista de enchimento e de padrões de texto de IA em português (CIE-11). Procurei outras skills em português (buscas por "escrita científica", "redação científica", "ABNT", "português acadêmico"): `GCNatureza/COautorIA` (MIT) é um aplicativo web, não uma skill, e os demais repositórios achados não têm licença ou não são material para agente.

| id | regra | origem |
| :- | :---- | :----- |
| CIE-1 | Nada inventado | `scientific-writing`: No fabrication, [K-Dense-AI/scientific-agent-skills](https://github.com/K-Dense-AI/scientific-agent-skills/tree/a1043ef544e3a100c28e70a736b6cf32554d0677/skills/scientific-writing) @ a1043ef<br>ouro-paper: nunca inventar referência, [marola-dev/marola-research](https://github.com/marola-dev/marola-research) @ 041cedd<br>AGENTS.md: invariante 2, [h0ffmann/closed-source](https://github.com/h0ffmann/closed-source) @ 563c339 |
| CIE-2 | Fonte lida antes de citada | `scientific-writing`: Evidence binding, [K-Dense-AI/scientific-agent-skills](https://github.com/K-Dense-AI/scientific-agent-skills/tree/a1043ef544e3a100c28e70a736b6cf32554d0677/skills/scientific-writing) @ a1043ef<br>ouro-paper: regras duras, [marola-dev/marola-research](https://github.com/marola-dev/marola-research) @ 041cedd |
| CIE-3 | Procurado e não citado | ouro-paper: passo 3 e o registro "Looked for and not cited", [marola-dev/marola-research](https://github.com/marola-dev/marola-research) @ 041cedd |
| CIE-4 | DOI é link, fato que envelhece leva data | AGENTS.md: regras duras de citação, [marola-dev/marola-research](https://github.com/marola-dev/marola-research) @ 041cedd<br>AGENTS.md: Audience and evidence, [h0ffmann/ww3-gpu](https://github.com/h0ffmann/ww3-gpu) @ 319e4e2 |
| CIE-5 | Código é `arquivo:linha` num commit | AGENTS.md: Audience and evidence; Third-party issues and pull requests, [h0ffmann/ww3-gpu](https://github.com/h0ffmann/ww3-gpu) @ 319e4e2 |
| CIE-6 | Issue e PR de terceiros não são fonte | AGENTS.md: Third-party issues and pull requests, [h0ffmann/ww3-gpu](https://github.com/h0ffmann/ww3-gpu) @ 319e4e2 |
| CIE-7 | Fidelidade científica | `scientific-writing`: Scientific fidelity e writing_principles, [K-Dense-AI/scientific-agent-skills](https://github.com/K-Dense-AI/scientific-agent-skills/tree/a1043ef544e3a100c28e70a736b6cf32554d0677/skills/scientific-writing) @ a1043ef<br>ouro-paper: regras duras (paridade, tolerância), [marola-dev/marola-research](https://github.com/marola-dev/marola-research) @ 041cedd<br>AGENTS.md: Audience and evidence, [h0ffmann/ww3-gpu](https://github.com/h0ffmann/ww3-gpu) @ 319e4e2 |
| CIE-8 | Confidencial fica confidencial | `scientific-writing`: Confidentiality, [K-Dense-AI/scientific-agent-skills](https://github.com/K-Dense-AI/scientific-agent-skills/tree/a1043ef544e3a100c28e70a736b6cf32554d0677/skills/scientific-writing) @ a1043ef<br>ouro-paper: fonte privada, [marola-dev/marola-research](https://github.com/marola-dev/marola-research) @ 041cedd |
| CIE-9 | Humano decide | `scientific-writing`: autoria e IA, declarações, [K-Dense-AI/scientific-agent-skills](https://github.com/K-Dense-AI/scientific-agent-skills/tree/a1043ef544e3a100c28e70a736b6cf32554d0677/skills/scientific-writing) @ a1043ef<br>AGENTS.md: invariantes 1 e 3, [h0ffmann/closed-source](https://github.com/h0ffmann/closed-source) @ 563c339<br>AGENTS.md: invariante 1, [ouroboric-dev/ouroboric](https://github.com/ouroboric-dev/ouroboric) @ 2858c11 |
| CIE-10 | Número completo | `scientific-writing`: writing_principles, Numbers and units, [K-Dense-AI/scientific-agent-skills](https://github.com/K-Dense-AI/scientific-agent-skills/tree/a1043ef544e3a100c28e70a736b6cf32554d0677/skills/scientific-writing) @ a1043ef<br>ouro-paper: todo número tem seu comando, [marola-dev/marola-research](https://github.com/marola-dev/marola-research) @ 041cedd |
| CIE-11 | Estilo | `scientific-writing`: Language review, [K-Dense-AI/scientific-agent-skills](https://github.com/K-Dense-AI/scientific-agent-skills/tree/a1043ef544e3a100c28e70a736b6cf32554d0677/skills/scientific-writing) @ a1043ef<br>`artigo`: padrões de texto de IA em português, [labdaps/labskills](https://github.com/labdaps/labskills/tree/246c1646a51a5a754aedf786fef0544d528064e7/skills/artigo) @ 246c164<br>.vale/styles/fabric, [h0ffmann/closed-source](https://github.com/h0ffmann/closed-source) @ 563c339; .vale/styles/research, [marola-dev/marola-research](https://github.com/marola-dev/marola-research) @ 041cedd |
| CIE-12 | Figura e tabela carregam a evidência | `scientific-writing`: figures_tables, [K-Dense-AI/scientific-agent-skills](https://github.com/K-Dense-AI/scientific-agent-skills/tree/a1043ef544e3a100c28e70a736b6cf32554d0677/skills/scientific-writing) @ a1043ef<br>AGENTS.md: Figures, [h0ffmann/ww3-gpu](https://github.com/h0ffmann/ww3-gpu) @ 319e4e2 |

O fluxo, a estrutura do texto e a revisão vêm de `scientific-writing` (workflow, imrad_structure, Revision and peer review). O estilo `research` de [ouroboric-dev/ouroboric](https://github.com/ouroboric-dev/ouroboric) @ 2858c11 (`docs/4-ways-of-working/writing-styles.md`) sustenta a separação de fato e análise de PTW-13. Autor das regras próprias: [@h0ffmann](https://github.com/h0ffmann).

</details>

## smart-note

<details>
<summary>o nível para quem mantém um segundo cérebro</summary>

`smart-note` é um nível para quem mantém um segundo cérebro (2nd-brain). Ele combina com os outros:
`smart-note + estrito` aplica as regras de `estrito` e entrega o resultado em notas. A pessoa escolhe
a saída:

- **`RAW`**: só o texto, pronto para colar.
- **`nota`**: conhecimento indexável, uma ideia por nota, com front matter (`id`, `type`, `status`,
  `source`, `tags`), links `[[id-slug]]` entre notas e nome permanente.

Sem pedido, a skill entrega as duas. As regras estão na seção `## Smart note` da
[`SKILL.md`](SKILL.md) e no [site](https://h0ffmann.github.io/ptbr-writter/#smart-note).

Base: *How to Take Smart Notes*, de Sönke Ahrens; o Zettelkasten (o fichário) de Niklas Luhmann; e o
fichário analógico de Robert M. Pirsig, as fichas com que ele organizou as ideias de *Lila* ⚠. Os
casos de regressão estão em [`evals/smart-note.md`](evals/smart-note.md).

</details>

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

## plugin do Claude Code

<details>
<summary>instalar, fixar uma versão, e como sai uma release</summary>

```console
/plugin marketplace add h0ffmann/ptbr-writter
/plugin install ptbr-writter@ptbr-writter
```

As skills carregam como `/ptbr-writter:ptbr-writter`, `/ptbr-writter:ptbr` e
`/ptbr-writter:cientifico`. A de retórica fica fora até ter regras. Um repositório fixa a versão em
`.claude/settings.json`, e, se quiser, oferece o plugin a quem abre o repositório com o hook de
`ptbr-routing` (silencioso depois de instalado ou com `PTBR_PLUGIN_OPTOUT=1`):

```json
{
  "extraKnownMarketplaces": {
    "ptbr-writter": { "source": { "source": "github", "repo": "h0ffmann/ptbr-writter", "ref": "v0.1.0" } }
  },
  "enabledPlugins": { "ptbr-writter@ptbr-writter": true },
  "hooks": {
    "SessionStart": [{ "hooks": [{ "type": "command", "timeout": 5,
      "command": "bash \"$CLAUDE_PROJECT_DIR/.claude/skills/ptbr-routing/scripts/session-check.sh\"" }] }]
  }
}
```

Código e plugin têm uma versão só: o `version` de
[`plugin.json`](plugins/ptbr-writter/.claude-plugin/plugin.json), o cabeçalho do
[`CHANGELOG.md`](CHANGELOG.md) e a tag `vX.Y.Z`. `just release X.Y.Z` roda duas vezes. Na
primeira, abre o PR `chore/release-vX.Y.Z` com o `plugin.json` e a entrada do `CHANGELOG.md`, que
você escreve no editor. Depois do merge, no `main`, a segunda cria e envia a tag; o
[`release.yml`](.github/workflows/release.yml) roda `just release-check` e publica a release. Usar
o plugin não exige release: sem `ref`, o marketplace lê o `main`. Uma tag que não bate com o `plugin.json` não vira
release. Entre tags, o `main` muda sem atingir quem fixou o `ref`; o site publica do `main` a cada
merge. As skills do plugin são links para os arquivos do repositório, e o Claude Code copia o
conteúdo ao instalar; `just plugin-check` (parte de `just quality`) falha num link quebrado.

</details>

## roteamento

<details>
<summary>uma skill e um nível, antes de carregar qualquer outra</summary>

[`skills/ptbr-routing/`](skills/ptbr-routing/SKILL.md) escolhe a skill e o nível de um pedido de escrita, para
que só essa skill entre no contexto. `route` é Python sem dependências e lê
[`routes.tsv`](skills/ptbr-routing/routes.tsv), uma linha por nível com as palavras que o indicam.
`smart-note` combina com o nível que empatar no topo.

```console
$ just route "reescreva este runbook de incidente"
ptbr-writter · estrito
$ just route "transforme em notas do Obsidian este comunicado"
ptbr-writter · smart-note + leve
```

Fica fora do plugin, como o `routing` do [marola-dev/marola-devkit](https://github.com/marola-dev/marola-devkit)
(v0.20.4): um repositório copia `skills/ptbr-routing/` para `.claude/skills/ptbr-routing/` e a fixa
num commit no `skills.lock`, com o `skills-vendor` do devkit:

```console
$ skills_vendor.py init ptbr-routing --upstream h0ffmann/ptbr-writter --path skills/ptbr-routing --paths LICENSE=LICENSE
```

O nome é `ptbr-routing`, não `routing`, para conviver com o `routing` do devkit no mesmo
`.claude/skills/`. Aqui, `.claude/skills/ptbr-routing` é um link para `skills/ptbr-routing`.
`just quality` roda o self-test e confere que cada nível de `SKILL.md` tem uma linha em `routes.tsv`.

</details>

## roadmap

Prontidão para produção. A fonte é [`site/data/roadmap.tsv`](site/data/roadmap.tsv); o
[site](https://h0ffmann.github.io/ptbr-writter/#roadmap) a lê no build.

| Peça | Status | O que falta ou o que há |
| :--- | :----- | :---------------------- |
| [`SKILL.md`](SKILL.md) | em andamento | regras PTW e níveis de rigor; as regras ainda estão marcadas como em desenvolvimento |
| [`evals/`](evals/cientifico.md) | em andamento | um caso de regressão por regra científica, escrito e ainda não rodado por um harness ⚠ |
| [skill científica](skills/cientifico/SKILL.md) | em andamento | skill própria para texto científico, com regras CIE; casos de regressão em `evals/cientifico-cie.md`, escritos e ainda não rodados ⚠ |
| [roteamento](skills/ptbr-routing/SKILL.md) | feito | `route` escolhe a skill e o nível antes de carregar uma skill, para que só ela entre no contexto |
| [plugin](plugins/ptbr-writter/.claude-plugin/plugin.json) | em andamento | plugin do Claude Code: marketplace e plugin `ptbr-writter`, com `ptbr-writter`, `ptbr` e `cientifico`; falta o OpenCode |
| [distribuição](CHANGELOG.md) | em andamento | uma versão para código e plugin: a tag `vX.Y.Z` vira release (`release.yml`) e outros repositórios fixam o marketplace nela; nenhuma tag publicada ainda |
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
| [hunspell/hunspell](https://github.com/hunspell/hunspell) | verificação ortográfica com dicionários pt-BR | [awesome-scientific-writing](https://github.com/writing-resources/awesome-scientific-writing) |
| [ajdavidl/Portuguese-NLP](https://github.com/ajdavidl/Portuguese-NLP) | índice de recursos de PLN para português | [awesome-nlp](https://github.com/keon/awesome-nlp) |

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
skills/ptbr/            alias /ptbr: link para SKILL.md
plugins/ptbr-writter/   plugin do Claude Code: plugin.json e links para as skills
.claude-plugin/         marketplace.json
skills/ptbr-routing/    roteamento: skill e nível de um pedido (route, routes.tsv)
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
(Apache-2.0) rodam só no build; a página publicada não leva o código deles. As fontes vêm do Google Fonts (SIL OFL). A bandeira do Brasil e a assinatura de Machado de Assis (1839–1908) no site vêm do
Wikimedia Commons, em domínio público.
A skill científica adapta, em palavras próprias, `scientific-writing` de [K-Dense-AI/scientific-agent-skills](https://github.com/K-Dense-AI/scientific-agent-skills) (`a1043ef`, MIT, Copyright (c) 2025 K-Dense Inc.) e `artigo` de [labdaps/labskills](https://github.com/labdaps/labskills) (`246c164`, MIT, Copyright (c) 2026 LABDAPS). As regras PTW-9 a PTW-13 adaptam a skill `ouro-paper` e o estilo `.vale/styles/research` de
[marola-dev/marola-research](https://github.com/marola-dev/marola-research) (`041cedd`) e a seção
"Audience and evidence" de [h0ffmann/ww3-gpu](https://github.com/h0ffmann/ww3-gpu) (`319e4e2`).

## licença

MIT. Veja [LICENSE](LICENSE).

<sub>ptbr-writter é um fork conceitual e independente do <a href="https://github.com/kayquer/portugues-tecnico-controlado">Português Técnico Controlado (PTC)</a>, de <a href="https://github.com/kayquer">kayquer</a> (MIT). A metodologia e as regras originais são de lá.</sub>
