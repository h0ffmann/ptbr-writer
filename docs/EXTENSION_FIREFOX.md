# A extensão no Firefox: o experimento de 2026-10-21

Leitura estimada: ~6 min (1019 palavras, 5 links, 3 blocos de código)

Este documento registra como a extensão do ptbr-writer foi construída, carregada e testada no
Firefox em 2026-10-21, com o provider local Ollama + Qwen. O objetivo é que outra pessoa repita
cada passo e compare o resultado. Um `⚠` marca o que ninguém checou.

> [!WARNING]
> O Ollama recusa a extensão (HTTP 403) até você rodar este comando uma vez. O comando cria um
> override do serviço `ollama` do systemd, com `OLLAMA_ORIGINS` para as origens de extensão, e
> reinicia o serviço:
>
> ```bash
> sudo mkdir -p /etc/systemd/system/ollama.service.d && printf '[Service]\nEnvironment="OLLAMA_ORIGINS=chrome-extension://*,moz-extension://*"\n' | sudo tee /etc/systemd/system/ollama.service.d/ptbr-writer.conf && sudo systemctl daemon-reload && sudo systemctl restart ollama
> ```
>
> Com o override, qualquer extensão instalada no navegador pode chamar o Ollama, não só a
> ptbr-writer. Para desfazer:
>
> ```bash
> sudo rm /etc/systemd/system/ollama.service.d/ptbr-writer.conf && sudo systemctl daemon-reload && sudo systemctl restart ollama
> ```

## A workstation

Os valores vêm dos comandos da última coluna, rodados na máquina do teste.

| Item | Valor | Comando |
| :--- | :---- | :------ |
| CPU | Intel Core i9-14900, 32 threads | `lscpu` |
| Memória | 184 GiB | `free -h` |
| GPU | NVIDIA GeForce RTX 4090, 24.564 MiB | `nvidia-smi --query-gpu=name,memory.total --format=csv` |
| Driver NVIDIA | 615.71.09 | `nvidia-smi --query-gpu=driver_version --format=csv` |
| Sistema | Ubuntu 26.04.1 LTS | `lsb_release -ds` |
| Kernel | 7.0.0-38-generic | `uname -r` |
| Firefox | Mozilla Firefox 157.0.1, snap, revisão 9036 | `snap list firefox` |
| Ollama | 0.12.11, serviço do systemd | `ollama --version` |
| Node.js | 22.22.1 | `node --version` |
| Nix | Determinate Nix 3.22.2 (Nix 2.35.2) | `nix --version` |
| just | 1.58.0 | `just --version` |

## O que foi feito

### 1. O build para o Firefox

O Firefox não roda o `background.service_worker` do manifesto MV3 do Chrome. O
`node build.mjs --firefox` gera o mesmo pacote com duas diferenças no manifesto:

- `background.scripts: ["background.js"]` no lugar do service worker;
- `browser_specific_settings.gecko`: o id `ptbr-writer@h0ffmann` e o Firefox mínimo 128.0.

O comando `just ext-build-firefox` gerou `extension/dist-firefox/` com 8 arquivos: `manifest.json`,
`background.js`, `content.js`, `content.css`, `popup.html`, `popup.js`, `options.html` e
`options.js`.

O `npx web-ext@8.3.0 lint --source-dir extension/dist-firefox` deu 0 erros, 0 notices e 1
aviso: `UNSAFE_VAR_ASSIGNMENT` (atribuição a `innerHTML`) em `options.js`. O aviso não impede a
carga da extensão.

### 2. A carga manual

O dono do repo carregou `extension/dist-firefox/manifest.json` em
`about:debugging#/runtime/this-firefox`, por **Carregar extensão temporária…**, e escolheu o
provider **Ollama + Qwen (local)** no popup. O dono relatou duas observações:

- **A lista de modelos detectou todos os modelos já baixados no Ollama.** A extensão lê
  `GET /api/tags` e junta os modelos instalados aos sugeridos. Os modelos instalados de antes do
  experimento apareceram sem `(baixar)`.
