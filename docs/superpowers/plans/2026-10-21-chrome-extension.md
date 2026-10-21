# Extensão do Chrome do ptbr-writer: plano de implementação

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** uma extensão Manifest V3 que, numa pausa de digitação, manda o texto do campo para o
Claude ou para a DeepSeek com as regras do `SKILL.md`. As sugestões voltam por trecho, como no
Grammarly.

**Architecture:**
- O content script lê o campo, espera 1 s sem digitação e pede a verificação ao service worker.
- O service worker guarda a chave, monta o prompt a partir do `SKILL.md` embutido no build, chama
  o provider e devolve a lista de sugestões.
- O content script desenha sublinhados numa camada por cima do campo, sem tocar no DOM do editor.
  Aceitar uma sugestão troca o trecho por um caminho que o editor reconhece como digitação.

**Tech Stack:**
- TypeScript, esbuild e Vitest com happy-dom.
- Playwright com o Chromium dele.
- Node do `flake.nix`.
- npm, com o `package-lock.json`.

**Spec:** `docs/superpowers/specs/2026-10-21-chrome-extension-design.md`

## Global Constraints

- **Repo:**
  - Python é proibido.
  - Nenhum segredo no código; os testes usam `sk-test`.
  - Nenhum teste chama uma API de verdade.
  - Todo commit termina com `Tested: <o que rodou>` e `Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>`.
- **Dependências:**
  - Ficam só em `extension/package.json`, com versões exatas (sem `^`), e entram pelo
    `npm ci`.
  - Só como devDependencies: `typescript`, `esbuild`, `vitest`, `happy-dom`,
    `@playwright/test`, `@types/chrome`.
- **Regras e níveis:** o texto vem do `SKILL.md` e de `skills/cientifico/SKILL.md` lidos no
  build. Nenhuma regra é digitada em `extension/src`.
- **Debounce:** 1000 ms.
- **Mínimo:** 3 palavras.
- **Limite:** 4000 caracteres. Acima disso, vai só o parágrafo do cursor.
- **Fora do texto enviado:** `blockquote` e `.gmail_quote`.
- **Endpoints:**
  - Claude: `POST https://api.anthropic.com/v1/messages`, com `x-api-key`,
    `anthropic-version: 2023-06-01` e `anthropic-dangerous-direct-browser-access: true`.
  - DeepSeek: `POST https://api.deepseek.com/chat/completions` com `Authorization: Bearer`.
- **Modelos padrão:** `claude-haiku-5-5` e `deepseek-chat`. O segundo tem ⚠ e é verificado na
  Task 4.
- **Níveis da extensão:** `estrito`, `descritivo`, `leve` e `científico`.
- **Sites padrão, ligados:**
  - `web.whatsapp.com`: `leve`;
  - `mail.google.com`: `descritivo`;
  - `github.com`: `descritivo`.
  - Os outros sites ficam desligados até o usuário liberar; ao liberar, o nível é `descritivo`.
- **Manifesto:**
  - `permissions: ["storage", "scripting"]`;
  - `host_permissions` só para os dois domínios das APIs;
  - `optional_host_permissions: ["<all_urls>"]`;
  - content scripts declarados só para os três sites padrão.
- **Versão:** a do `manifest.json` gerado é a de `plugins/ptbr-writer/.claude-plugin/plugin.json`.
  O build escreve essa versão e um teste confere. ← Divergência do spec: a versão é gerada, não
  conferida pelo `release-check`, porque assim não há duas fontes.
- **Saída do build:** `extension/dist/` (produção) e `extension/dist-test/` (testes), as duas no
  `.gitignore`.

## Review Focus

