# ptbr-writter

> Reescritor de texto técnico em português brasileiro, adaptado para uso pessoal e integração com agentes de IA.

---

## 🍴 Fork notice

Este repositório é um **fork conceitual e independente** do projeto [**Português Técnico Controlado (PTC)**](https://github.com/kayquer/portugues-tecnico-controlado), criado por [kayquer](https://github.com/kayquer) e licenciado sob MIT.

A ideia, a metodologia de regras de linguagem controlada e boa parte da inspiração vieram diretamente do PTC. O projeto original propõe 8 regras (PTC-1 a PTC-8), três níveis de rigor (`estrito`, `descritivo`, `leve`) e um modo bilíngue EN/PT — tudo isso foi desenvolvido no repositório original, não aqui.

Este fork existe para:

- adaptar o PTC às minhas necessidades pessoais e profissionais;
- experimentar novos formatos de integração (plugin, CLI, API);
- manter uma linha própria de evolução sem sobrecarregar o projeto upstream.

Se você procura a implementação original, completa e bem testada, use o repositório oficial:

🔗 **https://github.com/kayquer/portugues-tecnico-controlado**

---

## Objetivo

Tornar textos técnicos em português brasileiro mais claros, menos ambíguos e prontos para consumo humano ou por agentes de IA — mantendo o tom e as convenções que fazem sentido para o meu contexto.

---

## Estado atual

- [x] Criação do repositório
- [x] Licença MIT
- [x] README com créditos ao PTC original
- [x] Configuração para OpenCode + Qwen local
- [x] `justfile` com receitas de desenvolvimento
- [x] `flake.nix` para shell reprodutível
- [ ] Implementação completa da skill (`SKILL.md`)
- [ ] Casos de teste e regressão

---

## Desenvolvimento com OpenCode + Qwen

O repositório é desenvolvido e mantido com [OpenCode](https://github.com/opencode-ai) rodando contra um modelo Qwen local servido pelo `llama.cpp`.

### Requisitos

- [Nix](https://nixos.org/download/) com flakes habilitados
- NVIDIA driver e GPU com ~24 GB de VRAM (padrão: Qwen3.6-27B Q4_K_M, contexto 131072)
- OpenCode instalado e disponível no `PATH`

### Comandos

```bash
nix develop                # entra no shell de desenvolvimento
just                       # lista as receitas
just quality               # valida justfile, flake e configuração
just opencode-qwen         # sobe Qwen3.6-27B e abre o OpenCode
just opencode-status       # verifica se o servidor está saudável
just opencode-stop         # para o servidor na porta 8080
just opencode-logs         # acompanha o log do llama-server
```

A configuração do OpenCode está em `.opencode.json` (provider `llamacpp`, model `qwen3.6-27b`).

---

## Licença

MIT. Veja [LICENSE](LICENSE).

O projeto original PTC também é MIT. Ambos podem ser usados, modificados e redistribuídos livremente, desde que se mantenha o aviso de copyright.
