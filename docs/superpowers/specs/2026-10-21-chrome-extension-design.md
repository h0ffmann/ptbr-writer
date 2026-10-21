# Extensão do Chrome do ptbr-writer: design

Data: 2026-10-21. Status: rascunho para aprovação.

## Intenção

O dono do repo escreve em português no Chrome: no WhatsApp Web, no Gmail e no GitHub. Ele quer
as regras do ptbr-writer (PTW-1..13 do `SKILL.md`) corrigindo esse texto enquanto escreve, no
estilo do Grammarly. Cada site pede um tom diferente, mas a base é sempre português correto e
profissional.

A extensão chama um modelo de linguagem com a chave do próprio usuário. Ela não tem servidor
próprio.

## Objetivos

- Ao parar de digitar num campo de texto, a extensão manda o texto para o provider escolhido e
  mostra as sugestões no próprio campo.
- As sugestões aparecem por trecho, como no Grammarly (opção A dos mockups): o trecho fica
  sublinhado e um clique abre um cartão com a regra, o antes, o depois e os botões **Aceitar** e
  **Ignorar**.
- Há dois providers no início, cada um com a chave do usuário: **Claude** (API da Anthropic) e
  **DeepSeek** (API da DeepSeek).
- O tom vem do nível de rigor do `SKILL.md`, escolhido por site.
- Testes automatizados cobrem cada parte, e um teste de ponta a ponta abre o Chromium com a
  extensão carregada.

## Não objetivos

- Publicar na Chrome Web Store agora. O uso inicial é só do dono, carregando a extensão sem
  empacotar pelo modo desenvolvedor. A publicação vem depois, por isso nada no design pode
  bloqueá-la (ver Restrições).
- Ter um servidor próprio, conta, login ou sincronização.
- Ter outros providers além dos dois, como OpenCode Go, OpenAI ou um modelo local. A camada de
  provider deve aceitar um novo depois, mas este trabalho não inclui nenhum.
- Usar o OpenCode Go. Ele foi descartado porque os termos o descrevem como feito "para o OpenCode
  e outros agentes de código", e uma extensão de prosa no navegador fica numa zona cinzenta.
- Suportar outros navegadores, como Firefox e Safari.
- Usar os binários Rust do plugin (`ptbr ask-levels`, `ptbr-route`). Eles só existem para Linux e
  não rodam no navegador.

## Antiobjetivos (falha mesmo funcionando)

- Trocar texto sem o usuário aceitar.
- A chave sair do `chrome.storage.local` para qualquer lugar que não seja a API do próprio
  provider: código, repo, log, console ou outro domínio.
- Atrapalhar a digitação: quebrar o editor do site, mover o cursor, perder texto ou travar a
  página.
- Chamar a API a cada tecla e gastar crédito à toa.
- Copiar as regras à mão para dentro da extensão, de modo que elas divirjam do `SKILL.md`.

## Restrições do repo (de `AGENTS.md`)

- Python é proibido. A extensão é TypeScript.
- Nada de segredo no código. Os testes usam chaves falsas como `sk-test`.
- Determinismo: versões travadas. O Node vem do `flake.nix` (já está no dev shell), os pacotes
  npm vêm do `package-lock.json` instalado com `npm ci`, e o Chromium dos testes é o do
  Playwright, travado pela versão do Playwright no lock. O mise não entra, porque duplicaria o
  `flake.nix`.
- Nenhum recurso pago sem confirmação. Nenhum teste chama uma API de verdade.
- Uma feature por PR, e commits com os trailers `Tested:` e `Co-Authored-By`.
- `README.md` e `README.en.md` são espelhos: a seção que explica como instalar e usar a extensão
  entra nos dois no mesmo PR, e a linha de layout entra em `AGENTS.md`.
- O texto do `SKILL.md` vai para o contexto de outros modelos. Ele não ganha créditos nem links
  por causa da extensão.

## O que foi decidido

### Disparo

- **Debounce:** espera 1 s depois da última tecla num campo editável (`textarea`,
  `input[type=text]` e `contenteditable`). ← minha decisão.
- **Campos e textos ignorados:** campos de senha, campos com menos de 3 palavras e campos em
  sites desligados. ← minha decisão.
- **Nova digitação:** se o usuário volta a digitar, as sugestões daquele campo somem, a chamada em
  andamento é cancelada (`AbortController`) e uma nova chamada acontece na próxima pausa.
