# ISSUE_LABELS.md

Este arquivo lista os rótulos das issues e dos pull requests do ptbr-writer. Cada item tem um `tipo/*` e ao menos uma `área/*`. Um item aberto tem também um `status/*` e uma `prioridade/*`. Um item sobre um nível de rigor ou um nível de autor leva o rótulo `nível/*`.

O formato segue o `ISSUE_LABELS.md` do closed-source: uma família por seção, uma tabela por família, e a cor e a descrição de cada rótulo escritas aqui. Aqui as famílias têm nome em português, e as áreas são as partes deste repositório.

## tipo: a natureza da mudança

As palavras são as dos assuntos de commit (`feat`, `fix`, `docs`, `chore`).

| Rótulo | Cor | Significa |
|---|---|---|
| `tipo/feat` | `#A2EEEF` | Uma capacidade nova: regra, nível, skill, comando, página ou receita |
| `tipo/fix` | `#D73A4A` | Algo que estava quebrado e passa a funcionar |
| `tipo/docs` | `#0075CA` | Só documentação, metadados ou prosa |
| `tipo/chore` | `#C5DEF5` | Pins, atualizações e manutenção, sem mudar o comportamento |
| `tipo/meta` | `#5319E7` | Issue de acompanhamento: um resultado com lista "Pronto quando", fechada por vários PRs |

## área: a parte do repositório

| Rótulo | Cor | Significa |
|---|---|---|
| `área/skill` | `#C2E0C6` | `SKILL.md`, `skills/` e as regras PTW e CIE |
| `área/plugin` | `#C2E0C6` | `plugins/ptbr-writer`: manifesto, comandos e hooks do Claude Code |
| `área/opencode` | `#C2E0C6` | A distribuição e o uso no OpenCode, com o Qwen local |
| `área/eval` | `#BFD4F2` | `evals/` e o `claude plugin eval`: casos de regressão e métricas |
| `área/pdf` | `#BFD4F2` | A saída em PDF, pelo `to-pdf.sh` e pelo picasso |
| `área/figuras` | `#BFDADC` | Os diagramas Mermaid e as figuras Archify |
| `área/site` | `#BFDADC` | `site/`: a página em Gleam e o GitHub Pages |
| `área/ci` | `#006B75` | `.github/workflows/` e a porta de qualidade do CI |
| `área/release` | `#FBCA04` | Tags, `CHANGELOG.md` e `just release` |
| `área/repo` | `#FEF2C0` | README, AGENTS.md, licença, `REUSE.md` e os documentos do repositório |

## nível: o nível de rigor ou de autor

| Rótulo | Cor | Significa |
|---|---|---|
| `nível/científico` | `#7057FF` | O nível científico e as regras CIE |
| `nível/smart-note` | `#7057FF` | O nível smart-note |
| `nível/autor` | `#7057FF` | Os níveis de autor planejados (`machado-writer`, `saramago-writer`) e as regras de `REUSE.md` |

## status: o estado de um item aberto

| Rótulo | Cor | Significa |
|---|---|---|
| `status/triagem` | `#EDEDED` | Ainda não examinado, ou possivelmente obsoleto |
| `status/pronto` | `#0E8A16` | Delimitado; qualquer pessoa pode começar |
| `status/em-andamento` | `#FBCA04` | Um PR está aberto, ou PRs já fechados fizeram parte |
| `status/bloqueado` | `#5A5A5A` | Espera outra issue ou outro PR, que o corpo do item nomeia |

## prioridade

| Rótulo | Cor | Significa |
|---|---|---|
| `prioridade/alta` | `#B60205` | Fazer antes do resto do trabalho aberto |
| `prioridade/média` | `#FF9F1C` | A próxima, quando nenhuma alta está aberta |
| `prioridade/baixa` | `#FFF5B1` | Quando houver tempo |

## rsi: melhoria do harness

O PR #69 adiou a definição de RSI no `AGENTS.md` para um PR com fonte. Até lá, o rótulo marca só um uso: o erro que pede uma mudança no loop, e não só no arquivo.

| Rótulo | Cor | Significa |
|---|---|---|
| `rsi` | `#D4C5F9` | Erro que o harness deixou passar; a correção muda o loop para que ele não volte |

## Rótulos padrão do GitHub

Os rótulos padrão (`bug`, `enhancement`, `documentation`, …) ficam, para que nada que os filtre quebre. Não os aplique a itens novos: use `tipo/fix`, `tipo/feat` e `tipo/docs`.

## Como escolher

1. Escolha um `tipo/*`.
2. Escolha uma ou mais `área/*`: as partes do repositório que o item muda.
3. Se o item é sobre um nível, acrescente o `nível/*`.
4. Se a correção precisa mudar o harness, acrescente `rsi`.
5. Se o item está aberto, acrescente um `status/*` e uma `prioridade/*`. Ao fechar o item, tire esses dois.

## Sincronizar

Para criar ou atualizar os rótulos no GitHub a partir das tabelas deste arquivo:

```console
just labels
```

Para acrescentar um rótulo, acrescente a linha na tabela aqui e rode `just labels`. A receita lê cada linha de tabela que começa com um rótulo entre crases e cria o rótulo com o nome, a cor e a descrição da linha (`gh label create --force`).
