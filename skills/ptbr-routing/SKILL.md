---
name: ptbr-routing
description: Escolhe a skill e o nível de rigor antes de carregar uma skill de escrita em pt-BR. Use quando o pedido é escrever, reescrever ou revisar texto em português e não diz o nível nem a skill.
---

# Roteamento de um pedido de escrita

`route` é `bash .claude/skills/ptbr-routing/bin/run.sh "<pedido>"` (binário estático, sem dependências). A
resposta é uma linha: a skill a carregar e o nível.

- Carregue só a skill que a linha nomeia. Não leia as outras.
- Pedido que já diz o nível ou a skill: não rode `route`.
- Sem correspondência: carregue `ptbr-writer` sem nível. Ela classifica o texto e declara o nível.
- Mais de um nível empatado: escolha um e declare-o na saída.
- `smart-note + <nível>`: aplique as regras do nível e entregue em notas.
- Com o plugin instalado, a skill `X` é `ptbr-writer:X`.
- Sem o binário para o seu sistema: leia `routes.tsv`, ao lado deste arquivo, uma linha por nível com suas palavras.