- **Texto que mudou:** a resposta só é aplicada se o texto do campo ainda for igual ao texto
  enviado.
- **O que é enviado:** o texto que o usuário escreveu, sem o histórico citado. No Gmail, isso
  exclui o `blockquote` e o `.gmail_quote` de uma resposta. O limite é de 4.000 caracteres: um
  texto maior envia só o parágrafo onde está o cursor. ← minha decisão.

### Sugestão (opção A)

- Cada trecho é sublinhado no campo: vermelho para erro, amarelo para aviso.
- O clique abre um cartão com:
  - a regra (ex.: "PTW-1 · Quem faz, aparece");
  - o antes riscado;
  - o depois em verde;
  - os botões Aceitar e Ignorar.
- Aceitar troca só aquele trecho. Ignorar esconde a sugestão até a próxima chamada.
- Um selo no canto do campo mostra quantas sugestões faltam.

### Providers

- **Claude:** `POST https://api.anthropic.com/v1/messages` com os cabeçalhos:
  - `x-api-key`;
  - `anthropic-version: 2023-06-01`;
  - `anthropic-dangerous-direct-browser-access: true`.
  - O preflight de CORS libera esses cabeçalhos. Ele foi verificado em 2026-10-21, sem uma chave
    válida.
- **DeepSeek:** `POST https://api.deepseek.com/chat/completions`, no formato da OpenAI, com
  `Authorization: Bearer`. O preflight de CORS libera a origem `chrome-extension://`. Ele foi
  verificado em 2026-10-21; uma chave falsa recebeu 401, e não houve teste com uma chave válida.
- A chamada sai do service worker da extensão, que tem `host_permissions` para os dois domínios.
  Ela não sai do content script. Assim a chave nunca entra na página do site.
- Cada provider tem um modelo padrão rápido e barato que o usuário pode trocar nas opções.
  - Claude: `claude-haiku-5-5`.
  - DeepSeek: ⚠ não verificado. O builder confirma o ID atual na documentação da DeepSeek, e
    `deepseek-chat` é o candidato.

### Prompt e resposta

- **Prompt de sistema:**
  - As regras PTW-1..13 e a seção "Níveis de rigor" vêm do `SKILL.md`, embutidas no bundle na
    hora do build. Ninguém copia esse texto à mão.
  - Acrescenta o nível do site e a instrução de responder **só** com JSON.
  - O prompt não usa as seções "Uso", "Smart note" e "Formato de saída" do `SKILL.md`, que servem
    ao chat, não a uma correção inline.
- **Resposta:** uma lista de sugestões. Cada sugestão tem:
  - `original`: o trecho exato do texto;
  - `replacement`: a troca;
  - `rule`: por exemplo, `PTW-1`;
  - `severity`: `erro` ou `aviso`.
- **Validação:**
  - A extensão valida o JSON.
  - Ela localiza cada `original` no texto. Se um trecho aparece mais de uma vez, a sugestão vale
    para a ocorrência pedida, com um campo de contexto ou de índice; o formato fica a cargo do
    builder.
  - Uma sugestão cujo `original` não está no texto é descartada sem erro.

### Nível por site

| Site | Nível padrão |
|---|---|
| `web.whatsapp.com` | `leve` |
| `mail.google.com` | `descritivo` |
| `github.com` | `descritivo` |
| qualquer outro | desligado até o usuário liberar; ao liberar, `descritivo` |

Os três sites da tabela começam ligados. Os outros começam desligados, e o usuário libera cada
um. ← decisão do dono. Nenhum texto de um site não liberado sai do navegador.

O usuário troca o nível de cada site nas opções. ← A tabela é minha decisão, e o dono a aceitou na
conversa.

A lista de níveis das opções tem `estrito`, `descritivo`, `leve` e `científico`. ← minha decisão.
- O `científico` também embute `skills/cientifico/SKILL.md` no bundle, para as regras CIE
  valerem de fato.
- O `smart-note` fica fora da lista, porque transforma o texto em notas separadas e não faz
  sentido numa correção inline.

### Opções (página de configurações)

A página de opções tem:

- o provider (Claude ou DeepSeek);
- uma chave por provider;
- o modelo de cada provider;
- o nível por site;
- a lista de sites liberados, com Adicionar e Remover;
- um botão geral de liga/desliga e um por site.

