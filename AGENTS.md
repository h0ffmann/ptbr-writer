# AGENTS.md

Instructions for AI agents working on `ptbr-writer`.

## Goal

`ptbr-writer` is a personalized, OpenCode-first rewrite of the [Português Técnico Controlado (PTC)](https://github.com/kayquer/portugues-tecnico-controlado) rules for Brazilian Portuguese technical writing.

## Goals

- **Transparent.** Every fact on the site and in the README comes from a file in the repo or a pinned commit, and every dependency is visible (the figure of the `.pdf` dependency tree is one).
- **Deterministic.** Versions are pinned: the plugin tag, the picasso commit, `flake.lock`, `Cargo.lock`. The same input gives the same output.
- **Usage.** One command to install, one request to use; the site leads with the install.

## Hard rules

- **No secrets in code.** Placeholders only; use `.env.example` if needed.
- **One feature, one PR.** Keep commits small and focused.
- **Commit trailers.** Use `Tested:` and `Co-Authored-By: Claude <noreply@anthropic.com>`.
- **An author level respects the author's rights.** A level that imitates a writer (`machado-writer`, `saramago-writer`) never copies a passage; it cites the author as `REUSE.md` says, and `machado-writer` always cites Machado de Assis. Check that a new author's work is in the public domain before adding one, and write its rule in `REUSE.md`.
- **No credits in a skill's body.** A skill file (`SKILL.md`, `skills/*/SKILL.md`) holds instructions only: no credits, sources, links, commit pins, author names or history. They are loaded into other people's context. Credits go to `README.md` and `site/data/origins.tsv`.
- **No Python.** Python is not allowed in this repository. The plugin's runtime tools are static Rust binaries built by h0ffmann/picasso's `labs/ptbr-tools` and pinned by `plugins/ptbr-writer/bin/tools.lock`; the site and its generators are Gleam.
- **No paid resources** without explicit human confirmation.

## Setup

```bash
nix develop        # enters the dev shell
just               # list recipes
just quality       # run checks
just site          # build the website into site/dist
just opencode-qwen # serve Qwen3.6-27B and run OpenCode
```

## Repo layout

- `SKILL.md` — the skill prompt consumed by OpenCode / Claude Code.
- `skills/ptbr/SKILL.md` — a symlink to `SKILL.md`: the alias `/ptbr`. Edit `SKILL.md`, never the link.
- `skills/cientifico/SKILL.md` — the scientific-writing skill (rules CIE); applies PTW-1..13 and adds its own.
- `skills/bipr/SKILL.md` — `/bipr`: the branch's commits to a pushed PR with a bilingual body (pt-BR, then the English mirror); user-invoked only (`disable-model-invocation`). The PR body ends with `skills/bipr/footer.md` (the favicon and "Generated with h0ffmann/ptbr-writer bipr skill"), kept out of SKILL.md because a skill body has no links; the skill may later move to a devkit or skills repo.
- `scripts/install_current.sh` — installs the plugin from the tip of main (or `PTBR_REF`), no release: removes the old install and marketplace, installs again (main keeps the last version number, so `plugin update` sees nothing). For trying unreleased work; the tag stays the stable install.
- `.mcp.json` + `scripts/firefox-mcp.sh` — the Firefox MCP (Mozilla's `@mozilla/firefox-devtools-mcp`, pinned in the script, throwaway profile; on Ubuntu the snap's real binary, since geckodriver cannot start the `/usr/bin/firefox` wrapper): lets an agent install `extension/dist-firefox` (`install_extension`, type `path`) and drive the page. A dev tool, not part of the plugin.
- `.opencode.json` — OpenCode configuration pointing to local `llamacpp/qwen3.6-27b`.
- `justfile` — recipes for running OpenCode, serving Qwen, and quality checks.
- `flake.nix` / `flake.lock` — reproducible dev shell.
- `evals/` — regression cases for rules (not yet run by a harness).
- `README.md` — human-facing documentation (pt-BR); `README.en.md` is its English mirror. Hard cap: 10 minutes of reading time each, by the plugin's own calculator (`just quality` fails above it). Detail goes to the site (rules, origins, roadmap, awesome, ELI5, graphs), to `docs/`, or to *Reference* below, never back into the README.
- `.claude/skills/figure/` — the `figure` dev skill (not in the plugin): the rules for a Mermaid figure and its reading card.
- `lint/saramago/` — Vale prose linter for Saramago's punctuation; `just saramago FILE`, self-test `just saramago-check` (part of `just quality`).
- `REUSE.md` — reuse rules for the author levels (citation, public domain, style only).
- `ISSUE_LABELS.md` — the labels of issues and PRs (type, area, level, status, priority); `just labels` syncs them to GitHub.
- `.github/CODEOWNERS` — @h0ffmann and @robkler own every path; GitHub asks one of them to review each PR.
- `plugins/ptbr-writer/` — the Claude Code plugin, listed by `.claude-plugin/marketplace.json`; its `skills/` are symlinks to the repo's skills (no copies). Code and plugin share one version: `plugin.json`'s `version`, a `CHANGELOG.md` heading and the tag `vX.Y.Z` (`just release X.Y.Z`: a release PR, then the tag on main; `release.yml` runs `just release-check`). Installs see a change only when the version moves: a change to a plugin file (`just release-due` lists them) is not out until a release. A new skill in the plugin needs its link.
- `plugins/ptbr-writer/bin/` — `run.sh` picks `bin/<os>-<arch>/<binary>` (warns when none); `tools.lock` pins the picasso rev and each binary's sha256; `just tools-fetch REV` refreshes, `just tools-check` verifies.
- `plugins/ptbr-writer/hooks/` — `ask-levels.sh` runs the `ptbr ask-levels` binary (via `bin/run.sh`) on every prompt (UserPromptSubmit): a request that starts with `ptbr` gets one instruction built from `ptbr-writer.toml` (plugin defaults `ptbr-writer.default.toml` < `~/.config/ptbr-writer/` < `./ptbr-writer.toml` < the request); any other prompt gets nothing. Its self-test runs in `just plugin-check`.
- `plugins/ptbr-writer/evals/` — `claude plugin eval` cases (prompt.md + graders/), run by hand: `just plugin-eval` on your login, or `plugin.yml` → Run workflow on `ANTHROPIC_API_KEY`. `scripts/plugin-smoke.sh` installs the plugin into an empty config; `plugin.yml` runs it when the plugin changes.
- `scripts/install-opencode.sh` + `opencode/ptbr-writer.js` — the OpenCode install: from a tag, the skills `ptbr-writer`, `ptbr`, `cientifico` as real files (a `name:` is added where the source has none), the pinned binaries and the `ask-levels` plugin (`chat.message`, runs the same `ptbr ask-levels` via `bin/run.sh`). `scripts/opencode-smoke.sh` checks it in `just plugin-check`. Not in `release-due`: it is outside `plugins/ptbr-writer/`, but the tag must carry it.
- `skills/ptbr-routing/` — the `ptbr-routing` skill: the `ptbr-route` binary (`bin/run.sh`) picks the skill and the level from `routes.tsv`, and `scripts/session-check.sh` is the SessionStart hook that offers the plugin; outside any plugin, vendored as `.claude/skills/ptbr-routing` (here a symlink). A new level in `SKILL.md` needs its row, or `just quality` fails.
- `skills/retorica/SKILL.md` — a second skill, rhetoric from Plato's *Phaedo* and *Phaedrus* and Pirsig's Phaedrus (ZAMM); a skeleton with no rules yet.
- `extension/` — the Chrome extension (TypeScript, esbuild, Vitest, Playwright): underlines in a text field what a level asks to change, through Claude or DeepSeek with the user's key, or locally through Ollama (an LLM writes the replacement) or Ollaya (a decision model, `extension/src/decisions.ts`: one `/api/decide` per sentence, "which rule does it break?", an underline with no replacement); the two are independent flags, and with both on (`ollayaStyle`) Ollaya picks the conversation style and Ollama writes in it, alone when Ollaya is down. It reads `SKILL.md` and `skills/cientifico/SKILL.md` at build; not part of the plugin. `just ext-build`, `just ext-test` (part of `just quality`), `just ext-e2e`, `just ext-build-firefox` (Firefox 128+, `extension/dist-firefox/`), `just ext-ollama` (the Ollama provider, started with `OLLAMA_ORIGINS` for extension origins), `just ext-ollaya` (the Ollaya provider, with `OLLAYA_ORIGINS`; its models: `docs/SURVEY_DECISION_MODEL.md`); CI: `.github/workflows/extension.yml`.
- `site/` — the website (Gleam + Lustre, prerendered). `site/content` writes its facts from `SKILL.md`, `justfile` and `site/data/*.tsv` (`awesome.tsv`: related repos, each with its curated source; `roadmap.tsv`: production readiness; `origins.tsv`: each rule's source repo and creator's GitHub login (never a full name) — a new rule needs its row or the build fails); never type a rule, level, recipe, repo or status into `site/src`. Look rules: `DESIGN.md`.
- `forks/picasso` — submodule; its `labs/gleam-website` is the site toolchain, pinned in `flake.lock` to the same revision (`just pins`).

## Before changing `SKILL.md`

- Read the original PTC rules to understand the lineage.
- Keep changes scoped to one rule or one level of rigor at a time.
- Add a regression case when a rule changes behavior.

## Audience and evidence (hard rule)

- `⚠` marks a claim that was not checked. Never remove a `⚠` without doing the check, and never
  promote a projection to a measurement.
- A number without a command or a source does not enter a table.
- Prose is plain and direct. Cut a sentence because it repeats or teaches nothing, never to hit a
  length, and never pad one language's version to match the other's.
- `README.md` and `README.en.md` mirror each other: same sections, same facts. A change to one
  changes the other in the same PR.

## Figures (hard rule)

A diagram is a ```` ```mermaid ```` fence in the page it illustrates, starting with
`%% figure: <id>` and `%% title: <question>`, or a PNG drawn by `just figure` (below), followed by a *How to read this figure* card
(*Como ler esta figura* in the pt-BR page): takeaway, how to read, not shown, evidence. The
`figure` skill (`.claude/skills/figure/`) has the visual grammar. Where a figure and its prose
disagree, the figure is the bug. A PNG is drawn with Archify, vendored as the pinned submodule `vendor/archify`
(`.claude/skills/archify` links to it; it is never part of the plugin, so nothing of it goes under
`plugins/`): commit `docs/archify/<slug>.json`, `docs/archify/figures.lock` and `docs/img/<slug>.png`,
never the HTML. Every picture has an English twin, `docs/archify/en/<slug>.json` (`meta.locale` `en`, the same
lanes, nodes and edges, only the words translated) drawn to `docs/img/en/<slug>.png`: `README.md` embeds the
pt-BR PNG and `README.en.md` the English one, never the other. `just figure-init`, `just figure`,
`just figure-check` (the last runs in `just quality` and fails on a missing twin, a twin that is not the same
drawing, or a README that embeds the other language's PNG).

## Reference (moved from the README)

Facts that left the README when it was cut to 10 minutes. They stay true only while the code says
so; check the file before you repeat one.

**Install.**
- Inside Claude Code: `/plugin marketplace add h0ffmann/ptbr-writer#v0.2.0`, then
  `/plugin install ptbr-writer@ptbr-writer`. To update, remove the marketplace and add it again
  with the new tag; in a repository, change the `ref`.
- Pin in a repository's `.claude/settings.json`:
  `{"extraKnownMarketplaces": {"ptbr-writer": {"source": {"source": "github", "repo": "h0ffmann/ptbr-writer", "ref": "v0.2.0"}}}, "enabledPlugins": {"ptbr-writer@ptbr-writer": true}}`.
- `scripts/install_current.sh` (curl it inside Claude Code with `!`, then `/reload-plugins`)
  installs the tip of `main`, or `PTBR_REF`; `main` is not a fixed version.
- OpenCode: `scripts/install-opencode.sh vX.Y.Z` copies `ptbr-writer`, `ptbr` and `cientifico` as
  files to `~/.config/opencode/skills/` (`--project`: `./.opencode/`) and installs the `ask-levels`
  plugin; it needs `bash`, `curl` and `tar`, and the binaries are `linux-x86_64` and
  `linux-aarch64`. Tested on OpenCode 1.19.0: the skills load and the instruction reaches the model;
  ⚠ the model answering with the `question` tool was not measured (`opencode run` does not offer
  it). `ptbr-routing` is not installed: it depends on a Claude Code hook. ⚠ `v0.3.0` does not exist
  yet.

**Use.**
- `/ptbr-writer:ptbr-writer` is the same as `/ptbr-writer:ptbr`. A `bipr` example and its cost:
  `docs/bipr/exemplo-pr-49.md`.
- A request that starts with `ptbr` uses `ptbr-writer.toml`; ptbr-writer named mid-sentence
  triggers nothing. Layers, each over the one before: `plugins/ptbr-writer/ptbr-writer.default.toml`,
  `~/.config/ptbr-writer/ptbr-writer.toml`, `./ptbr-writer.toml`, the request. An unknown value or a
  folder outside the root is ignored with a warning. Keys: `levels`, `formats` (`txt`, `md`, `pdf`,
  `notes`), `folder` (relative, no `..`), `print` (what the console shows in full), `languages`
  (`pt-BR` and/or `asd-ste100`; the ASD-STE100 file ends in `-ste100`).
- File name: `ptbr-writer-<AAAAMMDDThhmmZ>-<objetivo>-<nível>.<ext>`, one UTC stamp per request;
  2 levels × 3 formats = 6 files. Suggest `ptbr-writer-*` in `.gitignore`. `notes` files follow
  smart-note (`<id>-<slug>.md`).
- `.pdf`: the machine's pandoc and xelatex; without them, after asking, Nix builds picasso's
  `labs/ptbr-pdf` (`to-pdf`, Rust, Typst, fonts embedded) at `PICASSO_REV`. Measured with
  `just pdf-bench` on picasso v0.2.0, fresh GitHub runner, 4 cores
  ([run](https://github.com/h0ffmann/ptbr-writer/actions/runs/39537436728)): 14 MiB download from 6
  cache.nixos.org paths, 72 MiB on disk, 25 MiB built, 892 s the first time.
- Planned levels, outside `SKILL.md`: `site/data/planned.tsv`.

**Browser extension.**
- A check runs after 1 s without typing, on fields of 3 words or more; password fields are skipped,
  Gmail's quoted history is not sent, and above 4000 characters only the cursor's paragraph goes.
- Default models: `claude-haiku-5-5`; `deepseek-flash` (⚠ read in DeepSeek's docs on 2026-10-21).
  Keys stay in `chrome.storage.local`. On by default: `web.whatsapp.com` (`leve`),
  `mail.google.com` and `github.com` (`descritivo`); other https sites by **Ativar neste site**.
- ⚠ Tested by hand on 2026-10-21 on WhatsApp Web with a real key; Gmail and GitHub not yet.
- Firefox: `about:debugging#/runtime/this-firefox`, load `extension/dist-firefox/manifest.json`,
  then grant the sites in `about:addons`; it goes away when Firefox closes. The `.mcp.json` Firefox
  MCP lets an agent install and drive it (`docs/EXTENSION_FIREFOX.md`).
- Ollama: `just ext-ollama [model]` (default `qwen3.6:27b`, the model of `just opencode-qwen`;
  `qwen3:8b` for a smaller GPU). Suggested: `qwen3.6:27b`, `qwen3:14b`, `qwen3:8b`, `qwen3:4b`, plus
  what Ollama has. Ollama answers 403 to an extension origin unless started with `OLLAMA_ORIGINS`;
  the recipe sets it and says how for a systemd Ollama. **Baixar** is very experimental (#63).
  ⚠ No review with a real Qwen 3 was measured.
- Ollaya: `just ext-ollaya [model]` (default `decima`, `docs/SURVEY_DECISION_MODEL.md`). One
  `/api/decide` per sentence, up to 32; a rule at 50% or more becomes an underline with only
  **Ignorar**. With **Ollaya escolhe o estilo** on, Ollaya picks `conciso`, `socrático` or `amigável`
  and Ollama writes in it; Ollaya down, Ollama writes alone. ⚠ Latency, accuracy and the threshold
  were not measured (#61).

**Reading time** (`ptbr-reading-time`): prose words (no code blocks, URLs or Markdown; inline code
counts) ÷ 238 per minute; ×1.15 when the mean sentence passes 20 words and ×1.15 when over 25% of
words have 10+ letters; 10 s per non-empty code line; 5 s per link; rounded up, `< 1 min` up to 60 s.
238 is Brysbaert (2019) for English non-fiction; ⚠ pt-BR uses the same rate unscaled, and the other
numbers are this project's choices. `bash plugins/ptbr-writer/bin/run.sh reading-time FILE`.

**Credits, pinned.** marola-research `041cedd`; ww3-gpu `319e4e2`; K-Dense scientific-writing
`a1043ef`; labdaps `artigo` `246c164`; closed-source `563c339` and ouroboric `2858c11` (CIE
invariants); orwell-writing `1be9984`; Archify v3.0.1 `2ab3cae`. Full citation: Kassis, T.,
Agarwal, V., He, Y., Patel, D., & Brueckner, A. M. (2026). *Scientific Agent Skills: A Library of
Procedural Knowledge for Research Agents*. arXiv:2609.00065. https://doi.org/10.48550/arXiv.2609.00065.
Brysbaert, M. (2019), https://doi.org/10.1016/j.jml.2019.104047. smart-note: *How to Take Smart
Notes* (Ahrens), Luhmann's Zettelkasten, Pirsig's slips for *Lila* ⚠. Retórica's epigraph: Pirsig,
*ZAMM* (1974), checked in two public reproductions, not the print edition ⚠.
