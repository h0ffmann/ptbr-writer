# Changelog

Uma versão só: a tag `vX.Y.Z`, o `version` de `plugins/ptbr-writter/.claude-plugin/plugin.json` e o
cabeçalho abaixo andam juntos. `just release-check vX.Y.Z` confere; `release.yml` publica a release.

## v0.1.0 — 2026-10-20

- Plugin do Claude Code: marketplace `ptbr-writter`, plugin `ptbr-writter` com as skills
  `ptbr-writter`, `ptbr` (alias) e `cientifico`.
- `ptbr-routing`: a skill fora do plugin que escolhe skill e nível, e o hook `session-check`.