- **O botão Baixar não pareceu funcionar direito.** ⚠ A mensagem e o modelo não foram registrados.
  A issue [#63](https://github.com/h0ffmann/ptbr-writer/issues/63) lista as hipóteses: o popup fecha
  e cancela o pedido, e o `stream: false` não mostra progresso.

⚠ Os sublinhados, o cartão e o **Aceitar** não foram registrados.

### 3. O MCP do Firefox

O `.mcp.json` registra o MCP do Firefox da Mozilla
([`@mozilla/firefox-devtools-mcp`](https://github.com/mozilla/firefox-devtools-mcp) 0.10.4) pelo
`scripts/firefox-mcp.sh`. Com ele, o Claude Code abre um Firefox num perfil descartável e instala a
extensão.

Um cliente MCP por stdio rodou o servidor com `--headless`:

- `tools/list` listou 34 tools, entre elas `install_extension` e `uninstall_extension`.
- Com o `/usr/bin/firefox`, o Firefox fechou com `Process ... unexpectedly closed with status 1`.
  No Ubuntu, o `/usr/bin/firefox` é um script que chama o snap, e o geckodriver não o abre.
- Com `--firefox-path /snap/firefox/current/usr/lib/firefox/firefox`, o
  `install_extension` (type `path`, `extension/dist-firefox`) respondeu
  `Extension installed (temporary): ID: ptbr-writer@h0ffmann`.

O `scripts/firefox-mcp.sh` passa esse binário sempre que ele existe.

### 4. O Ollama e as origens

O provider **Ollama (local)** (até 2026-10-22, **Ollama + Qwen (local)**) chama `http://127.0.0.1:11434/api/chat`. A extensão pede a
permissão para `http://127.0.0.1:11434/*` no clique que escolhe o provider.

O Ollama confere o cabeçalho `Origin`. Os resultados, com `curl -H "Origin: ..."` em `/api/tags`:

| Servidor | `chrome-extension://` | `moz-extension://` | `https://evil.example` |
| :------- | :-------------------: | :----------------: | :--------------------: |
| serviço do systemd, sem `OLLAMA_ORIGINS` | 403 | 403 | ⚠ não testado |
| `ollama serve` na porta 11435, com `OLLAMA_ORIGINS` | 200 | 200 | 403 |
| serviço do systemd, depois do override do aviso | ⚠ não testado | 200 | ⚠ não testado |

O `just ext-ollama` confere a mesma coisa antes de baixar o modelo. Sem o override, a receita
parou com `the Ollama at 127.0.0.1:11434 refuses extensions (HTTP 403)` e mostrou o comando do aviso.

## O que não foi feito

- ⚠ Nenhum modelo Qwen 3 estava instalado no fim do experimento; o único Qwen era o
  `qwen2.5-coder:7b`. Nenhuma sugestão do Qwen apareceu na extensão.
- ⚠ A latência de uma revisão com o `qwen3.6:27b` não foi medida.
- ⚠ O pedido de permissão do Firefox no `change` do seletor de provider não foi testado. Se o
  Firefox não mostrar o pedido, libere `127.0.0.1:11434` em `about:addons` → ptbr-writer →
  **Permissões**.
- A suíte e2e (`just ext-e2e`) roda só no Chromium.

## Como reproduzir

1. Clone o repo e entre no shell de desenvolvimento:

   ```bash
   git clone https://github.com/h0ffmann/ptbr-writer && cd ptbr-writer && nix develop
   ```

2. Rode o comando do aviso, no topo deste documento.
3. Confira que o Ollama aceita a extensão e baixe o modelo:

   ```bash
   just ext-ollama            # qwen3.6:27b, 18 GB
   just ext-ollama qwen3:8b   # 5,2 GB, para uma GPU menor
   ```

4. Gere a extensão:

   ```bash
   just ext-build-firefox
   ```

5. No Firefox, abra `about:debugging#/runtime/this-firefox`, clique em **Carregar extensão
   temporária…** e escolha `extension/dist-firefox/manifest.json`.
6. Em `about:addons` → ptbr-writer → **Permissões**, libere os sites e `127.0.0.1:11434`.
7. No popup, escolha **Ollama (local)** e o modelo baixado.
8. Abra o WhatsApp Web, escreva 3 palavras ou mais num campo e espere 1 s.

Para o Claude Code fazer os passos 5 a 8, abra uma sessão nova no repo, aprove o servidor `firefox`
do `.mcp.json` e peça: "instale `extension/dist-firefox` com `install_extension` e abra o WhatsApp
Web". O perfil do MCP é descartável, e o WhatsApp Web pede o QR code a cada sessão.

Os tamanhos de download vêm da página de tags do Ollama em 2026-10-21 (`ollama.com/library/qwen3/tags`
e `ollama.com/library/qwen3.6/tags`). ⚠ O tamanho do `qwen3.6:27b` (18 GB) foi lido entre variantes
de 18 a 24 GB.
