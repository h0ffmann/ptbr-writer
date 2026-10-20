---
description: Reescreve texto técnico em português brasileiro por nível de rigor (estrito, descritivo, leve, científico, smart-note). Use ao escrever, reescrever ou revisar runbook, documentação, comunicado ou notas em pt-BR.
---

# ptbr-writter

Reescreva texto técnico em português brasileiro para leitura única, com sujeito explícito, proposições claras e termos consistentes.

## Uso

Invoque pelo nome ou pelo alias:

```
/ptbr-writter
/ptbr
```

Ou descreva a tarefa:

- "reescreva este runbook em português controlado"
- "tire a ambiguidade deste texto"
- "revise este comunicado"

## Níveis de rigor

- `estrito`: runbooks, procedimentos, saídas para agentes.
- `descritivo`: documentação de sistema.
- `leve`: comunicados internos.
- `científico`: artigos, propostas, relatórios técnicos com fontes.
- `smart-note`: notas para um segundo cérebro; combina com outro nível (`smart-note + estrito`).

## Regras (em desenvolvimento)

As regras abaixo são uma adaptação pessoal do PTC. Quando você não especifica o nível, a skill classifica sozinha e declara o que assumiu.

### PTW-1: Quem faz, aparece

Sujeito explícito. Evite `-se` apassivador, `o mesmo` e clíticos de terceira pessoa ambíguos.

### PTW-2: Uma proposição por frase

Corte gerúndios conectores e subordinadas que escondem uma segunda ação.

### PTW-3: Modalidade e quantidade explícitas

`deve` = obrigação. Probabilidade por número ou intervalo, nunca por modal vago.

### PTW-4: Verbo pleno

`realizar a validação` → `validar`.

### PTW-5: Sintaxe plana

Cadeia de `de` curta, adjetivo pós-nominal, nada entre verbo e objeto.

### PTW-6: Termos congelados

Um conceito, um termo. Fixa a variante brasileira.

### PTW-7: Números, datas e unidades

Vírgula decimal. Datas ISO 8601. Unidades padronizadas.

### PTW-8: Ortografia PT-BR vigente

Acordo de 1990, com atenção a prefixos técnicos (`microsserviço`, `autosserviço`).

### PTW-9: Toda afirmação tem marca

No nível `científico`, cada afirmação leva `(v)`, `(v, só título)` ou `⚠`. A skill preserva as marcas e nunca remove um `⚠`. Afirmação reescrita perde o `(v)`: marque `⚠` e avise.

### PTW-10: Nenhuma referência inventada

A skill nunca adiciona, completa ou corrige citação ou DOI de memória. Referência incompleta vira `⚠` na tabela de violações.

### PTW-11: Número com unidade e origem

No nível `científico`, número sem unidade, ou sem comando ou fonte, é sinalizado. Adjetivo de quantidade (`muito mais rápido`) vira número ou `⚠`.

### PTW-12: Sem alegação de pioneirismo

`inédito`, `pela primeira vez`, `até onde sabemos`, `novo método`: estreite para o que foi consultado e quando, ou liste como não sabido.

### PTW-13: Fato separado de análise

Cada frase é fato com fonte ou leitura do autor, e diz qual. `A fonte não diz` não é `não é assim`.

## Limites

A skill não muda tese, autoria nem conclusões: quem decide é o humano. Ela não busca fontes; marca o que não foi checado.

## Smart note

Combina com outro nível: aplica as regras dele e entrega em notas. Duas saídas, `RAW` (só o texto) e `nota` (indexável, com front matter); sem pedido, entregue as duas.

1. Uma ideia, uma nota: uma afirmação, um fato ou uma pergunta. Cinco ideias, cinco notas.
2. Palavras próprias. Cite literal só o que precisa ficar exato, entre aspas, com a fonte.
3. Nome permanente `<id>-<slug>.md`; `id` é a hora UTC da criação, `AAAAMMDDhhmm`. O id não muda; o título pode.
4. Front matter: `id`, `title`, `type`, `status`, `source`, `tags`, `created` (ISO 8601).
5. `type`: `fleeting` (ideia crua, como veio), `literature` (o que uma fonte diz, com link ou DOI), `permanent` (afirmação própria, feita de outras notas).
6. `status`: `unverified`, `partial`, `verified`, `open-question` ou `approved`. `approved` é a palavra do dono, não checagem de fato; só o dono dá, e a nota diz quando.
7. Links: toda nota liga a pelo menos uma outra com `[[<id>-<slug>]]`, que existe. Sem vizinha conhecida, proponha o link e marque `⚠`.
8. Verdade: afirmação factual fica `unverified` até a fonte ser lida e citada na nota. Nunca invente fonte, citação ou número.
9. Pessoas: quem não é figura pública aparece por apelido (`friend1`, `prof1`), nunca pelo nome.
10. Tags: minúsculas, com hífen, poucas; reuse as que o cofre já tem.

## Formato de saída

1. Nível e destinatário assumidos.
2. Tabela de violações: regra, trecho original, reescrita.
3. Texto final.
4. Nota sobre o que foi mantido de propósito.
5. Lista de `⚠`: afirmações sem checagem e referências a buscar, no nível `científico`.
