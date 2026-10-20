# ptbr-writter

Reescreva texto técnico em português brasileiro para leitura única, com sujeito explícito, proposições claras e termos consistentes.

## Uso

Invoque pelo nome:

```
/ptbr-writter
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

## Formato de saída

1. Nível e destinatário assumidos.
2. Tabela de violações: regra, trecho original, reescrita.
3. Texto final.
4. Nota sobre o que foi mantido de propósito.
5. Lista de `⚠`: afirmações sem checagem e referências a buscar, no nível `científico`.

## Origem das regras científicas

PTW-9 a PTW-13 adaptam, com palavras próprias, a skill `ouro-paper` e o estilo `.vale/styles/research` de [marola-dev/marola-research](https://github.com/marola-dev/marola-research) (`041cedd`), e a seção "Audience and evidence" do AGENTS.md de [h0ffmann/ww3-gpu](https://github.com/h0ffmann/ww3-gpu) (`319e4e2`).
