# AGENTS.md

Instructions for AI agents working on `ptbr-writter`.

## Goal

`ptbr-writter` is a personalized, OpenCode-first rewrite of the [Português Técnico Controlado (PTC)](https://github.com/kayquer/portugues-tecnico-controlado) rules for Brazilian Portuguese technical writing.

## Hard rules

- **No secrets in code.** Placeholders only; use `.env.example` if needed.
- **One feature, one PR.** Keep commits small and focused.
- **Commit trailers.** Use `Tested:` and `Co-Authored-By: Claude <noreply@anthropic.com>`.
- **An author level respects the author's rights.** A level that imitates a writer (`machado-writter`, `saramago-writter`) never copies a passage; it cites the author as `REUSE.md` says, and `machado-writter` always cites Machado de Assis. Check that a new author's work is in the public domain before adding one, and write its rule in `REUSE.md`.
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
- `README.md` — human-facing documentation.
- `lint/saramago/` — Vale prose linter for Saramago's punctuation; `just saramago FILE`, self-test `just saramago-check` (part of `just quality`).
- `REUSE.md` — reuse rules for the author levels (citation, public domain, style only).
- `plugins/ptbr-writter/` — the Claude Code plugin, listed by `.claude-plugin/marketplace.json`; its `skills/` are symlinks to the repo's skills (no copies). Code and plugin share one version: `plugin.json`'s `version`, a `CHANGELOG.md` heading and the tag `vX.Y.Z` (`just release X.Y.Z`: a release PR, then the tag on main; `release.yml` runs `just release-check`). Installs see a change only when the version moves: a change to a plugin file (`just release-due` lists them) is not out until a release. A new skill in the plugin needs its link.
- `skills/ptbr-routing/` — the `ptbr-routing` skill: `scripts/route.py` (stdlib) picks the skill and the level from `routes.tsv`, and `scripts/session-check.sh` is the SessionStart hook that offers the plugin; outside any plugin, vendored as `.claude/skills/ptbr-routing` (here a symlink). A new level in `SKILL.md` needs its row, or `just quality` fails.
- `skills/retorica/SKILL.md` — a second skill, rhetoric from Plato's *Phaedo* and *Phaedrus* and Pirsig's Phaedrus (ZAMM); a skeleton with no rules yet.
- `site/` — the website (Gleam + Lustre, prerendered). `site/content` writes its facts from `SKILL.md`, `justfile` and `site/data/*.tsv` (`awesome.tsv`: related repos, each with its curated source; `roadmap.tsv`: production readiness; `origins.tsv`: each rule's source repo and creator's GitHub login (never a full name) — a new rule needs its row or the build fails); never type a rule, level, recipe, repo or status into `site/src`. Look rules: `DESIGN.md`.
- `forks/picasso` — submodule; its `labs/gleam-website` is the site toolchain, pinned in `flake.lock` to the same revision (`just pins`).

## Before changing `SKILL.md`

- Read the original PTC rules to understand the lineage.
- Keep changes scoped to one rule or one level of rigor at a time.
- Add a regression case when a rule changes behavior.
