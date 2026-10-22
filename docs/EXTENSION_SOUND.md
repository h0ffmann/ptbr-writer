# O som da extensão: impacto no tamanho e no tempo de execução

Leitura estimada: ~4 min (706 palavras, 0 links, 1 bloco de código)

Este documento mede quanto o som da extensão (#55, `extension/src/content/sound.ts`) acrescenta ao
tamanho dela e ao tempo de execução. Cada número tem o comando que o produziu. Um `⚠` marca o que
ninguém checou.

## Em uma frase

O som acrescenta 3 740 bytes à extensão (1 883 com gzip) e, ligado, custa entre 21,8 e 45,8 ms uma
vez, no primeiro efeito, e menos de 0,2 ms em cada efeito seguinte. Desligado, que é o padrão,
nenhum desses custos de execução acontece. Não há servidor nem arquivo de som.

## A máquina

| | Valor | Comando |
| :-- | :-- | :-- |
| Sistema | Linux 7.0.0-38-generic, x86_64 | `uname -srm` |
| CPU | Intel Core i9-14900, 32 threads | `grep -m1 'model name' /proc/cpuinfo`, `nproc` |
| Navegador | Chromium 156.0.8078.4, headless, do Playwright 1.64.1 | `just ext-bench-sound` |
| Data | 2026-10-22 | |

## Tamanho

O `just ext-size` lista cada arquivo de `extension/dist`, em bytes e com `gzip -9`. A comparação é
entre o build da `main` no commit `8fa35e3` (sem som) e o do branch do #79 no commit `b37aa69` (com
som), os dois com `node build.mjs`:

| Arquivo | Sem som | Com som | Diferença | Diferença com gzip |
| :-- | --: | --: | --: | --: |
| `content.js` | 25 407 | 28 171 | +2 764 | +935 |
| `options.js` | 12 933 | 13 413 | +480 | +112 |
| `popup.js` | 12 851 | 13 331 | +480 | +112 |
| `background.js` | 31 031 | 31 047 | +16 | +4 |
| os outros 4 arquivos | 4 371 | 4 371 | 0 | 0 |
| **total** | **86 593** | **90 333** | **+3 740 (+4,3%)** | **+1 883** |

- O `content.js` leva o código do som a cada página onde a extensão roda: os tons, a escolha da
  voz, os quatro eventos e o 🔊 do cartão.
- O `popup.js` e o `options.js` levam a chave "🔊 Som".
- O `background.js` só ganha o padrão `sound: false`.
- O total com gzip é o de um `tar.gz` da pasta. O pacote que a Chrome Web Store recebe é um `.zip`,
  e o tamanho dele não foi medido ⚠.

## Tempo de execução

O `just ext-bench-sound` roda `extension/bench/sound.mjs`. O script empacota o `sound.ts` real com o
esbuild, carrega numa página em branco do Chromium e mede com `performance.now()`. Foram três
rodadas de 200 efeitos cada:

| Medida | Rodada 1 | Rodada 2 | Rodada 3 |
| :-- | --: | --: | --: |
| primeiro efeito, que cria o `AudioContext` | 45,8 ms | 21,8 ms | 30,8 ms |
| efeitos seguintes, mediana | 0,1 ms | 0 ms | 0 ms |
| efeitos seguintes, p95 | 0,2 ms | 0,1 ms | 0,1 ms |
| busca da voz local, mediana | 0 ms | | |
| heap JS antes → depois | 1 480 309 → 1 685 841 B | → 1 679 449 B | → 1 682 685 B |

Como ler:

- **O primeiro efeito** cria o `AudioContext` uma vez por página. Isso só acontece com o som ligado
  e no primeiro evento, não ao carregar a página.
- **Os efeitos seguintes** só agendam as notas, e o áudio toca na thread de áudio do navegador. O
  Chromium arredonda o `performance.now()` em 0,1 ms sem isolamento de origem, então "0 ms" e
  "0,1 ms" querem dizer "no limite da resolução do relógio", não uma medida mais fina.
- **O heap** cresce ~200 KB, mas esse número inclui os 400 tempos que o próprio script guarda.
  Ele é um teto, não o custo do som sozinho ⚠.
- **As vozes:** o Chromium headless deste Linux não tem nenhuma voz do sistema (`voices: 0`). Nesta
  máquina, o 🔊 do cartão aparece desativado, com o título "Sem voz pt-BR local nesta máquina". O
  custo de ler em voz alta com uma voz local não foi medido ⚠.
- **O `AudioContext`** ficou `running` porque o script libera a reprodução sem gesto
  (`--autoplay-policy=no-user-gesture-required`). Numa página real, o Chrome pode deixá-lo
  suspenso até um clique ⚠: o primeiro efeito, então, pode não tocar.

## Desligado

Com o som desligado, que é o padrão, o `content.js` carrega os 2 764 bytes a mais, mas o código
não roda: o `AudioContext` não é criado, nenhum efeito é agendado e o cartão não tem o 🔊. Os testes
conferem isso (`extension/test/controller.test.ts`, "sound on: … off: silence").

## Repetir

```console
just ext-build && just ext-size   # o tamanho, arquivo por arquivo
just ext-bench-sound              # o tempo de execução, no Chromium do Playwright
```

Para comparar com uma versão sem som, faça o build de um commit anterior ao #79 numa worktree e
rode o `just ext-size` nas duas.
