# AGENTS.md

Instructions for AI agents working on `ptbr-writter`.

## Goal

`ptbr-writter` is a personalized, OpenCode-first rewrite of the [Português Técnico Controlado (PTC)](https://github.com/kayquer/portugues-tecnico-controlado) rules for Brazilian Portuguese technical writing.

## Hard rules

- **No secrets in code.** Placeholders only; use `.env.example` if needed.
- **One feature, one PR.** Keep commits small and focused.
- **Commit trailers.** Use `Tested:` and `Co-Authored-By: Claude <noreply@anthropic.com>`.
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
- `.opencode.json` — OpenCode configuration pointing to local `llamacpp/qwen3.6-27b`.
- `justfile` — recipes for running OpenCode, serving Qwen, and quality checks.
- `flake.nix` / `flake.lock` — reproducible dev shell.
- `evals/` — regression cases for rules (not yet run by a harness).
- `README.md` — human-facing documentation.
- `site/` — the website (Gleam + Lustre, prerendered). `site/content` writes its facts from `SKILL.md` and `justfile`; never type a rule, level or recipe into `site/src`. Look rules: `DESIGN.md`.
- `forks/picasso` — submodule; its `labs/gleam-website` is the site toolchain, pinned in `flake.lock` to the same revision (`just pins`).

## Before changing `SKILL.md`

- Read the original PTC rules to understand the lineage.
- Keep changes scoped to one rule or one level of rigor at a time.
- Add a regression case when a rule changes behavior.
