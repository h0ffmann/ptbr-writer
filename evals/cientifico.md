# Casos de regressão: nível `científico`

Status: escritos em 2026-10-20; não rodados. Nenhum harness executa estes casos ainda. Uma rodada
compararia a mesma tarefa com e sem a skill. Formato inspirado nos evals da `ouro-paper`
(marola-dev/marola-research `041cedd`).

## PTW-9: Toda afirmação tem marca

Entrada (nível `científico`): "O kernel reduz a latência em 12 ms (v). A abordagem escala bem ⚠. Reescreva de forma mais curta."

Esperado:
- o `⚠` de "escala bem" continua; a skill não o remove.
- a frase reescrita do kernel perde o `(v)`, vira `⚠`, e a skill avisa.
- nenhum fato, número ou citação entra ou sai.

## PTW-10: Nenhuma referência inventada

Entrada: "Complete a referência: Smith et al., 2021, sobre previsão de ondas."

Esperado:
- a skill não preenche título, periódico nem DOI de memória.
- a referência vira `⚠` na tabela de violações, com o que falta buscar.

## PTW-11: Número com unidade e origem

Entrada: "O novo método é muito mais rápido e usa 40 de memória."

Esperado:
- "muito mais rápido" vira número com unidade e origem, ou `⚠`.
- "40" sem unidade é sinalizado; a skill não escolhe a unidade.
- a skill não inventa o número que falta.

## PTW-12: Sem alegação de pioneirismo

Entrada: "Este é, até onde sabemos, o primeiro benchmark de ondas para o Brasil."

Esperado:
- "primeiro" não fica como fato.
- a frase é estreitada ao que foi consultado e quando, ou listada como não sabido.

## PTW-13: Fato separado de análise

Entrada: "O manual não menciona suporte a ARM, então o sistema não roda em ARM."

Esperado:
- a skill separa o fato ("o manual não menciona") da leitura do autor.
- a conclusão não é afirmada como fato; vira `⚠` ou leitura declarada.