O popup da extensão, aberto pelo ícone na barra, mostra o estado do site atual, com o botão
"Ativar neste site" ou "Desativar neste site", e o nível desse site.

Tudo fica no `chrome.storage.local`. A chave aparece mascarada depois de salva.

### Publicação futura

- O manifesto é Manifest V3 e pede permissões mínimas:
  - `storage`;
  - `host_permissions` só para os domínios das duas APIs.
- Os content scripts são declarados só para os três sites padrão. ← minha decisão, porque os
  outros sites começam desligados.
- Para liberar outro site, o usuário clica em "Ativar neste site" no popup da extensão ou adiciona
  o site nas opções. A extensão então:
  - pede a permissão daquele domínio com `chrome.permissions.request`, a partir de
    `optional_host_permissions: ["<all_urls>"]`;
  - registra o content script com `chrome.scripting.registerContentScripts`.
- Revogar o site remove a permissão e o registro. A extensão nunca pede acesso a todos os sites de
  uma vez.
- Nenhum código remoto e nenhuma telemetria.

### Construção e testes

- **Construção:**
  - TypeScript e esbuild.
  - Fica num diretório próprio na raiz, `extension/` ← minha decisão. Ele fica fora de
    `plugins/`, então não entra em `release-due`.
- **Testes de unidade (Vitest):**
  - montagem do prompt a partir do `SKILL.md`;
  - leitura da resposta de cada provider, inclusive com JSON inválido e erros 401 e 429;
  - casamento dos trechos com o texto;
  - aplicação de uma sugestão aceita;
  - nível por site e opções.
- **Teste de ponta a ponta (Playwright):**
  - Abre o Chromium com a extensão carregada sem empacotar.
  - Usa um servidor HTTP falso local, que responde nos formatos da Anthropic e da DeepSeek com
    sugestões fixas.
  - A extensão precisa de um jeito de apontar para outra URL base só nos testes, por exemplo uma
    opção oculta ou um build de teste. O jeito fica a cargo do builder.
  - O teste usa páginas locais que imitam um `textarea` e um `contenteditable` parecido com o do
    Gmail.
  - Fluxo: digita, espera o sublinhado, clica, aceita e confere o texto final. Roda uma vez para
    cada provider.
  - Um caso confere que um site não liberado não recebe sugestões e que o servidor falso não
    recebe nenhuma chamada dele.
- **Receitas no `justfile`:**
  - `just ext-build`, `just ext-test` e `just ext-e2e`;
  - `just quality` chama `ext-build` e `ext-test` (Vitest), mas não o `ext-e2e`. ← minha decisão.
  - O `ext-e2e` roda num job próprio do CI, em ubuntu, fora do `nix develop`, com
    `npx playwright install --with-deps chromium`. Um PR que muda `extension/` ou `SKILL.md`
    dispara esse job. ← minha decisão.
  - Localmente, `just ext-e2e` instala o Chromium do Playwright na primeira vez.
- **Versão e entrega:** ← minha decisão.
  - O `version` do `extension/manifest.json` segue a versão única do repo (`plugin.json`,
    `CHANGELOG.md` e a tag `vX.Y.Z`), e o `just release-check` confere isso.
  - O bundle não entra no git: `extension/dist/` vai para o `.gitignore`.
  - O dono roda `just ext-build` e carrega `extension/dist/` sem empacotar. Uma mudança no
    `SKILL.md` só chega à extensão num novo build.

## O que fica a cargo do builder

- A arquitetura interna: como content script, service worker e página de opções conversam.
- Como sublinhar sem quebrar editores controlados por framework, como o do WhatsApp Web e o do
  Gmail.
  - A sugestão é desenhar o sublinhado numa camada por cima, a partir de
    `Range.getClientRects()`, em vez de inserir `<span>` no editor.
  - Para aceitar, a troca usa um caminho que o editor do site reconheça como digitação, como
    `document.execCommand('insertText')` sobre uma seleção, e não uma escrita direta no DOM.
- O formato exato do JSON, desde que tenha os campos acima.
- O estilo visual, que deve ser próximo do mockup A.
- Como o `SKILL.md` entra no bundle: um plugin do esbuild ou um script de build em TypeScript.
- O tratamento de erros, que mostra no selo um estado simples, como "sem chave", "erro do
  provider" ou "limite atingido", sem alerta que interrompa a escrita.

## Referência

- Mockups: `.superpowers/brainstorm/` (local, fora do git). A opção A foi a escolhida.
