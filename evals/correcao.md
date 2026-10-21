# Casos de regressão: correção completa (PTW-14 a PTW-17)

Escritos em 2026-10-21, ainda não rodados por um harness ⚠.

## 1. PTW-14 a 16: palavras, pontuação e concordância

Entrada (`leve`):

> bomd ia tudo bem com vc

Esperado: `Bom dia, tudo bem com você?` (PTW-14 separa `bomd ia` e troca `vc`; PTW-15 põe a vírgula
depois da saudação e o `?`; PTW-16 põe a maiúscula).

## 2. PTW-15: vírgula antes de "mas"

Entrada (`leve`):

> Terminei o relatório mas ainda falta revisar.

Esperado: `Terminei o relatório, mas ainda falta revisar.` Nenhuma vírgula entre sujeito e verbo.

## 3. PTW-16: concordância

Entrada (`leve`):

> os relatório está pronto

Esperado: `Os relatórios estão prontos.`

## 4. PTW-17: frase confusa vira uma frase reorganizada

Entrada (`neutro`):

> O sistema que o time que trabalha com dados mantém falha quando o cache, que foi criado ontem, e que ninguém limpou, enche, o que causa lentidão.

Esperado: uma frase reorganizada, no mesmo sentido e sem informação nova, por exemplo `Quando o cache criado ontem, que ninguém limpou, enche, o sistema mantido pelo time de dados falha e fica lento.` Mantém todos os fatos da entrada: a falha e a lentidão. Já leva as correções das PTW-8 e PTW-14 a 16.

## 5. PTW-17: frase já clara

Entrada (`neutro`):

> O deploy falha quando o cache está frio.

Esperado: sem reescrita. A frase volta como veio.
