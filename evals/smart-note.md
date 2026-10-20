# Casos de regressão: smart-note

Escritos em 2026-10-20, ainda não rodados por um harness ⚠.

## 1. Uma ideia, uma nota

Entrada (`smart-note + estrito`, saída `nota`):

> O deploy falha quando o cache está frio. Além disso, o time de dados quer migrar para Parquet.

Esperado: duas notas, uma por ideia. Cada uma tem `id` no formato `AAAAMMDDhhmm`, `type: fleeting`,
`status: unverified` e pelo menos um link `[[...]]`; sem vizinha conhecida, o link é proposto e
marcado `⚠`. O texto de cada nota segue as regras de `estrito` (PTW-1: quem faz aparece).

## 2. As duas saídas

Entrada (`smart-note + científico`, sem pedir saída):

> Segundo Silva (2020), o método reduz o erro em 30%.

Esperado: o texto `RAW` e a `nota`. A nota é `type: literature`, `status: unverified` até a fonte
ser lida (PTW-10: a skill não completa a referência de memória), e o número leva unidade e origem
(PTW-11) ou `⚠`.

## 3. Pessoas

Entrada (`smart-note`, saída `nota`):

> Meu amigo João sugeriu usar SQLite no protótipo.

Esperado: `source` cita `friend1`, não "João". O nome não aparece na nota.
