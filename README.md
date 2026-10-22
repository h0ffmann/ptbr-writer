<div align="center">

<img alt="Uma pena de escrever, numa linha só: desenho original" src="site/static/favicon.svg" width="120"/>

# ptbr-writer

<sub><a href="#instalar">instalar</a> · <a href="#usar">usar</a> · <a href="#níveis">níveis</a> · <a href="#regras">regras</a> · <a href="#desenvolvimento">desenvolvimento</a> · <a href="#créditos">créditos</a> · <a href="CHANGELOG.md">versões</a> · <a href="https://h0ffmann.github.io/ptbr-writer/">o site</a></sub>

[![site](https://github.com/h0ffmann/ptbr-writer/actions/workflows/site.yml/badge.svg)](https://github.com/h0ffmann/ptbr-writer/actions/workflows/site.yml)
[![versão](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fraw.githubusercontent.com%2Fh0ffmann%2Fptbr-writer%2Fmain%2Fplugins%2Fptbr-writer%2F.claude-plugin%2Fplugin.json&query=%24.version&prefix=v&label=vers%C3%A3o&style=flat-square)](CHANGELOG.md)
[![license: MPL-2.0](https://img.shields.io/badge/license-MPL--2.0-1682D4?style=flat-square)](#licença)
[![leitura estimada: ~8 min](https://img.shields.io/badge/leitura%20estimada-~8%20min-1682D4?style=flat-square)](#tempo-de-leitura)
[![plugin: Claude Code](https://img.shields.io/badge/plugin-Claude%20Code-1682D4?style=flat-square)](#instalar)
[![extensão de navegador (Chrome): dev local (WIP)](https://img.shields.io/badge/extens%C3%A3o%20de%20navegador-dev%20local%20(WIP)-1682D4?style=flat-square&logo=googlechrome&logoColor=white)](#chrome)
[![extensão de navegador (Firefox): dev local (WIP)](https://img.shields.io/badge/extens%C3%A3o%20de%20navegador-dev%20local%20(WIP)-1682D4?style=flat-square&logo=firefoxbrowser&logoColor=white)](docs/EXTENSION_FIREFOX.md)
[![modelo local: Ollama](https://img.shields.io/badge/modelo%20local-Ollama-1682D4?style=flat-square&logo=ollama&logoColor=white)](#ollama)
[![modelo de decisão: Ollaya](https://img.shields.io/badge/modelo%20de%20decis%C3%A3o-Ollaya-1682D4?style=flat-square&logo=data:image/svg%2Bxml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAzMiAzMiI+PHJlY3Qgd2lkdGg9IjMyIiBoZWlnaHQ9IjMyIiByeD0iNyIgZmlsbD0iIzE3MTcxNyIvPjxnIHRyYW5zZm9ybT0idHJhbnNsYXRlKDEuNiAxLjQpIHNjYWxlKC45KSIgZmlsbD0ibm9uZSIgc3Ryb2tlPSIjZmZmIiBzdHJva2Utd2lkdGg9IjIuMyIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIj48cGF0aCBkPSJNNiA1LjUgMTAuNSA5LjVRMTYgNy41IDIxLjUgOS41TDI2IDUuNVYxOVEyNiAyNy41IDE2IDI3LjVUNiAxOVoiLz48Y2lyY2xlIGN4PSIxMS41IiBjeT0iMTUuNSIgcj0iMy4yNSIvPjxjaXJjbGUgY3g9IjIwLjUiIGN5PSIxNS41IiByPSIzLjI1Ii8+PGNpcmNsZSBjeD0iMTEuNSIgY3k9IjE1LjUiIHI9IjEuMiIgZmlsbD0iI2ZmZiIgc3Ryb2tlPSJub25lIi8+PGNpcmNsZSBjeD0iMjAuNSIgY3k9IjE1LjUiIHI9IjEuMiIgZmlsbD0iI2ZmZiIgc3Ryb2tlPSJub25lIi8+PHBhdGggZD0iTTE0Ljc1IDIxIDE2IDIzbDEuMjUtMiIvPjwvZz48L3N2Zz4K)](#ollaya)

</div>

🇬🇧 [Read in English](https://github.com/h0ffmann/ptbr-writer/blob/main/README.en.md)

## em um minuto

- **Reescreve texto técnico em português brasileiro para leitura única:** sujeito explícito, uma
  proposição por frase, termos consistentes. Serve a texto que pessoas e agentes leem.
- **Cada pedido passa por um nível de rigor**, e a skill declara o nível que assumiu.
- **Não muda tese, autoria nem conclusão, e não busca fontes.** Marca com `⚠` o que não checou.

![Fluxo do plugin do Claude Code: você escreve ptbr; na primeira vez, 4 perguntas gravadas em ptbr-writer.toml; depois a skill ptbr-writer (ou cientifico, se pede fontes) escreve um texto por nível e idioma; a saída é a resposta no console e, se pedido, arquivos .txt, .md, .pdf ou notes na pasta.](docs/img/em-um-minuto.png)

<details>
<summary>Como ler esta figura</summary>

**Em uma frase.** Você escreve `ptbr: …`, e a skill escreve um texto por nível e por idioma.

**Como ler.** Da esquerda para a direita. A faixa de cima é o caminho de todo pedido; a de baixo, a
configuração e os arquivos. A borda laranja marca onde você entra e o que você recebe.

**Fora da figura.** As regras, o nível `smart-note` e o pedido `ptbr reconfigurar`.

**Evidência.** [`docs/archify/em-um-minuto.json`](docs/archify/em-um-minuto.json), a fonte da figura.

</details>

## instalar

A versão oficial é a [v0.2.0](https://github.com/h0ffmann/ptbr-writer/releases/tag/v0.2.0). Num
terminal:

```console
claude plugin install ptbr-writer --marketplace h0ffmann/ptbr-writer#v0.2.0
```

Para fixar a versão num repositório, ponha no `.claude/settings.json` dele o marketplace
`h0ffmann/ptbr-writer` com `"ref": "v0.2.0"` em `extraKnownMarketplaces` e ative
`ptbr-writer@ptbr-writer` em `enabledPlugins`. Para atualizar, troque a tag.

### OpenCode

O script `scripts/install-opencode.sh` instala as skills e o plugin do `ask-levels` a partir de uma
tag. ⚠ Ele entra na próxima release; até lá, rode o script de um clone.

### Chrome

Uma extensão, ainda sem release, sublinha no campo onde você escreve o que o nível pede para mudar.

1. Rode `just ext-build`.
2. Em `chrome://extensions`, ligue o **Modo do desenvolvedor** e carregue `extension/dist/`.
3. No popup, escolha o provider e o modelo e clique em **Salvar**.

Nada muda no texto sem o seu clique em **Aceitar**. O Firefox 128+ usa `just ext-build-firefox`
([guia](docs/EXTENSION_FIREFOX.md)). Os providers:

- **Claude ou DeepSeek:** com a sua chave.
- <a id="ollama"></a>**Ollama (local):** `just ext-ollama` serve um modelo nesta máquina, sem chave.
- <a id="ollaya"></a>**Ollaya (local, só aponta):** `just ext-ollaya` serve um modelo de decisão. Ele
  aponta a regra que cada frase quebra e não escreve a troca.
- **Ollaya + Ollama:** com o Ollama, a chave **Ollaya escolhe o estilo** faz o Ollaya escolher o tom
  (conciso, socrático, amigável) em que o Ollama escreve.

⚠ Os providers locais não foram medidos em pt-BR.

## usar

| Comando | Faz |
| :------ | :-- |
| `/ptbr-writer:ptbr` | reescreve no nível pedido, ou classifica o texto e declara o nível |
| `/ptbr-writer:cientifico` | texto científico com fontes: artigo, proposta, relatório |
| `/ptbr-writer:bipr` | abre o PR do branch com o corpo em pt-BR e o espelho em inglês |

Ou descreva a tarefa: "reescreva este runbook em português controlado".

Um pedido que começa com `ptbr` usa o `ptbr-writer.toml`. Na primeira vez, o Claude pergunta os
níveis, os formatos, a pasta e os idiomas, e grava as respostas:

```toml
levels = ["estrito"]     # vazio: a skill classifica e declara o nível
formats = ["md", "pdf"]  # txt, md, pdf, notes
folder = "docs"
languages = ["pt-BR"]    # e/ou "asd-ste100"
```

Para mudar, edite o arquivo ou escreva `ptbr reconfigurar`. O pedido vence o arquivo: `ptbr leve:
...` usa `leve`. Os arquivos se chamam `ptbr-writer-<data>-<objetivo>-<nível>.<ext>`.

## níveis

| Nível | Uso |
| :---- | :-- |
| `estrito` | runbooks, procedimentos, saídas para agentes |
| `descritivo` | documentação de sistema |
| `leve` | comunicados internos |
| `científico` | artigos, propostas, relatórios técnicos com fontes |
| `smart-note` | notas para um segundo cérebro; combina com outro nível |

> [!WARNING]
> Um nível de autor, ainda planejado, imita o estilo e nunca copia texto ([`REUSE.md`](REUSE.md)).

## regras

As regras PTW-1 a PTW-13 estão na [`SKILL.md`](SKILL.md). A skill `cientifico` acrescenta as
regras CIE. O [site](https://h0ffmann.github.io/ptbr-writer/) mostra cada regra com a sua origem,
o roadmap, os projetos relacionados e as figuras:

- [Fluxo do plugin do Claude Code](https://h0ffmann.github.io/ptbr-writer/figuras/em-um-minuto.html)
- [Fluxo da extensão de navegador](https://h0ffmann.github.io/ptbr-writer/figuras/extensao-fluxo.html)
- [Fluxo do modelo local](https://h0ffmann.github.io/ptbr-writer/figuras/modelo-local.html)
- [De onde vem o .pdf](https://h0ffmann.github.io/ptbr-writer/figuras/pdf-dependencias.html)

## tempo de leitura

Todo texto que o plugin gera traz uma estimativa de leitura, calculada pelo binário
`ptbr-reading-time`, nunca de cabeça. Este README tem um limite de 10 minutos, que o `just quality`
confere.

## desenvolvimento

Requisito: [Nix](https://nixos.org/download/) com flakes. As regras para quem contribui, pessoa ou
agente, estão no [`AGENTS.md`](AGENTS.md).

```console
nix develop      # o shell de desenvolvimento
just quality     # tudo o que o CI checa
```

## créditos

- PTW-1 a PTW-8: o PTC de @kayquer (MIT).
- PTW-9 a PTW-13: a skill `ouro-paper` de marola-dev/marola-research e o "Audience and evidence" de
  h0ffmann/ww3-gpu.
- Skill científica: `scientific-writing` de K-Dense-AI/scientific-agent-skills (MIT, Copyright (c)
  2025 K-Dense Inc.) e `artigo` de labdaps/labskills (MIT, Copyright (c) 2026 LABDAPS), em palavras
  próprias. Citação pedida: Kassis, T. et al. (2026), *Scientific Agent Skills*, arXiv:2609.00065.
- smart-note: Sönke Ahrens, Niklas Luhmann e Robert M. Pirsig. Retórica: Platão e Pirsig.
- Nível por autor: a skill `orwell-writing` de @tamdogood (MIT).
- Tempo de leitura: Brysbaert (2019), *Journal of Memory and Language* 109, 104047.
- Extensão de navegador: @robkler, no #51.
- Figuras: Archify, de @tt-a1i (MIT). Plugin e roteamento: marola-dev/marola-devkit, de @brunogbv.
- Site: Lustre (MIT), a stdlib do Gleam (Apache-2.0), fontes do Google Fonts (SIL OFL), e a
  bandeira e a assinatura de Machado de Assis do Wikimedia Commons, em domínio público.

Os commits fixados de cada fonte estão no [`AGENTS.md`](AGENTS.md).

## licença

MPL-2.0. Veja [`LICENSE`](LICENSE). Quem distribuir um arquivo modificado do ptbr-writer publica
esse arquivo sob a MPL. A v0.2.0 e as versões anteriores continuam MIT. O que vem de terceiros
mantém a licença e o aviso de cada um.

<sub>ptbr-writer é um fork conceitual e independente do Português Técnico Controlado (PTC), de
kayquer (MIT). A metodologia e as regras originais são de lá.</sub>
