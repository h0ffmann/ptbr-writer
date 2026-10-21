# Changelog

Uma versão só: a tag `vX.Y.Z`, o `version` de `plugins/ptbr-writer/.claude-plugin/plugin.json` e o
cabeçalho abaixo andam juntos. `just release X.Y.Z` abre o PR da versão e, depois do merge, cria a tag; `release.yml` publica a release.

## v0.2.0 — 2026-10-21

- O projeto agora se chama **ptbr-writer** (antes `ptbr-writter`): o repositório
  `h0ffmann/ptbr-writer`, o marketplace e o plugin `ptbr-writer@ptbr-writer`, a skill
  `/ptbr-writer:ptbr-writer`, `ptbr-writer.toml` e os arquivos `ptbr-writer-<UTC>-...`. Quem instalou
  a v0.1.0 instala de novo: `claude plugin install ptbr-writer --marketplace h0ffmann/ptbr-writer#v0.2.0`.
- Um pedido que começa com `ptbr` pergunta uma vez os níveis, os formatos (`txt`, `md`, `pdf`,
  `notes`) e a pasta, grava `./ptbr-writer.toml` e não pergunta mais; `ptbr reconfigurar` refaz.
- `.pdf` pelo `to-pdf` em Rust com Typst do picasso (`labs/ptbr-pdf`), só com permissão para baixar.
- O site abre com o comando de instalação e um botão de copiar.

## v0.1.0 — 2026-10-20

- Plugin do Claude Code, publicado com o nome antigo: marketplace `ptbr-writter`, plugin
  `ptbr-writter` com as skills `ptbr-writter`, `ptbr` (alias) e `cientifico`.
- `ptbr-routing`: a skill fora do plugin que escolhe skill e nível, e o hook `session-check`.
