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

## Formato de saída

1. Nível e destinatário assumidos.
2. Tabela de violações: regra, trecho original, reescrita.
3. Texto final.
4. Nota sobre o que foi mantido de propósito.
