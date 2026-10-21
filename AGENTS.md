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
- `skills/cientifico/SKILL.md` — the scientific-writing skill (rules CIE); applies PTW-1..17 and adds its own.
- `skills/bipr/SKILL.md` — `/bipr`: the branch's commits to a pushed PR with a bilingual body (pt-BR, then the English mirror); user-invoked only (`disable-model-invocation`). The PR body ends with `skills/bipr/footer.md` (the favicon and "Generated with h0ffmann/ptbr-writer bipr skill"), kept out of SKILL.md because a skill body has no links; the skill may later move to a devkit or skills repo.
- `scripts/install_current.sh` — installs the plugin from the tip of main (or `PTBR_REF`), no release: removes the old install and marketplace, installs again (main keeps the last version number, so `plugin update` sees nothing). For trying unreleased work; the tag stays the stable install.
- `.opencode.json` — OpenCode configuration pointing to local `llamacpp/qwen3.6-27b`.
- `justfile` — recipes for running OpenCode, serving Qwen, and quality checks.
- `flake.nix` / `flake.lock` — reproducible dev shell.
- `evals/` — regression cases for rules (not yet run by a harness).
- `README.md` — human-facing documentation (pt-BR); `README.en.md` is its English mirror.
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
- `extension/` — the Chrome extension (TypeScript, esbuild, Vitest, Playwright): underlines in a text field what a level asks to change, through Claude or DeepSeek with the user's key. It reads `SKILL.md` and `skills/cientifico/SKILL.md` at build; not part of the plugin. `just ext-build`, `just ext-test` (part of `just quality`), `just ext-e2e`; CI: `.github/workflows/extension.yml`.
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
never the HTML. `just figure-init`, `just figure`, `just figure-check` (the last runs in `just quality`).
