# Changelog

Uma versão só: a tag `vX.Y.Z`, o `version` de `plugins/ptbr-writer/.claude-plugin/plugin.json` e o
cabeçalho abaixo andam juntos. `just release X.Y.Z` abre o PR da versão e, depois do merge, cria a tag; `release.yml` publica a release.

## Unreleased

- O app de desktop (`app/`, Tauri 2.12): Linux, macOS e Windows, Rust na janela (wasm) e no núcleo, que
  roda a revisão da extensão (`ptbr_ext::background::handle_check`) sobre reqwest. `just app-bootstrap`,
  `app-dev`, `app-build`, `app-test` e `app-screenshot`; o `app.yml` testa, empacota e fotografa a janela
  nos três sistemas.
- O app no Android (`app/src-tauri/gen/android`): a mesma janela, só com a Claude, e uma sobreposição
  (AccessibilityService) que revisa o campo em que se digita em qualquer app, sublinha e abre as correções
  por um selo. A revisão e a sessão são do Rust, via JNI (`overlay.rs`, `android.rs`). `just
  app-android-bootstrap`, `app-android-test`, `app-android-dev`, `app-android-build` e
  `app-android-screenshot`; o job `android` do `app.yml` roda a sobreposição num emulador.

- O shell do site (`.#site`) passa ao Node 24.22.1: o picasso fixado vai de `b919f9e` a `5da5237`
  (h0ffmann/picasso#143) no `flake.lock`, no submódulo `forks/picasso`, no `tools.lock` e no `to-pdf.sh`. Os
  binários `ptbr` refeitos nesse rev têm o mesmo sha256 (#83).
- A skill `ptbr-writer` (plugin) ganha as regras PTW-14 a PTW-17, e a skill `cientifico` aplica as
  PTW-1 a PTW-17.
- A extensão do Chrome sublinha de vermelho a ortografia, a pontuação e a concordância (PTW-14 a 16) e
  de amarelo a frase reorganizada (PTW-17, cartão **Reescrita**); as configurações ganham o tom, o
  quanto corrigir, a espera e a lista "nunca corrigir". O teste ponta a ponta cobre a frase
  reorganizada.
- Node 24.22.1, a release de segurança de 2026-10-21 (#83), no dev shell e no CI. O nixpkgs ainda está na
  24.21.0, afetada, então o `nix/nodejs-bin.nix` usa o binário oficial do nodejs.org, fixado pelo sha256.
  O `just node-check`, no `just quality`, falha abaixo das versões corrigidas. O `cve.yml` roda o
  OSV-Scanner nos locks, o `node-check` e o `is-my-node-vulnerable` na `main`, em cada PR e toda segunda.
  O shell do site recebe a mesma correção pelo picasso (h0ffmann/picasso#143).
- O site embute o whitepaper em pt-BR numa seção própria (`#whitepaper`), como o site do closed-source
  embute o artigo dele: o PDF num quadro, carregado sob demanda, com links para abrir e baixar onde o
  quadro não aparece. A pílula do pt-BR sai da capa; a do inglês fica.
- A extensão de navegador é reescrita em Rust, compilado para WebAssembly (wasm-bindgen); o
  TypeScript, o esbuild e o Vitest saem. `cargo xtask build [--test] [--firefox]` gera a extensão;
  o service worker e as páginas rodam o wasm, e o content script roda o mesmo módulo em JavaScript
  (wasm2js, do binaryen), porque a CSP da página vale para compilar wasm no mundo do content script
  e o github.com a proíbe. `just ext-test` roda fmt, clippy (host e wasm32) e os 113 testes de
  unidade; `just ext-e2e` roda os 32 testes Playwright, que agora cobrem também o modelo de texto,
  o watcher, o overlay, as páginas de opções e popup, os quatro providers e uma página com CSP
  estrita. O CI da extensão roda dentro do shell do flake. O "feito com" do site passa a listar o
  Rust como linguagem do repositório, e o grafo da extensão é o do código Rust.
- A v0.3.0 tem DOI no Zenodo, como preprint. O `CITATION.cff` traz o DOI de conceito, que sempre leva
  à versão mais recente, e o da v0.3.0. Os READMEs ganham um selo do Zenodo, e a pílula do site leva
  ao DOI de conceito; nenhum dos dois mostra o número.

## v0.3.0 — 2026-10-22

- Instale com `claude plugin install ptbr-writer --marketplace h0ffmann/ptbr-writer#v0.3.0`. É a
  primeira tag desde a `v0.1.0`: a v0.2.0 subiu o `plugin.json` na `main`, mas a tag nunca foi criada,
  então o que está nesta seção e na v0.2.0 sai numa tag pela primeira vez. No plugin mudou: `.claude-plugin/marketplace.json`, `SKILL.md`, `plugins/ptbr-writer/.claude-plugin/plugin.json`, `plugins/ptbr-writer/bin/linux-aarch64/ptbr`, `plugins/ptbr-writer/bin/linux-x86_64/ptbr`, `plugins/ptbr-writer/bin/run.sh`, `plugins/ptbr-writer/bin/tools.lock`, `plugins/ptbr-writer/hooks/ask-levels.sh`, `plugins/ptbr-writer/hooks/hooks.json`, `plugins/ptbr-writer/ptbr-writer.default.toml`, `plugins/ptbr-writer/scripts/to-pdf.sh`, `skills/bipr/SKILL.md`, `skills/bipr/footer.md`, `skills/cientifico/SKILL.md`.
- O whitepaper, em pt-BR (`WHITEPAPER.pdf`) e em inglês (`WHITEPAPER.en.pdf`), 8 páginas cada: a tese
  de que escrita agêntica em pt-BR é uma abstração barata, com três pilares (português brasileiro sob
  medida, portabilidade e um harness que melhora a si mesmo). `just whitepaper` o gera com pandoc e
  XeLaTeX, no template do Paper Fabric do closed-source, com os logos da marola e da ouroboric; o
  `just quality` falha acima de 8 páginas. O site ganha duas pílulas de PDF, uma por idioma.
- A licença passa de MIT para **MPL-2.0** (`LICENSE`, os selos e a seção de licença dos READMEs, o
  `plugin.json`, o `site/gleam.toml` e o rodapé do site). A `v0.1.0` e os commits anteriores continuam
  MIT, e o que vem de terceiros mantém a licença de cada um.
- O "feito com" do site mostra o ícone do dono de cada linguagem (Just, Shell, CSS, JavaScript e
  HTML ganham o seu), tira o TypeScript por enquanto e acrescenta o Rust como planejado.
- Os dois READMEs passam de ~31 para ~8 min de leitura, e o `just quality` falha acima de 10 min. As
  tabelas de regras, roadmap, awesome e níveis planejados ficam só no site, que as gera de
  `site/data/`; o detalhe de instalação, configuração, extensão, tempo de leitura e os commits dos
  créditos vai para a seção *Reference* do `AGENTS.md`.
- O roadmap ganha a **extensão do Chrome** e o **FOSS (Ollama, Ollaya)**, os dois em andamento.
- Os READMEs ganham os selos **modelo local: Ollama** e **modelo de decisão: Ollaya**, com o ícone de
  cada um; cada selo leva ao parágrafo do provider (`#ollama`, `#ollaya`).
- A extensão ganha o provider **Ollaya (local, só aponta)** (#61): um modelo de decisão do
  [Ollaya](https://github.com/ollaya-dev/ollaya) diz, frase a frase, qual regra do nível a frase
  quebra; a regra vira um sublinhado sem troca, e o cartão só tem **Ignorar**. A fiação é a do Ollama:
  sem chave, permissão `http://127.0.0.1:11435/*` pedida no clique, lista de modelos, **Baixar** e
  `just ext-ollaya` com `OLLAYA_ORIGINS`. O modelo sugerido é o `decima`; a escolha está em
  `docs/SURVEY_DECISION_MODEL.md`. ⚠ Latência e acerto em pt-BR não medidos.
- **Ollaya + Ollama**: com o Ollama escolhido, a chave **Ollaya escolhe o estilo** faz o Ollaya
  escolher o estilo da conversa (`conciso`, `socrático`, `amigável`) antes de cada revisão, e o Ollama
  escreve as trocas nesse estilo. As duas são chaves independentes: com o Ollaya fora do ar, o Ollama
  revisa sozinho.
- O provider **Ollama + Qwen (local)** vira **Ollama (local)**: o Qwen é só um dos modelos.
- O site diz "feito com" as linguagens do repositório (TypeScript por último, infelizmente) e com
  Claude Code, OpenCode, OpenCodeReview, Ollama e Ollaya.
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
