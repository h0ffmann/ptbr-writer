Leitura estimada: ~4 min (463 palavras, 2 links, 11 blocos de código)

# Runbook: refazer o deploy do site do ptbr-writer

## Escopo

- Este runbook vale quando o workflow `site` falha no branch `main`.
- O workflow `site` está em `.github/workflows/site.yml`.
- O workflow `site` tem dois jobs: `build` e `deploy`.
- O job `build` roda `just quality` e `just site`.
- O job `deploy` publica `site/dist` no GitHub Pages.
- O GitHub Pages não tem processo para reiniciar. "Reiniciar o site" significa publicar de novo um build bom.
- O site fica em <https://h0ffmann.github.io/ptbr-writer/>.

## Pré-requisitos

- O operador tem `gh` autenticado no repositório `h0ffmann/ptbr-writter`.
- O operador tem `nix` e `just` instalados.
- O operador está na raiz de um clone atualizado do repositório.

## Passo 1: achar a execução que falhou

1. Liste as últimas execuções do workflow no `main`:

   ```bash
   gh run list --workflow site.yml --branch main --limit 5
   ```

2. Anote o ID da execução mais recente com status `failure`. Este runbook chama esse ID de `<RUN_ID>`.
3. Confirme que nenhuma execução do workflow `site` está com status `in_progress`. O workflow não cancela uma execução em andamento. Uma nova execução espera na fila.

## Passo 2: ler o log do workflow

1. Mostre os jobs da execução:

   ```bash
   gh run view <RUN_ID>
   ```

2. Mostre só o log dos passos que falharam:

   ```bash
   gh run view <RUN_ID> --log-failed
   ```

3. Classifique a falha pela tabela:

   | Job e passo que falhou | Causa provável | Próximo passo |
   |---|---|---|
   | `build`, passo `just quality` | Erro no conteúdo, no justfile ou nos pins | Passo 3 |
   | `build`, passo `just site` | Erro no site (conteúdo, testes Gleam, prerender) | Passo 3 |
   | `build`, passo `install-nix-action` ou `checkout` | Falha de infraestrutura do GitHub Actions | Passo 4 |
   | `deploy` | Falha na publicação do GitHub Pages | Passo 4 |

4. O passo `release due` só emite um aviso. O passo `release due` nunca causa a falha.

## Passo 3: reproduzir o build localmente

1. Atualize o clone local:

   ```bash
   git switch main && git pull
   ```

2. Rode o gate de qualidade:

   ```bash
   nix develop --no-update-lock-file --command just quality
   ```

3. Rode o build do site:

   ```bash
   just site
   ```

4. Se `just quality` ou `just site` falhar, corrija o erro num branch e abra um PR. O workflow `site` roda no PR. O merge no `main` dispara o deploy. Pare este runbook aqui.
5. Se `just quality` e `just site` passarem localmente, a falha do workflow é de infraestrutura. Siga para o Passo 4.

## Passo 4: refazer o deploy

1. Rode de novo só os jobs que falharam:

   ```bash
   gh run rerun <RUN_ID> --failed
   ```

2. Se o `rerun` falhar de novo, dispare uma execução nova no `main`:

   ```bash
   gh workflow run site.yml --ref main
   ```

3. Acompanhe a execução até o fim:

   ```bash
   gh run watch
   ```

## Passo 5: verificar o site

1. Confirme que a última execução do workflow `site` no `main` tem status `success`:

   ```bash
   gh run list --workflow site.yml --branch main --limit 1
   ```

2. Confirme que o site responde:

   ```bash
   curl -sfI https://h0ffmann.github.io/ptbr-writer/
   ```

3. O `curl` deve mostrar `HTTP/2 200`.
4. Abra o site no navegador. O rodapé deve mostrar o commit do `main` que você publicou.

## Escalonamento

- Se o job `deploy` falhar duas vezes seguidas, confira em Settings → Pages que a fonte é `GitHub Actions`.
- Se a fonte já for `GitHub Actions`, consulte <https://www.githubstatus.com/> antes de tentar de novo.
