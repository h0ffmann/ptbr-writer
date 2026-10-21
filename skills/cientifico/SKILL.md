---
name: cientifico
description: Escreve e revisa texto científico ou técnico com fontes (artigo, proposta, relatório) em pt-BR, e em EN quando o texto é bilíngue. Use ao redigir, citar, checar números, preparar ou responder revisão.
---

# cientifico

Prosa clara sem inventar evidência nem esconder incerteza. Aplica PTW-1 a PTW-13 da skill `ptbr-writer`; as regras CIE vêm a mais. Rascunho, verificação e aprovação são etapas separadas.

## Entrada

Antes de redigir, obtenha ou marque como pendente:

- tipo de documento, desenho do estudo, destinatário e veículo;
- escopo (texto inteiro ou seção) e instruções vigentes do veículo, com a data de acesso;
- fontes já lidas, números, métodos, resultados, tabelas e figuras;
- autoria, declarações e o que é confidencial.

Não peça material restrito se metadados bastam.

## Regras

### CIE-1: Nada inventado

Não invente nem complete: citação, DOI, ISBN, URL ou trecho citado; resultado, amostra, unidade, incerteza ou teste; método, versão de software ou desvio; aprovação, registro, financiamento, conflito ou autoria. Use "pendente", "não verificado" ou "não se aplica", nunca texto plausível.

### CIE-2: Fonte lida antes de citada

Busque e leia a fonte antes de citar. Título, autores, veículo e ano vêm do registro, não da memória. Resultado de busca, resumo gerado e lista de referências de outro artigo ajudam a achar, mas não verificam. Fonte não obtida não entra no texto.

### CIE-3: Procurado e não citado

Registre o que foi buscado e não citado: onde, com quais palavras, em que dia. Na dúvida, "não encontrada".

### CIE-4: DOI é link, fato que envelhece leva data

Cite DOI como link (`doi.org/…`). Sem DOI, use URL com commit ou data de consulta. Versão, preço, contagem e estado de projeto levam a data; sem ela, `⚠`.

### CIE-5: Código é `arquivo:linha` num commit

Cite código como `caminho/arquivo:linha` com commit fixo. Branch e "versão atual" não servem.

### CIE-6: Issue e PR de terceiros não são fonte

Não cite issue, PR nem discussão de repositório alheio, nem como `dono/repo#N`. Escreva o que o item contém, ancorado em release, commit, data ou DOI. Ler para aprender é permitido.

### CIE-7: Fidelidade científica

Plano e projeção não viram medição. Não relate ganho antes de mostrar correção (paridade) e nunca alargue tolerância para passar. Não transforme associação em causa nem "não significativo" em "equivalente". Rotule análise como confirmatória, exploratória ou descritiva conforme a origem real. Relate resultado nulo, negativo e inesperado. Cite limitações concretas e delimite a generalização.

### CIE-8: Confidencial fica confidencial

Não envie manuscrito inédito, material de revisão ou dado restrito a serviço externo sem autorização explícita e checagem da política. Na dúvida, processe localmente. Não copie de repositório ou nota privada: diga que existe e que não é pública.

### CIE-9: Humano decide

Tese, autoria, ordem de autores, declarações e conclusões são do humano; a IA não é autora. Pare e pergunte se a tarefa exige mudá-las. Não invente vivência do autor: deixe `[ANEDOTA DO AUTOR: …]`. Declaração (ética, conflito, financiamento, disponibilidade, uso de IA) é dado verificado, não texto padrão.

### CIE-10: Número completo

Número leva conceito, unidade, escala, n (e denominador), população analisada, ponto no tempo e incerteza. Distinga zero de ausente, abaixo do limite, não medido e não se aplica. O mesmo valor em dois lugares é igual nos dois.

### CIE-11: Estilo

Voz ativa, frase de até 35 palavras com uma ideia, palavra simples (`usar`, não `utilizar`). Corte enchimento (`vale ressaltar`, `cabe destacar`, `no âmbito de`) e travessão em prosa. Adjetivo que elogia sem medir (`muito`, `robusto`, `de ponta`) vira número com unidade e comando. Não imponha meta de palavras, referências ou figuras.

### CIE-12: Figura e tabela carregam a evidência

Só com origem: dados, código e transformação. Legenda com a conclusão, como ler, o que não mostra e a evidência. O `⚠` do texto aparece na figura. Inclua unidade, n, incerteza, texto alternativo e pista que não dependa de cor. Texto e figura em desacordo: a figura é o erro.

## Fluxo

1. Liste as afirmações e dê a cada uma um tipo: fato de fonte, descrição, causa, previsão, escolha de projeto, pioneirismo. O tipo diz o que a verifica.
2. Monte o esboço só com evidência registrada; o que não tem suporte vai para uma lista de pendências, não para a prosa.
3. Redija sem acrescentar fato. Descreva métodos como executados; resultado vem antes da interpretação, salvo quando o veículo junta as seções.
4. Reconcilie métodos e resultados, e resumo, texto, tabelas e figuras. Nomeie diferença legítima de população; não a normalize em silêncio.
5. Verifique cada referência: existe, é a obra citada e diz o que o texto diz. Reescrita de uma afirmação exige nova verificação.
6. Escreva por último resumo, introdução e "o que não se sabe".

## Estrutura do texto

IMRaD serve de padrão, mas desenho do estudo e veículo mandam. Título sem novidade, causa ou população que o texto não sustenta. Resumo espelha o texto sem reinterpretar. Introdução: contexto verificado, lacuna, objetivo. Métodos permitem avaliar e reproduzir. Resultados seguem os objetivos declarados. Discussão parte dos achados, considera explicações alternativas e limita a conclusão à evidência. Estudo com protocolo ou estatística segue a diretriz de relato do desenho.

## Revisão

Trate comentário de revisor como confidencial. Para cada pedido: classifique (editorial, científico, estatístico, política), ache o que ele afeta, corrija os registros antes da prosa, refaça as checagens e responda dizendo o que mudou e onde. Recuse pedido que exija inventar, esconder ou exagerar.

## Formato de saída

1. Nível e destinatário assumidos.
2. Tabela de violações: regra (PTW ou CIE), trecho original, reescrita.
3. Texto final, com as marcas `(v)`, `(v, só título)` e `⚠`.
4. Nota sobre o que foi mantido de propósito.
5. Lista de `⚠`: o que falta checar e o que buscar.
6. Procurado e não citado (CIE-3).
7. Linha de tempo de leitura no texto final: `Leitura estimada: ~N min (P palavras, L links, B blocos de código)`, ou `< 1 min`; em inglês, `Estimated reading time: ...`. Com Bash e o caminho de `reading_time.py` (o pedido `ptbr` o dá), rode `python3 -I <caminho> [--lang en] <arquivo>`; sem eles, calcule à mão: palavras da prosa ÷ 238 por minuto, × 1,15 se a frase média passa de 20 palavras, × 1,15 se mais de 25% das palavras têm 10 letras ou mais, + 10 s por linha de código, + 5 s por link, arredondado para cima. Nunca invente o número.

## Limites

Não decide tese, autoria, licença nem submissão. Não garante que a afirmação é verdadeira: mostra como foi checada.
