# Modelos de decisão para a extensão

Levantamento feito em 2026-10-21, sobre o Ollaya no commit
[`39619acc`](https://github.com/ollaya-dev/ollaya/tree/39619accce0f5836dbeb60d2b14f5a056c2459ce)
e o Ollaya Bench, edição 2026.10. Nenhum número abaixo foi medido em pt-BR nem nas regras PTW ou CIE.

## Por que um modelo de decisão

Apontar que uma frase quebra uma regra é uma decisão. A pergunta é fechada: "qual regra esta frase
quebra?", com `nenhuma` e PTW-1..13 (mais CIE-1..12 no nível `científico`) como respostas. Um modelo
gerativo responde com texto livre. A extensão precisa então analisar esse texto como JSON, e a
resposta pode vir quebrada ou trazer uma regra que não existe. Um modelo de decisão lê o texto e as
opções e devolve uma probabilidade para cada opção numa única passada, sem gerar texto. A extensão
compara essa probabilidade com um limiar. O modelo de decisão também é menor, roda na CPU e não cobra
por token. Ele tem um limite: não escreve a frase corrigida. Ele só aponta.

## Candidatos

`Pontuação` é a acurácia corrigida pelo acaso (0 = chute, 100 = tudo certo) em 14 tarefas, com
`ECE` (erro de calibração, menor é melhor) no benchmark público. `CPU` e `GPU` são a mediana (p50)
de um pedido de cinco perguntas do preset `triage` via HTTP: CPU AMD Threadripper 3960X (24 núcleos),
GPU RTX 5090. Esses números são do Ollaya Bench, em tarefas em inglês e alemão, não em pt-BR.

| Modelo (tag exata) | Idiomas | Tamanho | Contexto | Opções | CPU p50 | GPU p50 / VRAM | Pontuação / ECE | Licença |
|---|---|---|---|---|---|---|---|---|
| `laya:multilingual` | mmBERT-base, cuja ficha lista `por` [1] | 322M [2] | 1.024 [2] | ~250, mas 256 tokens para pergunta e opções [3][4] | 515,5 ms [5] | 14,3 ms / 1,4 GB [6] | 36,7 / 0,155 [7][8] | Apache-2.0 [2] |
| `decima` = `decima:base` | a ficha lista `pt` entre 16 idiomas [9] | 321M [10] | 512 por pergunta [10] | 2–255; cada opção em linha própria de 64 tokens [10] | 1.212,5 ms [11] | 26,7 ms / 2,8 GB [12] | 31,0 / 0,052 [7][8] | Apache-2.0 [10] |
| `decima:small` | a ficha lista `pt` entre 20 idiomas [9] | 122M [10] | 512 [10] | 2–255, como `decima` [10] | 591,4 ms [11] | 14,2 ms / 1,3 GB [12] | 17,2 / 0,065 [7][8] | Apache-2.0 [10] |
| `largha` = `largha:150m` | mmBERT-small (`por`); o autor avaliou só `en` e `zh` [1][13] | 144M [13] | 512 [13] | 192 tokens para pergunta e opções [4][13] | 339,1 ms [11] | 17,8 ms / 1,0 GB [12] | 9,1 / 0,026, sem typed-decisions, em que foi treinado [7][8] | Apache-2.0 [13] |
| `winnow:e4b` | a ficha não declara idioma; base Gemma 4 E4B ⚠ [14] | Q8_0, 8,0 GB [15] | 8.192 [15] | 2–64 [15] | 4.095,3 ms [5] | 112,6 ms / 6,4 GB [6] | 59,9 / 0,071 [7][8] | Apache-2.0 [15] |
| `pplx-decider` = `pplx-decider:flash` | a ficha não declara idioma; base Qwen3.5-4B ⚠ [16] | 4B [17] | 8.192 [17] | 1–255 [16] | 5.533,6 ms [5] | 126,5 ms / 10,6 GB [6] | 73,7 / 0,016 [7][8] | Apache-2.0 [16] |
| `clef` = `clef:flash` | não declarado ⚠ | 9B [18] | – | – ⚠ | 12.477,2 ms [5] | 268,4 ms / 20,1 GB [6] | 65,1 / 0,025 [7][8] | Apache-2.0 [19] |

Descartados para a extensão, pela fonte de cada um:

- Só inglês: `nli` [20], `von` [21], `decider` [22], `laya:en` e `laya:typed-decisions` [2].
- Inglês e italiano: `snap` [23].
- Menos de 27 opções: `arbiter` (16) e `jevk5` (16) [19].
- Licença: `nimble:9b-v3` é CC BY-NC 4.0, sem uso comercial; `d1` exclui empresas com receita anual
  de US$ 10 milhões ou mais [19].
- Lentos na CPU: `nimble:9b` (62,0 s), `jeb:27b` (21,0 s) e `cygnet:12b` (20,7 s) [5].

Nomes de tag, conferidos nos manifestos do registro [24] (comando em "Como os fatos foram conferidos"):

- `decima` resolve para `decima:base`; `decima:small` e `decima:agent` são tags separadas.
- `largha` resolve para `largha:150m`.
- `laya` é o roteador: manda inglês para `laya:en` e o resto para `laya:multilingual` [2].
- `winnow` resolve para `winnow:12b`, não para `winnow:e4b`.
- `pplx-decider` resolve para `pplx-decider:flash`.

## O que limita as 27 opções

Todos os candidatos aceitam 27 opções no papel. Dois fatores reduzem esse número na prática.

**Orçamento de pergunta e opções (`laya:multilingual`, `largha`).** No layout `laya-markers-v1`, a
pergunta e as opções dividem um orçamento de tokens: 256 no `laya:multilingual` e 192 no `largha`
[4]. Cada opção começa com até 48 tokens. Quando as opções passam do orçamento, cada uma é cortada
para `(orçamento − 16) / número de opções` tokens, com mínimo de 4. A pergunta fica com o que sobra,
com mínimo de 8 tokens [3]. Pela conta do código, com 27 opções no `laya:multilingual`, cada opção
fica com 8 tokens e a pergunta com no máximo 40. Uma frase posta na pergunta seria cortada. ⚠ Conta
feita a partir do código, não medida.

A ficha do Laya também avisa: perguntas com mais de ~20 opções são mais fracas (0,425 no Banking77)
[2]. O `laya:multilingual` vem com temperatura 1,0, sem recalibração [2][25].

**Linha de 64 tokens por opção (`decima`).** O Decima codifica o estado e cada opção em separado, e
a ordem das opções não muda a resposta [10]. A pergunta inteira entra na linha do estado, então a
frase não se perde. Cada linha de opção junta o começo da pergunta e a opção em 64 tokens, então a
descrição longa de uma regra é cortada [10]. O estado tem 512 tokens. Um texto longo no `state` é
truncado, e `/api/decide` responde com `state_truncated: true` [10].

**Consequência para a extensão.** A extensão manda um pedido por frase, com a frase como `state`, uma
pergunta curta e, como opção, só o rótulo e o título da regra (`PTW-1`: `Quem faz, aparece`), sem o
primeiro parágrafo (`extension/src/decisions.ts`). ⚠ Que isso melhora o acerto não foi medido.

## Recomendação

Todo item desta seção é ⚠ não medido em pt-BR nem nas regras PTW e CIE.

- **Padrão (CPU): `decima`** (= `decima:base`). É o único candidato rápido na CPU cuja ficha lista
  `pt` [9]. Tem o menor ECE entre os modelos de CPU que leem português: 0,052 contra 0,155 do
  `laya:multilingual` [8]. Cada opção ganha sua linha, então 27 opções não disputam um orçamento
  comum, e a frase fica na linha do estado [10]. Custo: 1,2 s por cinco perguntas na CPU [11], contra
  0,5 s do `laya:multilingual` [5]. ⚠ Com 27 opções por frase o custo cresce, porque o Decima
  codifica uma linha por opção em cada pedido [10]. A latência com 27 opções não foi medida.
- **Alternativa rápida na CPU: `laya:multilingual`.** Tem pontuação um pouco maior que a do `decima`
  (36,7 contra 31,0) [7], mas tem calibração pior e o corte de opções descrito acima.
- **Escolha para GPU: `pplx-decider`** (= `pplx-decider:flash`). Tem a maior pontuação (73,7) e o
  menor ECE (0,016) da edição [7][8], 255 opções e 8.192 tokens [16][17]. Precisa de ~10,6 GB de
  VRAM [6]. Com menos VRAM, `winnow:e4b` (6,4 GB, 64 opções) [6][15]. Nenhum dos dois declara
  português na ficha. Os dois são modelos de 4B, lentos na CPU (4–6 s por cinco perguntas) [5].
- **Fora da lista: `largha`.** É o mais rápido na CPU, mas tem orçamento de 192 tokens para pergunta
  e opções [13], o autor só avaliou inglês e chinês [13] e tem pontuação 9,1 [7].

Lista sugerida para a extensão (`MODELS.ollaya` em `extension/src/settings.ts`), a primeira como
padrão:

```ts
ollaya: ['decima', 'laya:multilingual', 'decima:small', 'pplx-decider', 'winnow:e4b'],
```

A extensão usa esta lista. `decima` resolve para `decima:base` e `largha` para `largha:150m`.

## Comparação com o caminho gerativo

| | Ollaya (decisão, local) | Ollama + `qwen3.6:27b` (gerativo, local) | Nuvem (Claude, DeepSeek) | TypeSafe Jev (decisão, nuvem) |
|---|---|---|---|---|
| Saída | probabilidade por regra | JSON com trecho, regra e reescrita | JSON com trecho, regra e reescrita | probabilidade por opção [26] |
| Reescreve a frase | não | sim | sim | não |
| Analisa texto livre | não, a resposta é tipada | sim, e o JSON pode vir inválido | sim, e o JSON pode vir inválido | não |
| Calibração | publicada por modelo [8] | não publicada ⚠ | não publicada ⚠ | ECE 0,246, por terceiros [2] ⚠ |
| Custo | zero por pedido; só máquina | zero por pedido; só máquina | por token de entrada e de saída ⚠ | US$ 0,042 por milhão de tokens de entrada, saída grátis [26] |
| Latência | 0,3–1,2 s na CPU, 14–127 ms na GPU, por cinco perguntas [5][6] | ⚠ não medida | ⚠ não medida | ⚠ não medida |
| Texto sai da máquina | não | não | sim | sim |

- O caminho gerativo é o único que propõe a frase nova e o botão Aceitar.
- O modelo de decisão só aponta a regra. O cartão fica sem Aceitar.
- Uma combinação possível: o Ollaya aponta e o modelo gerativo reescreve só as frases apontadas.
  ⚠ Proposta, não medida.
- O Ollama 0.40 também serve nove modelos de decisão no mesmo formato TypeSafe [19]. Com os mesmos
  pesos do Nimble, o ECE é 0,022 no Ollaya e 0,115 no Ollama [19].
- A documentação do TypeSafe não fala de rodar o Jev na própria máquina [27]. Fontes secundárias
  dizem que o acesso é por lista de espera ⚠.

## O que falta medir

- ⚠ **não medido:** acurácia de qualquer modelo acima em frases pt-BR rotuladas por regra PTW ou CIE.
- ⚠ **não medido:** calibração em pt-BR. `THRESHOLD = 0.5` em `extension/src/decisions.ts` não foi
  ajustado.
- ⚠ **não medido:** latência com 14 e com 27 opções por pergunta, que é o caso da extensão. O
  benchmark usa o preset `triage`, de cinco perguntas.
- ⚠ **não medido:** se a frase sobrevive ao orçamento de tokens no `laya:multilingual`. Teste: o
  campo `usage.input_tokens` e a resposta com e sem a descrição longa nas opções.

Como medir, sem inventar número:

1. Criar `evals/ollaya-ptw.md` no formato de `evals/cientifico-cie.md`: para cada regra, frases que
   a quebram, com a regra esperada, e frases que não quebram nenhuma, com `nenhuma` esperado. A
   extensão grifa uma vez por frase, então cada frase de teste quebra uma regra só.
2. Rodar cada caso em cada tag e contar acertos por regra e o ECE com o limiar atual.
3. Medir a latência com o próprio Ollaya, que devolve `total_duration`, `load_duration` e
   `eval_duration` em nanossegundos [25b]. Aquecer o modelo antes (`keep_alive`) e repetir:

   ```sh
   curl -s http://127.0.0.1:11435/api/decide -d @pedido-27-opcoes.json \
     | jq '{model, total_ms: (.total_duration/1e6), eval_ms: (.eval_duration/1e6), usage, state_truncated}'
   ```

   `pedido-27-opcoes.json` é um dos corpos que `decideAsks` monta para o nível `científico`. Registrar a
   CPU, a versão do Ollaya e o commit junto com cada número.
4. Só depois trocar o padrão ou o limiar, e tirar o `⚠` do item medido.

## Como os fatos foram conferidos

- Tags: `git clone` do Ollaya no commit `39619acc` e, em `registry/v2/library/<modelo>/manifests/`,
  `sha256sum` de `latest` comparado com o de cada tag.
- Latência de CPU e GPU e ECE: `jq` sobre `results/runs/2026-10-{14,19,20}-latency-rtx5090-{cpu,cuda}.json`
  e `results/runs/2026-10-{14,19,20}-public-benchmark-rtx5090-cuda.json`, com o valor mais recente
  de cada modelo.
- Orçamento de tokens: blobs de configuração em `ollaya.dev/blobs/` (`head_max_len` 256 para
  `laya:multilingual`, 192 para `largha:150m`) e `crates/ollaya-decision/src/layout.rs`.
- Idiomas: `curl https://huggingface.co/api/models/<repo>` e o campo `cardData.language`.

## Fontes

Todas acessadas em 2026-10-21. `@39619acc` = `https://github.com/ollaya-dev/ollaya/blob/39619accce0f5836dbeb60d2b14f5a056c2459ce`.

1. Ficha do mmBERT, idiomas `por` entre 1.811: https://huggingface.co/jhu-clsp/mmBERT-base, https://huggingface.co/jhu-clsp/mmBERT-small
2. Laya no Ollaya: `@39619acc/site/content/library/laya.md`
3. Orçamento de opções: `@39619acc/crates/ollaya-decision/src/layout.rs`
4. API, limites por modelo (§5.3): `@39619acc/docs/api.md`; `head_max_len` em https://ollaya.dev/blobs/sha256-70fd2a20d3e0f259063958071637b2047a01f4d8dd5681ab2f3d647ee9193fac (`laya:multilingual`) e https://ollaya.dev/blobs/sha256-80dc4d8477bd3c79bb504294a5a28a060114dc2811bdcc2cd0f62f1ec816e987 (`largha:150m`)
5. Latência na CPU, 2026-10-14 e 2026-10-19: `@39619acc/results/runs/2026-10-14-latency-rtx5090-cpu.json`, `@39619acc/results/runs/2026-10-19-latency-rtx5090-cpu.json`
6. Latência e VRAM na GPU: `@39619acc/results/runs/2026-10-14-latency-rtx5090-cuda.json`, `@39619acc/results/runs/2026-10-19-latency-rtx5090-cuda.json`, `@39619acc/results/runs/2026-10-20-latency-rtx5090-cuda.json`
7. Pontuação, edição 2026.10: https://bench.ollaya.dev e `@39619acc/results/editions/2026.10.json`
8. ECE (`pooled_ece`): `@39619acc/results/runs/2026-10-14-public-benchmark-rtx5090-cuda.json`, `-10-19-` e `-10-20-`
9. Fichas do Decima, idiomas: https://huggingface.co/amyrmahdy/decima-base, https://huggingface.co/amyrmahdy/decima-small
10. Decima no Ollaya: `@39619acc/docs/families/decima.md`
11. Latência na CPU, 2026-10-20 (Decima, Largha): `@39619acc/results/runs/2026-10-20-latency-rtx5090-cpu.json`
12. Latência e VRAM na GPU, 2026-10-20: `@39619acc/results/runs/2026-10-20-latency-rtx5090-cuda.json`
13. Largha no Ollaya: `@39619acc/site/content/library/largha.md`; ficha: https://huggingface.co/Phocinae/Phocinae-Largha-150M-v1
14. Ficha do Winnow-E4B: https://huggingface.co/EldanRing/Winnow-E4B
15. Winnow no Ollaya: `@39619acc/site/content/library/winnow.md`
16. Ficha do pplx-decider-flash v1.2: https://huggingface.co/perplexity-ai/pplx-decider-flash-v1.2
17. pplx-decider no Ollaya: `@39619acc/site/content/library/pplx-decider.md`
18. Clef no Ollaya: `@39619acc/site/content/library/clef.md`
19. README do Ollaya (modelos, licenças, Ollama 0.40): `@39619acc/README.md`
20. NLI no Ollaya: `@39619acc/docs/families/nli.md`
21. Von no Ollaya: `@39619acc/docs/families/von.md`
22. Decider no Ollaya: `@39619acc/docs/families/decider.md`
23. Snap no Ollaya: `@39619acc/docs/families/snap.md`
24. Manifestos do registro: `@39619acc/registry/v2/library/`
25. Temperaturas do `laya:multilingual` (1,0): https://ollaya.dev/blobs/sha256-316915b3e458dcd3df2154562613132542e20244103338f6a150b712e9185e10
    - 25b. Campos de tempo de `/api/decide` (§7.3): `@39619acc/docs/api.md`
26. Preço e limites do TypeSafe Jev: https://docs.typesafe.ai/models
27. TypeSafe, System One: https://docs.typesafe.ai/concepts/system-one