1. **O modelo responde o JSON dentro de cercas ```` ```json ```` ou com uma frase antes.** O
   esperado é que as sugestões sejam lidas mesmo assim. O teste fica na Task 3.
2. **O usuário volta a digitar enquanto a chamada está no ar.** O esperado é que a resposta antiga
   seja descartada e nada sublinhe texto que mudou. O teste fica na Task 6.
3. **O modelo devolve dois trechos sobrepostos, ou um `original` que não existe no texto.** O
   esperado é manter o primeiro e descartar os outros, sem erro. O teste fica na Task 3.
4. **A chave falta ou é inválida (401), há limite (429) ou a rede cai.** O esperado é que o selo
   mostre o estado, sem exceção nem alerta. O teste fica na Task 5.
5. **O texto tem emoji ou acentos combinados, como no WhatsApp.** O esperado é que as posições
   continuem certas (UTF-16 e NFC). O teste fica nas Tasks 3 e 6.

---

## Estrutura de arquivos

```
extension/
  package.json, package-lock.json, tsconfig.json, vitest.config.ts, playwright.config.ts
  build.mjs                 # esbuild; gera dist/ ou dist-test/ (--test) e o manifest
  static/options.html, static/popup.html, static/content.css
  src/types.ts              # Level, Suggestion, Settings, ProviderId, CheckRequest, CheckResult
  src/rules.ts              # buildSystemPrompt
  src/suggestions.ts        # parseSuggestions, locate, dropOverlaps, applyToString
  src/settings.ts           # DEFAULT_SETTINGS, levelFor, loadSettings, saveSettings
  src/providers.ts          # buildRequest, readText, classifyError
  src/background.ts         # service worker: check, abort, registrar sites
  src/content/text.ts       # textModel, selectForSending, rangeFor
  src/content/watcher.ts    # campos, debounce, ciclo de pedido
  src/content/overlay.ts    # sublinhados, cartão, selo
  src/content/apply.ts      # aceitar uma sugestão no campo
  src/content/main.ts       # entrada do content script
  src/options.ts, src/popup.ts
  test/*.test.ts            # Vitest
  e2e/fake-server.ts, e2e/pages/*.html, e2e/extension.spec.ts
```

---

### Task 1: Esqueleto, build e receitas

**Files:**
- Create:
  - `extension/package.json`
  - `extension/tsconfig.json`
  - `extension/vitest.config.ts`
  - `extension/build.mjs`
  - `extension/src/types.ts`
  - `extension/src/skills.ts`
  - `extension/static/*.html`
  - `extension/test/build.test.ts`
- Modify:
  - `.gitignore`: `extension/node_modules/`, `extension/dist/`, `extension/dist-test/`;
  - `justfile`: receitas `ext-build`, `ext-test` e `ext-e2e`; `quality` depende de `ext-test`.

**Interfaces:**
- Produces, em `src/types.ts`:
  ```ts
  export type Level = 'estrito' | 'descritivo' | 'leve' | 'científico';
  export type ProviderId = 'claude' | 'deepseek';
  export type Severity = 'erro' | 'aviso';
  export interface Suggestion { original: string; replacement: string; rule: string; severity: Severity; occurrence: number } // occurrence 1-based
  export interface Located extends Suggestion { start: number; end: number } // UTF-16, no texto enviado
  export interface SiteConfig { enabled: boolean; level: Level }
  export interface Settings {
    enabled: boolean; provider: ProviderId;
    keys: Record<ProviderId, string>; models: Record<ProviderId, string>;
    sites: Record<string, SiteConfig>;            // chave: hostname
    apiBase?: Partial<Record<ProviderId, string>>; // honrado só quando __TEST__
  }
  export type CheckError = 'no-key' | 'auth' | 'rate-limit' | 'provider' | 'network';
  export interface CheckRequest { type: 'check'; key: string; text: string; level: Level }
  export type CheckResult = { ok: true; suggestions: Suggestion[] } | { ok: false; error: CheckError };
  ```
- Produces, em `src/skills.ts`:
  - `export const SKILL: string` e `export const CIENTIFICO: string`;
  - os dois são importados de `../../SKILL.md` e `../../skills/cientifico/SKILL.md` pelo loader
    `text` do esbuild.
- Produces, no build: `node build.mjs [--test]` escreve `dist/` (ou `dist-test/`) com:
  - `background.js`, `content.js`, `options.js`, `popup.js`;
  - os HTML/CSS de `static/`;
  - `manifest.json`.
  - Com `--test`:
    - `define: { __TEST__: 'true' }`;
    - `http://127.0.0.1/*` e `http://localhost/*` entram em `host_permissions` e nos `matches`
      do content script.
- **Receitas:**
  - `ext-build`: `cd extension && npm ci && node build.mjs`;
  - `ext-test`: `ext-build`, depois `npx vitest run`;
  - `ext-e2e`: `npx playwright install chromium`, depois `node build.mjs --test`, depois
    `npx playwright test`.

- [ ] **Step 1:** Crie o `package.json` (com `"type": "module"` e versões exatas), rode `npm install` e commite o lock.
- [ ] **Step 2: Escreva o teste que falha.** Em `test/build.test.ts`, `it('manifest segue a versão do plugin')`:
  - roda `node build.mjs` com `execFileSync`;
  - lê `dist/manifest.json` e espera:
    - `version` igual ao `version` de `../plugins/ptbr-writer/.claude-plugin/plugin.json`;
    - `manifest_version === 3`;
    - `permissions` igual a `['storage','scripting']`;
    - `host_permissions` igual a `['https://api.anthropic.com/*','https://api.deepseek.com/*']`;
    - `optional_host_permissions` igual a `['<all_urls>']`;
    - `content_scripts[0].matches` igual a `['https://web.whatsapp.com/*','https://mail.google.com/*','https://github.com/*']`.
- [ ] **Step 3:** Rode `cd extension && npx vitest run test/build.test.ts`. Esperado: FAIL, porque o `build.mjs` não existe.
- [ ] **Step 4:** Implemente o `build.mjs`, o `types.ts`, o `skills.ts`, os HTML mínimos e entradas vazias para cada bundle.
- [ ] **Step 5:** Acrescente ao mesmo teste o caso `--test`: os `matches` incluem `http://127.0.0.1/*`. Rode `just ext-test`. Esperado: PASS.
- [ ] **Step 6:** Faça o commit `feat(extension): esqueleto, build e receitas`.

### Task 2: Prompt de sistema a partir do `SKILL.md`

**Files:**
- Create: `extension/src/rules.ts` e `extension/test/rules.test.ts`.

**Interfaces:**
- Consumes: `Level` e o texto de `SKILL` e de `CIENTIFICO`, passados como argumentos.
- Produces: `buildSystemPrompt(level: Level, skill: string, cientifico: string): string`.
  - O retorno contém só as seções `## Níveis de rigor` e `## Regras (em desenvolvimento)` do
    `skill`, cortadas por cabeçalho `## `.
  - Para `científico`, acrescenta o corpo de `cientifico` sem o front matter.
  - Depois vem o nível e o contrato de saída: só um array JSON de objetos
    `{original, replacement, rule, severity, occurrence}`.
    - `original` é uma cópia exata do texto.
    - `severity` é `erro` ou `aviso`.
    - `occurrence` diz qual ocorrência do trecho, contando a partir de 1.
    - Texto sem problemas responde `[]`.
    - O modelo não reescreve fora dos trechos.

- [ ] **Step 1: Testes que falham.** Os testes leem os arquivos reais com `fs`.
  - `it('inclui PTW-1 a PTW-13')`: espera cada `### PTW-n` no retorno.
  - `it('exclui Uso, Smart note e Formato de saída')`: espera que `## Smart note` e
    `## Formato de saída` não apareçam.
  - `it('científico embute CIE')`: espera que o retorno de `científico` contenha um título `CIE`
    de `skills/cientifico/SKILL.md` e que o de `leve` não contenha.
  - `it('declara o nível')`: espera que o retorno contenha `` `leve` `` como o nível pedido.
  - `it('falha se a seção some')`: chama com um `skill` sem `## Regras` e espera um throw com a
    mensagem `SKILL.md: seção Regras não encontrada`, para o build quebrar e não gerar um prompt
    vazio.
- [ ] **Step 2:** Rode `npx vitest run test/rules.test.ts`. Esperado: FAIL.
- [ ] **Step 3:** Implemente `buildSystemPrompt`.
- [ ] **Step 4:** Rode os testes. Esperado: PASS.
- [ ] **Step 5:** Faça o commit `feat(extension): prompt de sistema a partir do SKILL.md`.

### Task 3: Leitura e posicionamento das sugestões

**Files:**
- Create: `extension/src/suggestions.ts` e `extension/test/suggestions.test.ts`.

**Interfaces:**
- Consumes: `Suggestion` e `Located`.
- Produces:
  - `parseSuggestions(raw: string): Suggestion[]`
    - Tira cercas ```` ``` ````. Pega do primeiro `[` ao último `]`.
    - Descarta itens sem os campos ou com `severity` inválida.
    - `occurrence` ausente vira 1.
    - Se o JSON for inválido, devolve `[]`.
  - `locate(text: string, s: Suggestion[]): Located[]`
    - Compara em NFC, mas devolve as posições do texto original.
    - Encontra a ocorrência `occurrence` de cada trecho.
    - Descarta o que não encontra e o que tem `original === replacement`.
    - Ordena por `start`.
  - `dropOverlaps(l: Located[]): Located[]`: mantém o primeiro de cada sobreposição.
  - `applyToString(text: string, l: Located): string`

- [ ] **Step 1: Testes que falham.** Use os valores abaixo.
  - **Cercas:** `parseSuggestions('```json\n[{"original":"Foi decidido","replacement":"A equipe decidiu","rule":"PTW-1","severity":"erro"}]\n```')` devolve um item com `occurrence: 1`.
  - **Prosa:** `parseSuggestions('Aqui está: [] ok')` devolve `[]`.
  - **Inválido:** `parseSuggestions('não é json')` devolve `[]`.
  - **Severity desconhecida:** um item com `severity: "info"` é descartado.
  - **Ocorrência:** com o texto `'a nível de x e a nível de y'` e `occurrence: 2`, a sugestão
    fica em `start: 15`.
  - **Ausente:** um `original` que não está no texto é descartado.
  - **Sobreposição:** `[0,12)` e `[4,20)` viram só `[0,12)`.
  - **Emoji:** com `'Oi 👋 foi decidido'`, `locate` de `'foi decidido'` dá `start: 6`, em
    UTF-16.
  - **NFC:** um texto com `'é'` casa com o `original` `'é'`, e o `start`/`end` valem no
    texto original.
  - **Aplicar:** `applyToString('Foi decidido hoje', {start:0,end:12,replacement:'A equipe decidiu',...})` dá `'A equipe decidiu hoje'`.
- [ ] **Step 2:** Rode os testes. Esperado: FAIL.
- [ ] **Step 3:** Implemente as funções.
- [ ] **Step 4:** Rode os testes. Esperado: PASS.
- [ ] **Step 5:** Faça o commit `feat(extension): ler e posicionar sugestões`.

### Task 4: Providers e configurações

**Files:**
- Create:
  - `extension/src/providers.ts` e `extension/src/settings.ts`;
  - `extension/test/providers.test.ts` e `extension/test/settings.test.ts`.

**Interfaces:**
- Produces, em `providers.ts`:
  - `buildRequest(p: ProviderId, s: Settings, system: string, user: string): { url: string; init: RequestInit }`
    - **Claude:** body `{model, max_tokens: 1024, temperature: 0, system, messages:[{role:'user',content:user}]}`.
    - **DeepSeek:** body `{model, temperature: 0, messages:[{role:'system',content:system},{role:'user',content:user}]}`.
    - **URL base:** `s.apiBase[p]` só quando `__TEST__`; senão, os endpoints de Global
      Constraints.
  - `readText(p: ProviderId, json: unknown): string`
    - Claude: o `text` concatenado dos blocos `content` do tipo `text`.
    - DeepSeek: `choices[0].message.content`.
    - Formato inesperado: `''`.
  - `classifyError(status: number | null): CheckError`
    - `null` → `network`;
    - 401 e 403 → `auth`;
    - 429 → `rate-limit`;
    - qualquer outro → `provider`.
- Produces, em `settings.ts`:
  - `DEFAULT_SETTINGS: Settings`, com os valores de Global Constraints, `enabled: true`,
    provider `claude` e chaves vazias;
  - `levelFor(host: string, s: Settings): Level | null`: `null` quando a extensão está
    desligada, o host não está em `sites` ou o site está desligado;
  - `loadSettings(): Promise<Settings>`: mescla o `chrome.storage.local` sobre o padrão;
  - `saveSettings(s: Settings): Promise<void>`.

- [ ] **Step 0:** Consulte https://api-docs.deepseek.com e confirme o ID atual do modelo de chat.
  - Se for diferente de `deepseek-chat`, use o novo nos testes e no padrão.
  - Anote a URL e a data no comentário ao lado de `DEFAULT_SETTINGS.models.deepseek`.
- [ ] **Step 1: Testes que falham.**
  - **Claude:** os cabeçalhos são exatamente os três de Global Constraints mais
    `content-type: application/json`, e `body.system` é igual ao `system`.
  - **DeepSeek:** `Authorization` é `Bearer sk-test`, e `messages[0].role` é `system`.
  - **readText:** lê a resposta de exemplo de cada provider e devolve `''` para `{}`.
  - **classifyError:** `401` dá `auth`, `429` dá `rate-limit`, `500` dá `provider` e `null` dá
    `network`.
  - **levelFor, sites padrão:** `web.whatsapp.com` dá `leve`, `github.com` dá `descritivo` e
    `bank.example` dá `null`.
  - **levelFor, desligada:** com `enabled:false`, `github.com` dá `null`.
  - **levelFor, site liberado:** depois de adicionar `{enabled:true, level:'descritivo'}`, o site
    devolve `descritivo`.
  - **loadSettings:** usa um stub de `chrome.storage.local` e mantém as chaves padrão ausentes no
    storage.
- [ ] **Step 2:** Rode os testes. Esperado: FAIL.
- [ ] **Step 3:** Implemente as funções.
- [ ] **Step 4:** Rode os testes. Esperado: PASS.
- [ ] **Step 5:** Faça o commit `feat(extension): providers Claude e DeepSeek e configurações`.

### Task 5: Service worker

**Files:**
- Create: `extension/src/background.ts` e `extension/test/background.test.ts`.

**Interfaces:**
- Consumes: as Tasks 2 a 4 e `SKILL` e `CIENTIFICO`.
- Produces:
  - `handleCheck(req: CheckRequest, deps: { fetch: typeof fetch; settings: Settings }): Promise<CheckResult>`
    - Chave vazia: `{ok:false,error:'no-key'}`, sem chamada.
    - Para o mesmo `req.key`, aborta a chamada anterior com o `AbortController` guardado num
      `Map`.
    - Uma chamada abortada devolve `{ok:true, suggestions:[]}`.
    - Uma exceção de rede devolve `network`.
    - Status diferente de 2xx devolve `classifyError(status)`.
    - Sucesso: `{ok:true, suggestions: parseSuggestions(readText(...))}`.
  - O listener `chrome.runtime.onMessage` recebe `CheckRequest` e responde `CheckResult`.
  - `{type:'enable-site', host}` e `{type:'disable-site', host}`: registram ou removem o content
    script daquele host com `chrome.scripting.registerContentScripts` e
    `unregisterContentScripts`. O id é `site:<host>`, e o `matches` é `https://<host>/*`.
  - Na partida, o service worker registra de novo os hosts liberados que não são padrão.
- A chave nunca sai em log: nenhum `console.*` recebe `Settings` nem um objeto de headers.

- [ ] **Step 1: Testes que falham.** Use `fetch` falso.
  - `no-key`: o `fetch` não é chamado.
  - 200 com a resposta de exemplo do Claude: devolve 1 sugestão.
  - 401: devolve `auth`; 429: devolve `rate-limit`.
  - `fetch` que rejeita: devolve `network`.
  - **Abort:** duas chamadas seguidas com o mesmo `key`. O `signal` da primeira fica `aborted`,
    e a primeira devolve `suggestions: []`.
  - **Chave fora do log:** um spy em `console` não recebe nenhum argumento que contenha
    `sk-test`.
- [ ] **Step 2:** Rode os testes. Esperado: FAIL.
- [ ] **Step 3:** Implemente `handleCheck` e os listeners.
- [ ] **Step 4:** Rode os testes. Esperado: PASS.
- [ ] **Step 5:** Faça o commit `feat(extension): service worker chama o provider`.

### Task 6: Texto do campo e ciclo de pedido

**Files:**
- Create:
  - `extension/src/content/text.ts` e `extension/src/content/watcher.ts`;
  - `extension/test/text.test.ts` e `extension/test/watcher.test.ts`.
- Nos testes, use o ambiente `happy-dom`.

**Interfaces:**
- Produces, em `text.ts`:
  - `interface TextModel { text: string; nodes: { node: Text; start: number }[] }`
  - `textModel(root: HTMLElement): TextModel`
    - Concatena os nós de texto com um `TreeWalker`.
    - Pula as subárvores de `blockquote` e `.gmail_quote`.
    - Para um `textarea` ou um `input`, devolve `{text: el.value, nodes: []}`.
  - `selectForSending(text: string, caret: number): { text: string; offset: number }`
    - Se `text.length <= 4000`, devolve o texto inteiro com offset 0.
    - Senão, devolve o parágrafo delimitado por `\n` que contém o `caret`, cortado em 4000.
  - `rangeFor(m: TextModel, start: number, end: number): Range | null`
  - `wordCount(text: string): number`
- Produces, em `watcher.ts`:
  - `createWatcher(opts: { level: Level; send: (r: CheckRequest) => Promise<CheckResult>; onResult: (el: HTMLElement, located: Located[], offset: number) => void; onState: (el: HTMLElement, s: 'idle'|'checking'|CheckError) => void; delayMs?: number })`
  - Retorno: `{ attach(el: HTMLElement): void; detach(el: HTMLElement): void }`.
    - Ignora `input[type=password]` e textos com `wordCount < 3`.
    - Cada `input` limpa as sugestões, chamando `onResult(el, [], 0)`, e reinicia o timer de
      `delayMs ?? 1000`.
    - No disparo, envia o texto escolhido por `selectForSending`.
    - A resposta só é aplicada se o texto atual do campo for igual ao texto enviado; senão, é
      descartada.
    - `key` é um id estável por campo, gerado por um `WeakMap`.

- [ ] **Step 1: Testes que falham.** Use os timers falsos do Vitest.
  - **textModel:** num `div[contenteditable]` com `<p>Oi</p><div class="gmail_quote">antigo</div>`,
    `text` é igual a `'Oi'`.
  - **selectForSending:** com 3 parágrafos e 5000 caracteres, e o `caret` no segundo parágrafo,
    devolve só o segundo, com `offset` igual ao início dele.
  - **rangeFor:** com emoji antes do trecho, `range.toString()` é igual ao trecho.
  - **Debounce:** 3 eventos `input` em 500 ms geram 1 `send`, que acontece 1000 ms depois do
    último.
  - **Campos ignorados:** senha e texto com 2 palavras não geram `send`.
  - **Resposta velha:** `send` resolve depois que o valor do campo mudou, e `onResult` não recebe
    sugestões.
  - **Estado:** `send` devolve `{ok:false,error:'auth'}`, e `onState` recebe `'auth'`.
- [ ] **Step 2:** Rode os testes. Esperado: FAIL.
- [ ] **Step 3:** Implemente as funções.
- [ ] **Step 4:** Rode os testes. Esperado: PASS.
- [ ] **Step 5:** Faça o commit `feat(extension): texto do campo e debounce`.

### Task 7: Sublinhados, cartão, selo e aceitar

**Files:**
- Create:
  - `extension/src/content/overlay.ts`, `extension/src/content/apply.ts` e
    `extension/src/content/main.ts`;
  - `extension/static/content.css`;
  - `extension/test/apply.test.ts` e `extension/test/overlay.test.ts`.

**Interfaces:**
- Consumes: `createWatcher`, `textModel`, `rangeFor` e `levelFor`.
- Produces, em `apply.ts`:
  - `acceptSuggestion(el: HTMLElement, l: Located, offset: number): boolean`
    - **textarea e input:** `setRangeText(replacement, start+offset, end+offset)` e depois
      `dispatchEvent(new InputEvent('input', {bubbles:true}))`.
    - **contenteditable:** seleciona o `rangeFor` e chama
      `document.execCommand('insertText', false, replacement)`.
    - Antes de trocar, confere que o trecho atual ainda é `original`; se não for, devolve `false`
      e não muda nada.
- Produces, em `overlay.ts`:
  - `renderSuggestions(el: HTMLElement, located: Located[], offset: number, onAccept: (l: Located) => void, onIgnore: (l: Located) => void): void`
    - Desenha numa camada própria, fora do editor, a partir de `Range.getClientRects()`.
      - No `contenteditable`, usa o range real.
      - No `textarea`, usa um div espelho com os estilos computados.
    - A classe é `ptbr-underline-erro` ou `ptbr-underline-aviso`.
    - O clique abre `.ptbr-card` com:
      - a regra;
      - o `original` em `<s>`;
      - o `replacement` em `<strong>`;
      - os botões `Aceitar` (`data-action="accept"`) e `Ignorar` (`data-action="ignore"`).
  - `renderBadge(el: HTMLElement, state: 'idle'|'checking'|CheckError, count: number): void`
    - Mostra o número de sugestões pendentes, ou um destes rótulos:
      - `no-key`: `sem chave`;
      - `auth`: `chave inválida`;
      - `rate-limit`: `limite atingido`;
      - `provider` e `network`: `erro do provider`.
    - O selo tem a classe `.ptbr-badge`.
  - Todo o DOM da extensão fica num host com shadow root (`#ptbr-writer-root`), para o CSS do
    site não interferir.
- `main.ts`:
  - Lê as configurações e chama `levelFor(location.hostname)`. Se o retorno for `null`, sai sem
    anexar nada.
  - Senão, anexa o watcher aos campos editáveis focados (`focusin`).
  - Repete as posições em `scroll` e `resize`.

- [ ] **Step 1: Testes que falham.**
  - **textarea:** em `'Foi decidido hoje'`, aceitar `[0,12)` deixa o `value` igual a
    `'A equipe decidiu hoje'` e dispara 1 evento `input`.
  - **Trecho mudou:** com o `value` trocado antes, `acceptSuggestion` devolve `false` e o `value`
    fica igual.
  - **contenteditable:** com um stub de `execCommand`, verifica que ele recebe `insertText` com o
    `replacement` e que a seleção cobria o `original`.
  - **Cartão:** `renderSuggestions` com 1 erro e 1 aviso cria 1 `.ptbr-underline-erro` e 1
    `.ptbr-underline-aviso` no shadow root, e nada dentro de `el`.
  - **Aceitar:** clicar no sublinhado e depois em `[data-action=accept]` chama `onAccept` com o
    item.
  - **Selo:** `renderBadge(el,'rate-limit',0)` mostra `limite atingido`.
- [ ] **Step 2:** Rode os testes. Esperado: FAIL.
- [ ] **Step 3:** Implemente as funções.
- [ ] **Step 4:** Rode os testes. Esperado: PASS.
- [ ] **Step 5:** Faça o commit `feat(extension): sublinhados, cartão e aceitar`.

### Task 8: Opções e popup

**Files:**
- Create:
  - `extension/src/options.ts` e `extension/src/popup.ts`;
  - `extension/test/options.test.ts`.
- Modify: `extension/static/options.html` e `extension/static/popup.html`.

**Interfaces:**
- Consumes: `loadSettings`, `saveSettings`, `DEFAULT_SETTINGS` e as mensagens
  `enable-site`/`disable-site` da Task 5.
- Produces, nas **opções**:
  - os campos `#provider` (select), `#key-claude`, `#key-deepseek`, `#model-claude` e
    `#model-deepseek`;
  - uma tabela de sites com o nível (select) e liga/desliga por linha;
  - `#add-site` e `#enabled`.
  - Ao adicionar um site, as opções chamam `chrome.permissions.request({origins:['https://<host>/*']})`.
    Só se a resposta for `true`, gravam `{enabled:true, level:'descritivo'}` e enviam
    `enable-site`.
  - Remover um site chama `chrome.permissions.remove` e envia `disable-site`.
  - Uma chave salva aparece como `••••` mais os 4 últimos caracteres.
- Produces, no **popup**:
  - o host da aba ativa, o estado e o nível;
  - o botão `#toggle-site` com o texto `Ativar neste site` ou `Desativar neste site`, que segue o
    mesmo fluxo de permissão.

- [ ] **Step 1: Testes que falham.** Use um stub de `chrome`.
  - **Permissão negada:** `permissions.request` devolve `false`, o site não é gravado e nenhuma
    mensagem é enviada.
  - **Permissão aceita:** o site é gravado com `descritivo`, e `enable-site` é enviada com o host.
  - **Chave mascarada:** depois de salvar `sk-test-1234`, o campo mostra `••••1234`.
- [ ] **Step 2:** Rode os testes. Esperado: FAIL.
- [ ] **Step 3:** Implemente as opções e o popup.
- [ ] **Step 4:** Rode os testes. Esperado: PASS.
- [ ] **Step 5:** Faça o commit `feat(extension): opções e popup por site`.

### Task 9: Teste de ponta a ponta no Chromium e job de CI

**Files:**
- Create:
  - `extension/playwright.config.ts`;
  - `extension/e2e/fake-server.ts`;
  - `extension/e2e/pages/textarea.html` e `extension/e2e/pages/editable.html`;
  - `extension/e2e/extension.spec.ts`;
  - `.github/workflows/extension.yml`.

**Interfaces:**
- Consumes: o `dist-test/` da Task 1 e `Settings.apiBase`.
- **`fake-server.ts`:** `startFakeServer(): Promise<{ port: number; calls: { path: string; host: string }[]; close(): Promise<void> }>`.
  - Serve `e2e/pages/*`.
  - `POST /v1/messages` responde no formato da Anthropic, e `POST /chat/completions` responde no
    formato da OpenAI.
  - As duas rotas devolvem uma sugestão fixa:
    `{"original":"Foi decidido","replacement":"A equipe decidiu","rule":"PTW-1","severity":"erro"}`.
  - Cada chamada registra o cabeçalho `host` em `calls`.
- **Spec:**
  - Usa `chromium.launchPersistentContext` com `channel: 'chromium'` e headless, com
    `--disable-extensions-except=<dist-test>` e `--load-extension=<dist-test>`.
  - Pega o service worker e grava as configurações direto no `chrome.storage.local`:
    - `apiBase` aponta para o servidor falso;
    - as chaves são `sk-test`;
    - `sites`: `{'127.0.0.1': {enabled:true, level:'descritivo'}}`.

- [ ] **Step 1: Escreva os testes.**
  - `for (const provider of ['claude','deepseek'])`, `it(\`${provider}: textarea, sublinha, aceita\`)`:
    - abre `http://127.0.0.1:<port>/textarea.html`;
    - digita `Foi decidido que o deploy vai ser na sexta.`;
    - espera `.ptbr-underline-erro` no shadow root, com timeout de 5 s;
    - clica no sublinhado e em `Aceitar`;
    - espera o valor `A equipe decidiu que o deploy vai ser na sexta.`.
  - `it('contenteditable estilo Gmail')`:
    - repete o fluxo em `editable.html`, que tem um `.gmail_quote` com `Foi decidido`;
    - confere que só o trecho fora da citação é sublinhado e trocado.
  - `it('site não liberado não chama a API')`:
    - abre `http://localhost:<port>/textarea.html`;
    - digita e espera 2 s;
    - espera que não haja `.ptbr-underline-erro` e que `calls.filter(c => c.host.startsWith('localhost'))` esteja vazio.
- [ ] **Step 2:** Rode `just ext-e2e`. Esperado: tudo PASS. Se algum caso falhar por defeito das
  Tasks 5 a 8, corrija na task de origem e anote no commit.
- [ ] **Step 3: Escreva `.github/workflows/extension.yml`.**
  - Gatilho: `pull_request` e `push` para `main`, com `paths`: `extension/**`, `SKILL.md`,
    `skills/cientifico/SKILL.md` e `plugins/ptbr-writer/.claude-plugin/plugin.json`.
  - Roda em `ubuntu-latest`, sem nix:
    - `actions/setup-node` com a versão do Node do `nix develop`; anote a versão;
    - `npm ci`;
    - `npx playwright install --with-deps chromium`;
    - `npx vitest run`;
    - `node build.mjs --test`;
    - `npx playwright test`.
- [ ] **Step 4:** Rode `just quality`. Esperado: `quality: ok`, já com o `ext-test`.
- [ ] **Step 5:** Faça o commit `test(extension): ponta a ponta no Chromium com servidor falso`.

### Task 10: Documentação

**Files:**
- Modify:
  - `README.md`: uma seção `### Chrome` dentro de `## instalar` e um parágrafo em `## usar`;
  - `README.en.md`: o espelho das duas mudanças;
  - `AGENTS.md`: uma linha em "Repo layout" para `extension/`;
  - `CHANGELOG.md`: uma entrada em "não lançado", no formato atual do arquivo.

O conteúdo:
- **Instalar:**
  - `just ext-build`;
  - abrir `chrome://extensions` e ligar o modo desenvolvedor;
  - em "Carregar sem compactação", escolher `extension/dist/`;
  - nas opções, escolher o provider e colar a chave.
- **Usar:**
  - os três sites padrão e os níveis;
  - como liberar um site;
  - o fato de que o texto vai só para o provider escolhido;
  - o ⚠: ninguém testou com uma chave válida.
- **Não mude:** `SKILL.md`, `site/data/*.tsv` nem o roadmap. As tabelas do README conferidas
  pelo `quality` não mudam.

- [ ] **Step 1:** Escreva as seções nos dois READMEs, com os mesmos fatos.
- [ ] **Step 2:** Rode `just reading-time`, porque o selo de tempo de leitura muda com o texto.
  Depois rode `just quality`. Esperado: `quality: ok`.
- [ ] **Step 3:** Faça o commit `docs(extension): instalar e usar no Chrome`.
