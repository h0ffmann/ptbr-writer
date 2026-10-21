<div align="center">

<img alt="A writing quill in a single line: original drawing" src="site/static/favicon.svg" width="120"/>

# ptbr-writer

<sub><a href="#install">install</a> · <a href="#use">use</a> · <a href="#levels">levels</a> · <a href="#rules">rules</a> · <a href="#eli5">ELI5</a> · <a href="#figures">figures</a> · <a href="#roadmap">roadmap</a> · <a href="#development">development</a> · <a href="#credits">credits</a> · <a href="CHANGELOG.md">versions</a> · <a href="https://h0ffmann.github.io/ptbr-writer/">the site</a></sub>

[![site](https://github.com/h0ffmann/ptbr-writer/actions/workflows/site.yml/badge.svg)](https://github.com/h0ffmann/ptbr-writer/actions/workflows/site.yml)
[![version](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fraw.githubusercontent.com%2Fh0ffmann%2Fptbr-writer%2Fmain%2Fplugins%2Fptbr-writer%2F.claude-plugin%2Fplugin.json&query=%24.version&prefix=v&label=version&style=flat-square)](CHANGELOG.md)
[![license: MIT](https://img.shields.io/badge/license-MIT-1682D4?style=flat-square)](#license)
[![estimated reading time: ~22 min](https://img.shields.io/badge/reading%20time-~22%20min-1682D4?style=flat-square)](#reading-time)
[![plugin: Claude Code](https://img.shields.io/badge/plugin-Claude%20Code-1682D4?style=flat-square)](#install)

</div>

🇧🇷 [Leia em português](https://github.com/h0ffmann/ptbr-writer/blob/main/README.md) · this is the English version

## in one minute

- **It rewrites technical text in Brazilian Portuguese for a single read:** explicit subject, one
  proposition per sentence, consistent terms. It serves text that humans and agents read.
- **Every request goes through a level of rigor** (`estrito`, `descritivo`, `leve`, `científico`,
  `smart-note`), and the skill states the level it assumed.
- **It does not change thesis, authorship or conclusion, and it does not look up sources.** It marks
  with `⚠` what it did not check. Goal: keep a repository's PT-BR and EN documentation while
  spending fewer tokens.

![Flow of a request (labels in Portuguese): you write ptbr; the first time, 4 questions saved in ptbr-writer.toml; then the ptbr-writer skill (or cientifico, if the text needs sources) writes one text per level and language; the output is the reply in the console and, if asked, .txt, .md, .pdf or notes files in the folder.](docs/img/em-um-minuto.png)

<details open>
<summary>How to read this figure</summary>

**Takeaway.** You write `ptbr: …`; the first time the plugin asks 4 questions and saves the
answers, and then the skill writes one text per level and per language, in the reply and, if you
asked, in files.

**How to read.** Left to right, in the top lane, which every request goes through. The bottom lane
is the configuration and the files. A labelled arrow says when the path applies. A box with an
orange border is where you come in or what you get back.

**Not shown.** The levels and the rules (PTW and CIE), the `smart-note` level, the
`ptbr reconfigurar` request and the routing skill `ptbr-routing`, which is outside the plugin. The
figure is in Portuguese in both READMEs: its words are the plugin's own (`ptbr:`,
`ptbr-writer.toml`, the file names), and one drawing keeps the two pages saying the same.

**Evidence.** The 4 questions, `ptbr-writer.toml`, the formats and the languages:
[`labs/ptbr-tools`](https://github.com/h0ffmann/picasso/tree/main/labs/ptbr-tools) (`ptbr-ask-levels`). The
`cientifico` skill for sourced text and the output format: [`SKILL.md`](SKILL.md) and
[`skills/cientifico/SKILL.md`](skills/cientifico/SKILL.md). Source of the figure:
[`docs/archify/em-um-minuto.json`](docs/archify/em-um-minuto.json); `just figure` draws it again.
Whether Claude picks `cientifico` by itself, from its description, was not measured ⚠.

</details>

## install

The official version is **[v0.2.0](https://github.com/h0ffmann/ptbr-writer/releases/tag/v0.2.0)**.
Install by tag: the plugin changes only when you change the tag. One line, in a terminal:

```console
claude plugin install ptbr-writer --marketplace h0ffmann/ptbr-writer#v0.2.0
```

Or, inside Claude Code:

```console
/plugin marketplace add h0ffmann/ptbr-writer#v0.2.0
/plugin install ptbr-writer@ptbr-writer
```

For everyone who opens a repository to use the same version, put this in its `.claude/settings.json`:

```json
{
  "extraKnownMarketplaces": {
    "ptbr-writer": {
      "source": { "source": "github", "repo": "h0ffmann/ptbr-writer", "ref": "v0.2.0" }
    }
  },
  "enabledPlugins": { "ptbr-writer@ptbr-writer": true }
}
```

**Update:** change the tag (`/plugin marketplace remove ptbr-writer` and `add` again; in a
repository, the `ref`). What changes in each version is in [`CHANGELOG.md`](CHANGELOG.md).

### OpenCode

One line that downloads the tag, copies the skills `ptbr-writer`, `ptbr` and `cientifico` as files
(no links) into `~/.config/opencode/skills/` and installs the `ask-levels` plugin:

```console
curl -fsSL https://raw.githubusercontent.com/h0ffmann/ptbr-writer/v0.3.0/scripts/install-opencode.sh | bash -s -- v0.3.0
```

`--project` at the end installs into `./.opencode/`. ⚠ `v0.3.0` does not exist yet: the script
ships in the next release; until then, run `scripts/install-opencode.sh vX.Y.Z` from a clone of the
tag you use. **Update:** run it again with the new tag. The plugin does what the hook does in Claude Code: a
request that starts with `ptbr` gets the instruction to ask levels, formats, folder and languages
through the `question` tool; any other request gets nothing. Tested on OpenCode 1.19.0: the three
skills load and the instruction reaches the model; ⚠ that the model answers with the `question`
tool was not measured (`opencode run` does not offer it). It needs `bash`, `curl` and `tar`; the
binaries are `linux-x86_64` and `linux-aarch64`. `ptbr-routing` is not included: it depends on a
Claude Code hook.

## use

| Command | Does |
| :------ | :--- |
| `/ptbr-writer:ptbr` | rewrites at the requested level, or classifies the text and states the level |
| `/ptbr-writer:ptbr-writer` | the same, with the full name |
| `/ptbr-writer:cientifico` | scientific text with sources: paper, proposal, report (CIE rules) |
| `/ptbr-writer:bipr` | opens or updates the current branch's PR, with the body in pt-BR and its English mirror, from the commits (`--dry-run` only shows it) |

Or describe the task: "reescreva este runbook em português controlado".

From v0.2.0, a request that starts with `ptbr` ("ptbr: reescreva este texto", `/ptbr`) uses
`ptbr-writer.toml`. Naming ptbr-writer inside a sentence triggers nothing. The first time, with no
such file, Claude asks once, multiple choice:

- **levels:** one text per level;
- **formats:** `.txt`, `.md`, `.pdf` and `notes` (smart-note);
- **folder:** the session root (the default), `docs/`, or another one, typed in;
- **languages:** pt-BR (the default) and/or ASD-STE100, Simplified Technical English; one text per language.

and writes the answers to `./ptbr-writer.toml`. From then on it does not ask: edit the file, or
write `ptbr reconfigurar`.

```toml
levels = ["estrito"]     # empty: the skill classifies and states the level
formats = ["md", "pdf"]  # txt, md, pdf, notes; empty: only the answer
folder = "docs"          # relative to the session root, no ".."
print = ["txt", "md"]    # shown in full in the console, besides the file
languages = ["pt-BR"]    # and/or "asd-ste100"; the ASD-STE100 file ends in -ste100
```

Each layer overrides the one before: the plugin's defaults
([`ptbr-writer.default.toml`](plugins/ptbr-writer/ptbr-writer.default.toml)),
`~/.config/ptbr-writer/ptbr-writer.toml`, `./ptbr-writer.toml` and, last, the request
(`ptbr leve: ...` uses `leve`). An unknown value or a folder outside the root is ignored, with a
warning. Files share one name pattern:

```text
ptbr-writer-<YYYYMMDDThhmmZ>-<goal>-<level>.<ext>
ptbr-writer-20261020T2215Z-reinicio-servico-estrito.md
```

The timestamp is UTC and the same for every file of one request. Two levels in three formats make
six files. So a `git add -A` does not pick them up, add `ptbr-writer-*` to `.gitignore`. Notes from
`notes` follow the smart-note rules (`<id>-<slug>.md`). `.pdf` uses the machine's pandoc.
Without it, Claude asks before downloading, through Nix, picasso's
[`labs/ptbr-pdf`](https://github.com/h0ffmann/picasso/tree/main/labs/ptbr-pdf) (a Rust binary,
`to-pdf`, with Typst and fonts embedded). The binary is not in cache.nixos.org yet: Nix compiles it
from source. Measured with `just pdf-bench` (x86_64-linux), at picasso v0.2.0, with the slim
binary ([run](https://github.com/h0ffmann/ptbr-writer/actions/runs/39537436728)):

| download | on disk | built on the machine | first run, fresh GitHub runner (4 cores) |
| :------- | :------ | :------------------- | :--------------------------------------- |
| 14 MiB, 6 paths from cache.nixos.org | 72 MiB, 7 paths | 25 MiB, 1 path (`to-pdf`, compiled) | 892 s (15 min) |

The time depends on the network. From the second run on, Nix reuses the store.

![Where the .pdf comes from: to-pdf.sh uses pandoc and xelatex if present; otherwise it downloads, through Nix, picasso at the commit to-pdf.sh pins (PICASSO_REV), whose labs/ptbr-pdf compiles the to-pdf 0.1.0 crate, with pulldown-cmark 0.13.4, typst 0.15.1 and typst-as-lib 0.17.0 and 6 embedded fonts.](docs/img/pdf-dependencias.png)

<details open>
<summary>How to read this figure</summary>

**Takeaway.** The `.pdf` comes from the machine's pandoc, if it has one with xelatex; otherwise from
a Rust binary that Nix compiles from a fixed picasso commit, with each library at the version of
`Cargo.lock`.

**How to read.** Left to right. The middle lane is the path without pandoc; the bottom one, the
binary's Rust libraries; the top one, the local path and the finished file. A box with an orange
border is where a path starts or ends. The labels are in Portuguese, as in the first figure.

**Not shown.** The dependencies of the dependencies (`Cargo.lock` has over 3000 lines), the
`flake.lock` of `labs/ptbr-pdf` and Nix itself.

**Evidence.** [`plugins/ptbr-writer/scripts/to-pdf.sh`](plugins/ptbr-writer/scripts/to-pdf.sh)
(`PICASSO_REV`, both paths). In picasso, at commit `4bc03e5`: `labs/ptbr-pdf/Cargo.toml` (the five
dependencies), `labs/ptbr-pdf/Cargo.lock` (`pulldown-cmark` 0.13.4, `typst`, `typst-layout` and
`typst-pdf` 0.15.1, `typst-as-lib` 0.17.0, `to-pdf` 0.1.0) and `labs/ptbr-pdf/fonts/` (six files:
Libertinus Serif and DejaVu Sans Mono). Source of the figure:
[`docs/archify/pdf-dependencias.json`](docs/archify/pdf-dependencias.json). That `to-pdf.sh` and
the commit pinned in `forks/picasso` agree, `just pins` checks.

</details>

## levels

| Level | Use |
| :---- | :-- |
| `estrito` | runbooks, procedures, outputs for agents |
| `descritivo` | system documentation |
| `leve` | internal announcements |
| `científico` | papers, proposals, technical reports with sources |
| `smart-note` | notes for a second brain; combines with another level (`smart-note + estrito`) |

Planned, outside `SKILL.md` and with no context cost: `retórico`, `ensaístico`, `crônico`,
`anedótico`, `machado-writer`, `saramago-writer` (source: [`site/data/planned.tsv`](site/data/planned.tsv)).

> [!WARNING]
> An author level imitates style and never copies text. Citation and public-domain rules are in
> [`REUSE.md`](REUSE.md).

## rules

The text of each rule is in [`SKILL.md`](SKILL.md) and [on the site](https://h0ffmann.github.io/ptbr-writer/#regras).
The origin of each one comes from [`site/data/origins.tsv`](site/data/origins.tsv). The
`cientifico` skill applies PTW-1 to PTW-13 and adds CIE-1 to CIE-12.

| id | rule | origin | creator |
| :- | :--- | :----- | :------ |
| PTW-1 | Quem faz, aparece (the actor appears) | PTC-1 | [@kayquer](https://github.com/kayquer) |
| PTW-2 | Uma proposição por frase (one proposition per sentence) | PTC-2 | [@kayquer](https://github.com/kayquer) |
| PTW-3 | Modalidade e quantidade explícitas (explicit modality and quantity) | PTC-3 | [@kayquer](https://github.com/kayquer) |
| PTW-4 | Verbo pleno (full verb) | PTC-4 | [@kayquer](https://github.com/kayquer) |
| PTW-5 | Sintaxe plana (flat syntax) | PTC-5 | [@kayquer](https://github.com/kayquer) |
| PTW-6 | Termos congelados (frozen terms) | PTC-6 | [@kayquer](https://github.com/kayquer) |
| PTW-7 | Números, datas e unidades (numbers, dates and units) | PTC-7 | [@kayquer](https://github.com/kayquer) |
| PTW-8 | Ortografia PT-BR vigente (current PT-BR spelling) | PTC-8 | [@kayquer](https://github.com/kayquer) |
| PTW-9 | Toda afirmação tem marca (every claim has a mark) | ouro-paper; Audience and evidence | [@h0ffmann](https://github.com/h0ffmann) |
| PTW-10 | Nenhuma referência inventada (no invented reference) | ouro-paper | [@h0ffmann](https://github.com/h0ffmann) |
| PTW-11 | Número com unidade e origem (a number with unit and origin) | ouro-paper; Audience and evidence | [@h0ffmann](https://github.com/h0ffmann) |
| PTW-12 | Sem alegação de pioneirismo (no priority claim) | Vale style `PriorityClaims` | [@h0ffmann](https://github.com/h0ffmann) |
| PTW-13 | Fato separado de análise (fact apart from analysis) | ouro-paper | [@h0ffmann](https://github.com/h0ffmann) |

## reading time

Every text the plugin generates carries a line such as `Estimated reading time: ~3 min (612 words, 4 links, 2 code blocks)`,
computed by `ptbr-reading-time` (static binary, no dependencies), never guessed.
It is an estimate: the same text always gives the same number, but people read 175 to 300 words per minute.

| Part | How it counts |
|------|---------------|
| Prose | words (no code blocks, URLs or Markdown syntax; inline code counts as words) ÷ 238 per minute |
| Complexity | technical text is slower: ×1.15 if the mean sentence has more than 20 words and ×1.15 if more than 25% of the words have 10 letters or more (1.0 to 1.3) |
| Code | 10 s per non-blank line of a code block |
| Links | 5 s per link, to decide whether to follow it; the time to read the target is not counted |
| Result | rounded up to whole minutes; 60 s or less prints `< 1 min` |

The 238 words per minute rate is silent reading of English non-fiction in the meta-analysis of Brysbaert (2019).
⚠ The paper gives no Portuguese rate we could read (it only says other languages can be estimated from the number of words
they need to say what English says); pt-BR uses the same 238, unscaled. The complexity thresholds and the code and link
times are this project's choices, not the paper's.

```console
bash plugins/ptbr-writer/bin/run.sh reading-time README.md   # --lang en, --json, --self-test
```

## ELI5

Three real requests, generated by the plugin itself; the PDF comes from the `.md` through `to-pdf.sh`. The numbers come from `claude -p --output-format json` ([`RESULTS.tsv`](docs/eli5/RESULTS.tsv)). The prompts and texts are in pt-BR.

| Prompt | PDF | Tokens | Level |
| :-- | :-- | --: | :-- |
| [Runbook to redo the site deploy](docs/eli5/1-runbook-deploy.prompt.md) | [PDF](docs/eli5/ptbr-writer-20261021T0434Z-runbook-deploy-site-estrito.pdf) | 350,529 (input 16, output 8,184, cache 342,329) | estrito |
| [Notice about the reading time](docs/eli5/2-comunicado-tempo-leitura.prompt.md) | [PDF](docs/eli5/ptbr-writer-20261021T0436Z-comunicado-tempo-leitura-leve.pdf) | 184,688 (input 10, output 2,401, cache 182,277) | leve |
| [Explain ptbr-writer to a 5-year-old](docs/eli5/3-explique-ptbr-writer.prompt.md) | [PDF](docs/eli5/ptbr-writer-20261021T0436Z-explicacao-para-criancas-descritivo.pdf) | 232,366 (input 12, output 3,434, cache 228,920) | descritivo |

## figures

Each figure is an Archify page, published on the site:

- [What happens to a ptbr request](https://h0ffmann.github.io/ptbr-writer/figuras/em-um-minuto.html)
- [Where the .pdf comes from](https://h0ffmann.github.io/ptbr-writer/figuras/pdf-dependencias.html)

And the code graph of this repository, built on each site build by the `code-graph` of
[h0ffmann/picasso](https://github.com/h0ffmann/picasso) (tree-sitter, no model):
[Code graph](https://h0ffmann.github.io/ptbr-writer/grafo/graph.html). The nodes that use picasso
lead into [picasso's graph](https://h0ffmann.github.io/ptbr-writer/grafo/picasso.html), built from the
`forks/picasso` submodule at the same commit as the other pins.

## roadmap

Source: [`site/data/roadmap.tsv`](site/data/roadmap.tsv). The first column keeps the Portuguese names
that the site and the README guard read.

| Piece | Status | What is missing or what exists |
| :---- | :----- | :----------------------------- |
| [`SKILL.md`](SKILL.md) | in progress | PTW rules and levels of rigor; the rules are still marked as in development |
| [`evals/`](evals/cientifico.md) | in progress | regression cases written, not yet run by a harness ⚠ |
| [skill científica](skills/cientifico/SKILL.md) | in progress | CIE rules; cases in `evals/cientifico-cie.md`, not yet run ⚠ |
| [roteamento](skills/ptbr-routing/SKILL.md) | done | `route` picks the skill and the level before a skill loads |
| [plugin](plugins/ptbr-writer/.claude-plugin/plugin.json) | done | Claude Code plugin, installed with one command at the release version; OpenCode reads `SKILL.md` directly |
| [distribuição](CHANGELOG.md) | done | release by tag with `just release`; v0.2.0 is the official version, installed by tag |
| [opencode](scripts/install-opencode.sh) | in progress | skills and the `ask-levels` plugin for OpenCode, installed by tag; tested on OpenCode 1.19.0 (skills and instruction), the `question` tool was not measured ⚠; the script ships in the next release ⚠ |
| [retórica](skills/retorica/SKILL.md) | planned | only the skeleton, no rules |

## awesome-ptbr-writer

Other projects for people who write technical text in Portuguese (source: [`site/data/awesome.tsv`](site/data/awesome.tsv)).

| Repository | What it gives |
| :--------- | :------------ |
| [kayquer/portugues-tecnico-controlado](https://github.com/kayquer/portugues-tecnico-controlado) | the original: 8 rules of controlled technical Portuguese |
| [fabricioctelles/skills](https://github.com/fabricioctelles/skills) | "humanizar" skill: removes AI-text patterns in pt-BR |
| [vale-cli/vale](https://github.com/vale-cli/vale) | prose linter with its own styles |
| [hunspell/hunspell](https://github.com/hunspell/hunspell) | spell checking with pt-BR dictionaries |
| [ajdavidl/Portuguese-NLP](https://github.com/ajdavidl/Portuguese-NLP) | index of NLP resources for Portuguese |

## development

Plugin runtime: static Rust binaries (linux x86_64 and aarch64), no python3 or node. Requirements to develop: [Nix](https://nixos.org/download/) with flakes. Rules for agents and contributors:
[`AGENTS.md`](AGENTS.md).

```console
nix develop           # the development shell
just quality          # everything CI checks, including the README
just site             # the site in site/dist
just release-due      # did the plugin change since the last tag?
```

Without Claude Code (OpenCode or another agent that reads skills): copy [`SKILL.md`](SKILL.md) to
the agent's skills folder.

## credits

- Rules PTW-1 to PTW-8: the PTC by [@kayquer](https://github.com/kayquer) (MIT).
- PTW-9 to PTW-13: the `ouro-paper` skill and the `.vale/styles/research` style of
  [marola-dev/marola-research](https://github.com/marola-dev/marola-research) (`041cedd`), and
  "Audience and evidence" of [h0ffmann/ww3-gpu](https://github.com/h0ffmann/ww3-gpu) (`319e4e2`).
- Scientific skill: `scientific-writing` by
  [K-Dense-AI/scientific-agent-skills](https://github.com/K-Dense-AI/scientific-agent-skills/tree/a1043ef544e3a100c28e70a736b6cf32554d0677/skills/scientific-writing)
  (`a1043ef`, MIT, Copyright (c) 2025 K-Dense Inc.) and `artigo` by
  [labdaps/labskills](https://github.com/labdaps/labskills/tree/246c1646a51a5a754aedf786fef0544d528064e7/skills/artigo)
  (`246c164`, MIT, Copyright (c) 2026 LABDAPS), in our own words. Requested citation: Kassis, T.,
  Agarwal, V., He, Y., Patel, D., & Brueckner, A. M. (2026). *Scientific Agent Skills: A Library of
  Procedural Knowledge for Research Agents*. arXiv:2609.00065.
  <https://doi.org/10.48550/arXiv.2609.00065>. The CIE rules on nothing invented and on the human
  deciding also come from the invariants of [h0ffmann/closed-source](https://github.com/h0ffmann/closed-source)
  (`563c339`) and of [ouroboric-dev/ouroboric](https://github.com/ouroboric-dev/ouroboric) (`2858c11`).
- smart-note: *How to Take Smart Notes*, by Sönke Ahrens; Niklas Luhmann's Zettelkasten; Robert M.
  Pirsig's card file for *Lila* ⚠.
- Rhetoric: the epigraph is from Pirsig, *Zen and the Art of Motorcycle Maintenance* (1974), checked
  against two public reproductions, not the printed edition ⚠. Basis: Plato's *Phaedo* and *Phaedrus*.
- Author level: the skill [orwell-writing](https://github.com/tamdogood/builder-essential-skills/tree/1be9984/skills/orwell-writing), by @tamdogood (MIT).
- Reading time: Brysbaert, M. (2019). How many words do we read per minute? A review and meta-analysis of reading rate. *Journal of Memory and Language*, 109, 104047. <https://doi.org/10.1016/j.jml.2019.104047> (238 words per minute, English non-fiction).
- Figures: [Archify](https://github.com/tt-a1i/archify) (@tt-a1i, MIT), the submodule `vendor/archify` pinned at v3.0.1
  (`2ab3cae`), used only to draw `docs/archify/`; it is not part of the plugin. Look: [h0ffmann/picasso](https://github.com/h0ffmann/picasso).
- Plugin and routing in the mould of the plugin and the `routing` skill of [marola-dev/marola-devkit](https://github.com/marola-dev/marola-devkit),
  by [@brunogbv](https://github.com/brunogbv).
  README layout: [h0ffmann/picasso](https://github.com/h0ffmann/picasso) and ww3-gpu. Site stack:
  closed-source and ww3-gpu; Lustre (MIT) and the Gleam stdlib (Apache-2.0) only at build time;
  Google Fonts (SIL OFL); the flag and signature of Machado de Assis from Wikimedia Commons, in the
  public domain.

## license

MIT. See [`LICENSE`](LICENSE).

<sub>ptbr-writer is a conceptual, independent fork of <a href="https://github.com/kayquer/portugues-tecnico-controlado">Português Técnico Controlado (PTC)</a>, by <a href="https://github.com/kayquer">kayquer</a> (MIT). The original methodology and rules are from there.</sub>
