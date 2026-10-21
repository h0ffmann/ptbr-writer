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
- `.opencode.json` — OpenCode configuration pointing to local `llamacpp/qwen3.6-27b`.
- `justfile` — recipes for running OpenCode, serving Qwen, and quality checks.
- `flake.nix` / `flake.lock` — reproducible dev shell.
- `evals/` — regression cases for rules (not yet run by a harness).
- `README.md` — human-facing documentation (pt-BR); `README.en.md` is its English mirror.
- `.claude/skills/figure/` — the `figure` dev skill (not in the plugin): the rules for a Mermaid figure and its reading card.
- `lint/saramago/` — Vale prose linter for Saramago's punctuation; `just saramago FILE`, self-test `just saramago-check` (part of `just quality`).
- `REUSE.md` — reuse rules for the author levels (citation, public domain, style only).
- `plugins/ptbr-writer/` — the Claude Code plugin, listed by `.claude-plugin/marketplace.json`; its `skills/` are symlinks to the repo's skills (no copies). Code and plugin share one version: `plugin.json`'s `version`, a `CHANGELOG.md` heading and the tag `vX.Y.Z` (`just release X.Y.Z`: a release PR, then the tag on main; `release.yml` runs `just release-check`). Installs see a change only when the version moves: a change to a plugin file (`just release-due` lists them) is not out until a release. A new skill in the plugin needs its link.
- `plugins/ptbr-writer/hooks/` — `ask-levels.sh` runs `ask_levels.py` on every prompt (UserPromptSubmit): a request that starts with `ptbr` gets one instruction built from `ptbr-writer.toml` (plugin defaults `ptbr-writer.default.toml` < `~/.config/ptbr-writer/` < `./ptbr-writer.toml` < the request); any other prompt gets nothing. Its self-test runs in `just plugin-check`.
- `plugins/ptbr-writer/evals/` — `claude plugin eval` cases (prompt.md + graders/), run by hand: `just plugin-eval` on your login, or `plugin.yml` → Run workflow on `ANTHROPIC_API_KEY`. `scripts/plugin-smoke.sh` installs the plugin into an empty config; `plugin.yml` runs it when the plugin changes.
- `skills/ptbr-routing/` — the `ptbr-routing` skill: `scripts/route.py` (stdlib) picks the skill and the level from `routes.tsv`, and `scripts/session-check.sh` is the SessionStart hook that offers the plugin; outside any plugin, vendored as `.claude/skills/ptbr-routing` (here a symlink). A new level in `SKILL.md` needs its row, or `just quality` fails.
- `skills/retorica/SKILL.md` — a second skill, rhetoric from Plato's *Phaedo* and *Phaedrus* and Pirsig's Phaedrus (ZAMM); a skeleton with no rules yet.
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
