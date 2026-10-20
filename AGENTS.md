# AGENTS.md

Instructions for AI agents working on `ptbr-writter`.

## Goal

`ptbr-writter` is a personalized, OpenCode-first rewrite of the [Português Técnico Controlado (PTC)](https://github.com/kayquer/portugues-tecnico-controlado) rules for Brazilian Portuguese technical writing.

## Hard rules

- **No secrets in code.** Placeholders only; use `.env.example` if needed.
- **One feature, one PR.** Keep commits small and focused.
- **Commit trailers.** Use `Tested:` and `Co-Authored-By: Claude <noreply@anthropic.com>`.
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
- `skills/cientifico/SKILL.md` — the scientific-writing skill (rules CIE); applies PTW-1..13 and adds its own.
- `.opencode.json` — OpenCode configuration pointing to local `llamacpp/qwen3.6-27b`.
- `justfile` — recipes for running OpenCode, serving Qwen, and quality checks.
- `flake.nix` / `flake.lock` — reproducible dev shell.
- `evals/` — regression cases for rules (not yet run by a harness).
- `README.md` — human-facing documentation.
- `skills/retorica/SKILL.md` — a second skill, rhetoric from Plato's *Phaedo* and *Phaedrus* and Pirsig's Phaedrus (ZAMM); a skeleton with no rules yet.
- `site/` — the website (Gleam + Lustre, prerendered). `site/content` writes its facts from `SKILL.md`, `justfile` and `site/data/*.tsv` (`awesome.tsv`: related repos, each with its curated source; `roadmap.tsv`: production readiness; `origins.tsv`: each rule's source repo and creator's GitHub login (never a full name) — a new rule needs its row or the build fails); never type a rule, level, recipe, repo or status into `site/src`. Look rules: `DESIGN.md`.
- `forks/picasso` — submodule; its `labs/gleam-website` is the site toolchain, pinned in `flake.lock` to the same revision (`just pins`).

## Before changing `SKILL.md`

- Read the original PTC rules to understand the lineage.
- Keep changes scoped to one rule or one level of rigor at a time.
- Add a regression case when a rule changes behavior.
