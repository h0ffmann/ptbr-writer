---
name: cientifico
description: Escreve e revisa texto científico ou técnico com fontes (artigo, proposta, relatório) em pt-BR, e em EN quando o texto é bilíngue. Use ao redigir, citar, checar números ou preparar texto para revisão.
---

# cientifico

Texto com fonte verificável. Aplica PTW-1 a PTW-13 da skill `ptbr-writter`; as regras abaixo vêm a mais.

## Regras

### CIE-1: Fonte lida antes de citada

Busque a fonte e leia antes de citar. Título, autores, periódico e ano vêm do registro, não da memória. Resultado de busca, resumo e lista de referências de outro artigo não contam como leitura. Fonte que você não obteve não entra no texto.

### CIE-2: Procurado e não citado

Registre o que foi buscado e não citado: onde, com quais palavras, em que dia. Dúvida sobre existência vale como "não encontrada".

### CIE-3: DOI é link

Cite DOI como link (`doi.org/…`). Sem DOI, use URL com commit ou data de consulta.

### CIE-4: Fato que envelhece leva data

Versão, preço, contagem, estado de projeto: escreva a data da consulta. Sem data, marque `⚠`.

### CIE-5: Código é `arquivo:linha` num commit

Cite código como `caminho/arquivo:linha` com o commit fixo. Branch e "versão atual" não servem.

### CIE-6: Issue e PR de terceiros não são fonte

Não cite issue, PR nem discussão de repositório alheio, nem como `dono/repo#N`. Escreva o que o item contém, ancorado em release, commit, data ou DOI. Ler para aprender é permitido.

### CIE-7: Plano não é medição

Projeção continua projeção. Não relate ganho antes de mostrar correção (paridade). Nunca alargue tolerância para passar uma checagem. Causa só com medição; razão sem medição é proposta.

### CIE-8: Fonte privada fica privada

Não copie de repositório privado nem de nota reservada. Diga que existe e que não é pública.

### CIE-9: Humano decide

Tese, autoria e conclusões são do humano; pare e pergunte se a tarefa exige mudá-las. Não invente vivência do autor: deixe `[ANEDOTA DO AUTOR: …]`.

### CIE-10: Voz ativa, frase curta, número

Voz ativa. Frase com até 35 palavras e uma ideia. Palavra simples (`usar`, não `utilizar`). Adjetivo que elogia sem medir (`muito`, `robusto`, `de ponta`) sai ou vira número com unidade e comando.

### CIE-11: Figura carrega a evidência

Toda figura leva legenda com a conclusão, como ler, o que não mostra e a evidência. O `⚠` do texto aparece na figura. Texto e figura em desacordo: a figura é o erro.

## Formato de saída

1. Nível e destinatário assumidos.
2. Tabela de violações: regra (PTW ou CIE), trecho original, reescrita.
3. Texto final, com as marcas `(v)`, `(v, só título)` e `⚠`.
4. Nota sobre o que foi mantido de propósito.
5. Lista de `⚠`: o que falta checar e o que buscar.
6. Procurado e não citado (CIE-2).

## Limites

Não decide tese, autoria, licença nem publicação. Não garante que a afirmação é verdadeira: mostra como foi checada.
