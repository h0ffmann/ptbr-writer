# Changelog

Uma versão só: a tag `vX.Y.Z`, o `version` de `plugins/ptbr-writer/.claude-plugin/plugin.json` e o
cabeçalho abaixo andam juntos. `just release X.Y.Z` abre o PR da versão e, depois do merge, cria a tag; `release.yml` publica a release.

## Unreleased

- Cada figura do Archify ganha um gêmeo em inglês (`docs/archify/en/<slug>.json`, desenhado em
  `docs/img/en/<slug>.png`): o `README.md` mostra as figuras em pt-BR e o `README.en.md` as em inglês. O
  `just figure-check` falha quando falta o gêmeo, quando o gêmeo não é o mesmo desenho, ou quando um README
  mostra o PNG do outro idioma.
- Figuras (#66): a figura do pedido `ptbr` vira o *fluxo do plugin do Claude Code*, e duas figuras
  novas do Archify mostram o *fluxo da extensão de navegador* (os três modos: chave de API, Ollama +
  Qwen, Ollaya planejado) e o *fluxo do modelo local* (LLM ou SLM), com o **Baixar** sob demanda
  marcado como muito experimental (#63). O `just site` confere que o grafo da extensão tem o
  `ollamaPull()` e que o grafo do repo tem o `opencode/ptbr-writer.js`.
- Os READMEs ganham dois selos de extensão de navegador em dev local (WIP), um com o logo do Chrome
  (leva à seção Chrome) e outro com o logo do Firefox (leva a `docs/EXTENSION_FIREFOX.md`).
- `docs/EXTENSION_FIREFOX.md` registra o experimento da extensão no Firefox em 2026-10-21: a
  workstation, o build, o MCP do Firefox, as origens do Ollama, o que não foi testado e os passos
  para reproduzir.
- `.mcp.json` registra o MCP do Firefox da Mozilla (`@mozilla/firefox-devtools-mcp` 0.10.4, por
  `scripts/firefox-mcp.sh`): o Claude Code abre um Firefox e instala `extension/dist-firefox` com
  `install_extension`.
- A extensão ganha o provider **Ollama + Qwen (local)**, sem chave: `just ext-ollama` serve o
  `qwen3.6:27b` (ou outro modelo) com `OLLAMA_ORIGINS` para as origens de extensão; a permissão
  `http://127.0.0.1:11434/*` é opcional no manifesto e pedida no clique que escolhe o Ollama.
  A lista sugere o `qwen3.6:27b`, oferece outros Qwen e os modelos instalados, e **Baixar** puxa o
  escolhido pelo Ollama.
- A extensão também sai para o Firefox (128+): `just ext-build-firefox` gera `extension/dist-firefox/`,
  com `background.scripts` no lugar do service worker e o id `ptbr-writer@h0ffmann`.
- O site ganha o grafo do código TypeScript da extensão do Chrome (`grafo/extensao.html`, gerado
  pelo `code-graph` só de `extension/`, com volta ao grafo do ptbr-writer) e a pílula **Grafo da
  extensão** na capa.
- O site ganha a página `bipr.html` (Gleam, como a capa): a descrição, os passos e o rodapé da
  `/bipr` lidos de `skills/bipr/`, as capturas do PR #49 com o rodapé renderizado pelo GitHub e a
  instalação pela `main`. A capa tem a pílula `/bipr`.
- `scripts/install_current.sh` instala o plugin da ponta da `main` (ou de `PTBR_REF`), sem release:
  remove a instalação e o marketplace anteriores e instala de novo, porque a `main` guarda o número da
  última versão. Dentro do Claude Code: `! curl -fsSL .../install_current.sh | bash` e `/reload-plugins`.
- `/ptbr-writer:bipr`: abre ou atualiza o PR do branch atual com o corpo bilíngue (pt-BR, depois o
  espelho em inglês), gerado dos commits e dos trailers `Tested:`; recusa no branch padrão, com
  mudança não commitada ou com o PR já mergeado. Só roda quando você chama.
- OpenCode: `scripts/install-opencode.sh vX.Y.Z` baixa a tag e instala as skills `ptbr-writer`, `ptbr`
  e `cientifico` como arquivos em `~/.config/opencode/skills/` (ou `./.opencode/` com `--project`),
  mais o plugin `ask-levels` (`opencode/ptbr-writer.js`), que roda o mesmo binário `ptbr` do hook do
  Claude Code. `scripts/opencode-smoke.sh` roda em `just plugin-check`.
- Extensão do Chrome em `extension/` (fora do plugin): sublinha no campo o que o nível pede para mudar,
  pelo Claude ou pela DeepSeek com a chave do usuário; `just ext-build`, `just ext-test` e `just ext-e2e`.
  ⚠ Não testada com uma chave válida nas APIs reais.

## v0.2.0 — 2026-10-21

- O projeto agora se chama **ptbr-writer** (antes `ptbr-writter`): o repositório
  `h0ffmann/ptbr-writer`, o marketplace e o plugin `ptbr-writer@ptbr-writer`, a skill
  `/ptbr-writer:ptbr-writer`, `ptbr-writer.toml` e os arquivos `ptbr-writer-<UTC>-...`. Quem instalou
  a v0.1.0 instala de novo: `claude plugin install ptbr-writer --marketplace h0ffmann/ptbr-writer#v0.2.0`.
- Um pedido que começa com `ptbr` pergunta uma vez os níveis, os formatos (`txt`, `md`, `pdf`,
  `notes`), a pasta e os idiomas (pt-BR por padrão, e/ou ASD-STE100), grava `./ptbr-writer.toml` e não pergunta mais; `ptbr reconfigurar` refaz.
- `.pdf` pelo `to-pdf` em Rust com Typst do picasso (`labs/ptbr-pdf`), só com permissão para baixar.
- O site abre com o comando de instalação e um botão de copiar.

## v0.1.0 — 2026-10-20

- Plugin do Claude Code, publicado com o nome antigo: marketplace `ptbr-writter`, plugin
  `ptbr-writter` com as skills `ptbr-writter`, `ptbr` (alias) e `cientifico`.
- `ptbr-routing`: a skill fora do plugin que escolhe skill e nível, e o hook `session-check`.
