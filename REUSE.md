# REUSE.md

Regras de reúso do texto que os níveis de autor produzem. Um nível de autor imita o estilo de um
escritor da língua portuguesa; ele nunca copia trecho de obra, e respeita os direitos do autor.

## Hard rule: `machado-writer` cita Machado de Assis

Todo texto produzido ou reescrito no nível `machado-writer` leva, onde for usado, a linha:

> Estilo inspirado em Machado de Assis (1839–1908). Obra em domínio público.

A obra de Machado de Assis está em domínio público: ele morreu em 1908, e a lei brasileira
(Lei 9.610/1998, art. 41) protege a obra por 70 anos contados de 1º de janeiro do ano seguinte à
morte. Domínio público não dispensa crédito: quem usa o texto cita o autor. Quando o nível for
implementado, a saída da skill traz essa linha, e um caso em `evals/` falha sem ela.

## `saramago-writer`: só estilo

José Saramago morreu em 2010, e sua obra continua protegida por direito autoral. O nível
`saramago-writer` imita traços de estilo (períodos longos, pontuação mínima, diálogo sem
travessão) e nunca reproduz trecho, personagem ou enredo de uma obra dele. O texto produzido diz
que o estilo é inspirado em José Saramago.

## Um autor novo

Antes de criar um nível para outro autor, confira se a obra está em domínio público e escreva aqui
a regra de citação dele. Obra protegida serve só de modelo de estilo.
