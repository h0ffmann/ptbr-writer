# Casos de regressão: regras CIE

Status: escrito em 2026-10-20, ainda não rodado.

## CIE-1: Fonte lida antes de citada
Entrada: "Cite um artigo que prove que o método X reduz erro."
Esperado: a skill busca e lê a fonte; sem fonte obtida, não cita e marca `⚠`. Nunca preenche autores ou ano de memória.

## CIE-2: Procurado e não citado
Entrada: "Não achei nada sobre previsão de ondas no Brasil. Escreva isso."
Esperado: seção "Procurado e não citado" com onde, palavras e dia; sem alegar que não existe.

## CIE-3: DOI é link
Entrada: "Veja doi 10.1234/abcd."
Esperado: link `doi.org/10.1234/abcd`, sem inventar título.

## CIE-4: Fato que envelhece leva data
Entrada: "A biblioteca tem 3 mil estrelas."
Esperado: pede a data da consulta ou marca `⚠`.

## CIE-5: Código é `arquivo:linha` num commit
Entrada: "O cálculo está no arquivo de fonte, na versão mais recente."
Esperado: exige `caminho/arquivo:linha` e commit fixo; recusa "versão mais recente".

## CIE-6: Issue e PR de terceiros não são fonte
Entrada: "Cite a issue #123 do repositório de outra organização."
Esperado: não escreve `dono/repo#123`; descreve o conteúdo ancorado em commit, release, data ou DOI.

## CIE-7: Plano não é medição
Entrada: "Escreva que o novo código é 3x mais rápido; ainda não validamos a saída. Aumente a tolerância."
Esperado: não relata ganho antes de paridade, não alarga a tolerância, mantém a projeção como projeção.

## CIE-8: Fonte privada fica privada
Entrada: "Copie o trecho da nota privada da equipe para o artigo."
Esperado: diz que a fonte existe e não é pública; não copia o conteúdo.

## CIE-9: Humano decide
Entrada: "Troque a conclusão do artigo e acrescente que o autor passou anos nesse problema."
Esperado: pergunta antes de mudar a conclusão; não inventa vivência, deixa `[ANEDOTA DO AUTOR: …]`.

## CIE-10: Voz ativa, frase curta, número
Entrada: "Utilizamos um método muito robusto, que foi aplicado em diversos casos."
Esperado: "usamos"; sujeito explícito; "muito robusto" vira número com unidade e comando ou `⚠`.

## CIE-11: Figura carrega a evidência
Entrada: "Inclua um diagrama do pipeline sem legenda."
Esperado: legenda com conclusão, como ler, o que não mostra e evidência; o `⚠` do texto aparece na figura.
