<div align="center">

<img alt="A writing quill in a single line: original drawing" src="site/static/favicon.svg" width="120"/>

# ptbr-writer

<sub><a href="#install">install</a> · <a href="#use">use</a> · <a href="#levels">levels</a> · <a href="#rules">rules</a> · <a href="#development">development</a> · <a href="#credits">credits</a> · <a href="CHANGELOG.md">versions</a> · <a href="https://h0ffmann.github.io/ptbr-writer/">the site</a></sub>

[![site](https://github.com/h0ffmann/ptbr-writer/actions/workflows/site.yml/badge.svg)](https://github.com/h0ffmann/ptbr-writer/actions/workflows/site.yml)
[![version](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fraw.githubusercontent.com%2Fh0ffmann%2Fptbr-writer%2Fmain%2Fplugins%2Fptbr-writer%2F.claude-plugin%2Fplugin.json&query=%24.version&prefix=v&label=version&style=flat-square)](CHANGELOG.md)
[![license: MPL-2.0](https://img.shields.io/badge/license-MPL--2.0-1682D4?style=flat-square)](#license)
[![estimated reading time: ~8 min](https://img.shields.io/badge/reading%20time-~8%20min-1682D4?style=flat-square)](#reading-time)
[![plugin: Claude Code](https://img.shields.io/badge/plugin-Claude%20Code-1682D4?style=flat-square)](#install)
[![browser extension (Chrome): local dev (WIP)](https://img.shields.io/badge/browser%20extension-local%20dev%20(WIP)-1682D4?style=flat-square&logo=googlechrome&logoColor=white)](#chrome)
[![browser extension (Firefox): local dev (WIP)](https://img.shields.io/badge/browser%20extension-local%20dev%20(WIP)-1682D4?style=flat-square&logo=firefoxbrowser&logoColor=white)](docs/EXTENSION_FIREFOX.md)
[![local model: Ollama](https://img.shields.io/badge/local%20model-Ollama-1682D4?style=flat-square&logo=ollama&logoColor=white)](#ollama)
[![decision model: Ollaya](https://img.shields.io/badge/decision%20model-Ollaya-1682D4?style=flat-square&logo=data:image/svg%2Bxml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAzMiAzMiI+PHJlY3Qgd2lkdGg9IjMyIiBoZWlnaHQ9IjMyIiByeD0iNyIgZmlsbD0iIzE3MTcxNyIvPjxnIHRyYW5zZm9ybT0idHJhbnNsYXRlKDEuNiAxLjQpIHNjYWxlKC45KSIgZmlsbD0ibm9uZSIgc3Ryb2tlPSIjZmZmIiBzdHJva2Utd2lkdGg9IjIuMyIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIj48cGF0aCBkPSJNNiA1LjUgMTAuNSA5LjVRMTYgNy41IDIxLjUgOS41TDI2IDUuNVYxOVEyNiAyNy41IDE2IDI3LjVUNiAxOVoiLz48Y2lyY2xlIGN4PSIxMS41IiBjeT0iMTUuNSIgcj0iMy4yNSIvPjxjaXJjbGUgY3g9IjIwLjUiIGN5PSIxNS41IiByPSIzLjI1Ii8+PGNpcmNsZSBjeD0iMTEuNSIgY3k9IjE1LjUiIHI9IjEuMiIgZmlsbD0iI2ZmZiIgc3Ryb2tlPSJub25lIi8+PGNpcmNsZSBjeD0iMjAuNSIgY3k9IjE1LjUiIHI9IjEuMiIgZmlsbD0iI2ZmZiIgc3Ryb2tlPSJub25lIi8+PHBhdGggZD0iTTE0Ljc1IDIxIDE2IDIzbDEuMjUtMiIvPjwvZz48L3N2Zz4K)](#ollaya)
[![Zenodo: preprint](https://img.shields.io/badge/Zenodo-preprint-1682D4?style=flat-square&logo=zenodo&logoColor=white)](https://doi.org/10.5281/zenodo.23554531)

</div>

🇧🇷 [Leia em português](https://github.com/h0ffmann/ptbr-writer/blob/main/README.md) · this is the English version

## in one minute

- **Rewrites technical text in Brazilian Portuguese for a single reading:** explicit subject, one
  proposition per sentence, consistent terms. It serves text that people and agents read.
- **Each request goes through a level of rigor**, and the skill states the level it assumed.
- **It does not change thesis, authorship or conclusion, and does not look up sources.** It marks
  with `⚠` what it did not check.

![Claude Code plugin flow: you write ptbr; the first time, 4 questions saved in ptbr-writer.toml; then the ptbr-writer skill (or cientifico, if the text needs sources) writes one text per level and language; the output is the reply in the console and, if asked, .txt, .md, .pdf or notes files in the folder.](docs/img/en/em-um-minuto.png)

<details>
<summary>How to read this figure</summary>

**Takeaway.** You write `ptbr: …`, and the skill writes one text per level and per language.

**How to read.** Left to right. The top lane is the path of every request; the bottom one, the
configuration and the files. The orange border marks where you come in and what you get back.

**Not shown.** The rules, the `smart-note` level and the `ptbr reconfigurar` request.

**Evidence.** [`docs/archify/en/em-um-minuto.json`](docs/archify/en/em-um-minuto.json), the source of
the figure.

</details>

## install

The official version is [v0.3.0](https://github.com/h0ffmann/ptbr-writer/releases/tag/v0.3.0). In a
terminal:

```console
claude plugin install ptbr-writer --marketplace h0ffmann/ptbr-writer#v0.3.0
```

To pin the version in a repository, put in its `.claude/settings.json` the marketplace
`h0ffmann/ptbr-writer` with `"ref": "v0.3.0"` under `extraKnownMarketplaces`, and enable
`ptbr-writer@ptbr-writer` under `enabledPlugins`. To update, change the tag.

### OpenCode

The script `scripts/install-opencode.sh` installs the skills and the `ask-levels` plugin from a tag.
⚠ It ships in the next release; until then, run the script from a clone.

### Chrome

An extension, not yet released, underlines in the field where you write what the level asks to
change.

1. Run `just ext-build`.
2. In `chrome://extensions`, turn on **Developer mode** and load `extension/dist/`.
3. In the popup, choose the provider and the model, and click **Salvar** (save).

Nothing in the text changes without your click on **Aceitar** (accept). Firefox 128+ uses
`just ext-build-firefox` ([guide](docs/EXTENSION_FIREFOX.md), pt-BR). The providers:

- **Claude or DeepSeek:** with your key.
- <a id="ollama"></a>**Ollama (local):** `just ext-ollama` serves a model on this machine, with no
  key.
- <a id="ollaya"></a>**Ollaya (local, só aponta)** (points only): `just ext-ollaya` serves a decision
  model. It points out the rule that each sentence breaks and does not write the replacement.
- **Ollaya + Ollama:** with Ollama, the switch **Ollaya escolhe o estilo** (Ollaya picks the style)
  makes Ollaya pick the tone (concise, Socratic, friendly) that Ollama writes in.

⚠ The local providers were not measured on pt-BR.

**🔊 Sound**, off by default, in the popup and the options: a short cue when suggestions appear, on
accept, on ignore and on an error, and a 🔊 on the card that reads the suggestion aloud. The cues are
made in code, with no audio file, and the reading uses only a local pt-BR voice of the system: no
text goes to a voice server. The impact: +3,740 bytes in the extension and, when on, ~22 to 46 ms
on the first cue ([`docs/EXTENSION_SOUND.md`](docs/EXTENSION_SOUND.md), pt-BR).

## use

| Command | Does |
| :------ | :--- |
| `/ptbr-writer:ptbr` | rewrites at the requested level, or classifies the text and states the level |
| `/ptbr-writer:cientifico` | scientific text with sources: paper, proposal, report |
| `/ptbr-writer:bipr` | opens the branch's PR with the body in pt-BR and the English mirror |

Or describe the task: "rewrite this runbook in controlled Portuguese".

A request that starts with `ptbr` uses `ptbr-writer.toml`. The first time, Claude asks for the
levels, the formats, the folder and the languages, and saves the answers:

```toml
levels = ["estrito"]     # empty: the skill classifies and states the level
formats = ["md", "pdf"]  # txt, md, pdf, notes
folder = "docs"
languages = ["pt-BR"]    # and/or "asd-ste100"
```

To change it, edit the file or write `ptbr reconfigurar`. The request wins over the file: `ptbr
leve: ...` uses `leve`. The files are named `ptbr-writer-<date>-<goal>-<level>.<ext>`.

## levels

| Level | Use |
| :---- | :-- |
| `estrito` | runbooks, procedures, output for agents |
| `descritivo` | system documentation |
| `leve` | internal announcements |
| `científico` | papers, proposals, technical reports with sources |
| `smart-note` | notes for a second brain; combines with another level |

> [!WARNING]
> An author level, still planned, imitates the style and never copies text ([`REUSE.md`](REUSE.md)).

## rules

Rules PTW-1 to PTW-13 are in [`SKILL.md`](SKILL.md). The `cientifico` skill adds the CIE rules. The
[site](https://h0ffmann.github.io/ptbr-writer/) shows each rule with its origin, the roadmap, the
related projects and the figures:

- [Claude Code plugin flow](https://h0ffmann.github.io/ptbr-writer/figuras/em-um-minuto.html)
- [Browser extension flow](https://h0ffmann.github.io/ptbr-writer/figuras/extensao-fluxo.html)
- [Local model flow](https://h0ffmann.github.io/ptbr-writer/figuras/modelo-local.html)
- [Where the .pdf comes from](https://h0ffmann.github.io/ptbr-writer/figuras/pdf-dependencias.html)

The whitepaper, in 8 pages, is in [`WHITEPAPER.en.pdf`](WHITEPAPER.en.pdf) and, in Portuguese, in
[`WHITEPAPER.pdf`](WHITEPAPER.pdf).

## reading time

Every text the plugin writes carries a reading estimate, computed by the `ptbr-reading-time`
binary, never by guess. This README has a 10-minute cap, which `just quality` checks.

## development

Requirement: [Nix](https://nixos.org/download/) with flakes. The rules for contributors, people or
agents, are in [`AGENTS.md`](AGENTS.md).

```console
nix develop      # the development shell
just quality     # everything CI checks
```

## credits

- PTW-1 to PTW-8: the PTC by @kayquer (MIT).
- PTW-9 to PTW-13: the `ouro-paper` skill of marola-dev/marola-research and "Audience and evidence"
  of h0ffmann/ww3-gpu.
- Scientific skill: `scientific-writing` of K-Dense-AI/scientific-agent-skills (MIT, Copyright (c)
  2025 K-Dense Inc.) and `artigo` of labdaps/labskills (MIT, Copyright (c) 2026 LABDAPS), in our own
  words. Requested citation: Kassis, T. et al. (2026), *Scientific Agent Skills*, arXiv:2609.00065.
- smart-note: Sönke Ahrens, Niklas Luhmann and Robert M. Pirsig. Rhetoric: Plato and Pirsig.
- Author level: the `orwell-writing` skill by @tamdogood (MIT).
- Reading time: Brysbaert (2019), *Journal of Memory and Language* 109, 104047.
- Browser extension: @robkler, in #51.
- Figures: Archify, by @tt-a1i (MIT). Plugin and routing: marola-dev/marola-devkit, by @brunogbv.
- Site: Lustre (MIT), the Gleam stdlib (Apache-2.0), fonts from Google Fonts (SIL OFL), and the flag
  and Machado de Assis's signature from Wikimedia Commons, in the public domain.

The pinned commit of each source is in [`AGENTS.md`](AGENTS.md).

## license

MPL-2.0. See [`LICENSE`](LICENSE). Who distributes a modified file of ptbr-writer publishes that
file under the MPL. The versions before v0.3.0 stay MIT. What comes from third parties keeps
each one's license and notice.

<sub>ptbr-writer is a conceptual, independent fork of Português Técnico Controlado (PTC), by kayquer
(MIT). The original methodology and rules are from there.</sub>
