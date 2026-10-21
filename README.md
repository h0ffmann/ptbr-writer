<div align="center">

<img alt="Uma pena de escrever, numa linha só: desenho original" src="site/static/favicon.svg" width="120"/>

# ptbr-writer

<sub><a href="#instalar">instalar</a> · <a href="#usar">usar</a> · <a href="#níveis">níveis</a> · <a href="#regras">regras</a> · <a href="#eli5">ELI5</a> · <a href="#figuras">figuras</a> · <a href="#roadmap">roadmap</a> · <a href="#desenvolvimento">desenvolvimento</a> · <a href="#créditos">créditos</a> · <a href="CHANGELOG.md">versões</a> · <a href="https://h0ffmann.github.io/ptbr-writer/">o site</a></sub>

[![site](https://github.com/h0ffmann/ptbr-writer/actions/workflows/site.yml/badge.svg)](https://github.com/h0ffmann/ptbr-writer/actions/workflows/site.yml)
[![versão](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fraw.githubusercontent.com%2Fh0ffmann%2Fptbr-writer%2Fmain%2Fplugins%2Fptbr-writer%2F.claude-plugin%2Fplugin.json&query=%24.version&prefix=v&label=vers%C3%A3o&style=flat-square)](CHANGELOG.md)
[![license: MIT](https://img.shields.io/badge/license-MIT-1682D4?style=flat-square)](#licença)
[![leitura estimada: ~24 min](https://img.shields.io/badge/leitura%20estimada-~24%20min-1682D4?style=flat-square)](#tempo-de-leitura)
[![plugin: Claude Code](https://img.shields.io/badge/plugin-Claude%20Code-1682D4?style=flat-square)](#instalar)

</div>

🇬🇧 [Read in English](https://github.com/h0ffmann/ptbr-writer/blob/main/README.en.md)

## em um minuto

- **Reescreve texto técnico em português brasileiro para leitura única:** sujeito explícito, uma
  proposição por frase, termos consistentes. Serve a texto que humanos e agentes leem.
- **Cada pedido passa por um nível de rigor** (`estrito`, `descritivo`, `leve`, `científico`,
  `smart-note`), e a skill declara o nível que assumiu.
- **Não muda tese, autoria nem conclusão, e não busca fontes.** Marca com `⚠` o que não checou.
  Objetivo: manter a documentação PT-BR e EN de um repositório gastando menos tokens.

![Fluxo de um pedido: você escreve ptbr; na primeira vez, 4 perguntas gravadas em ptbr-writer.toml; depois a skill ptbr-writer (ou cientifico, se pede fontes) escreve um texto por nível e idioma; a saída é a resposta no console e, se pedido, arquivos .txt, .md, .pdf ou notes na pasta.](docs/img/em-um-minuto.png)

<details open>
<summary>Como ler esta figura</summary>

**Em uma frase.** Você escreve `ptbr: …`; na primeira vez o plugin faz 4 perguntas e guarda as
respostas, e depois a skill escreve um texto por nível e por idioma, na resposta e, se você pediu,
em arquivos.

**Como ler.** Da esquerda para a direita, na faixa de cima, que todo pedido percorre. A faixa de
baixo é a configuração e os arquivos. Seta com rótulo diz quando o caminho vale. Caixa com borda
laranja é onde você entra ou o que você recebe.

**Fora da figura.** Os níveis e as regras (PTW e CIE), o nível `smart-note`, o pedido
`ptbr reconfigurar` e a skill de roteamento `ptbr-routing`, que fica fora do plugin. A figura está em
português nas duas versões do README.

**Evidência.** As 4 perguntas, o `ptbr-writer.toml`, os formatos e os idiomas:
[`labs/ptbr-tools`](https://github.com/h0ffmann/picasso/tree/main/labs/ptbr-tools) (`ptbr-ask-levels`). A skill
`cientifico` para texto com fontes e o formato de saída: [`SKILL.md`](SKILL.md) e
[`skills/cientifico/SKILL.md`](skills/cientifico/SKILL.md). Fonte da figura:
[`docs/archify/em-um-minuto.json`](docs/archify/em-um-minuto.json); `just figure` a redesenha. Quando o
Claude escolhe a `cientifico` sozinho, pela descrição dela, não foi medido ⚠.

</details>

## instalar

A versão oficial é a **[v0.2.0](https://github.com/h0ffmann/ptbr-writer/releases/tag/v0.2.0)**.
Instale por tag: o plugin só muda quando você trocar a tag. Uma linha, num terminal:

```console
claude plugin install ptbr-writer --marketplace h0ffmann/ptbr-writer#v0.2.0
```

Ou, dentro do Claude Code:

```console
/plugin marketplace add h0ffmann/ptbr-writer#v0.2.0
/plugin install ptbr-writer@ptbr-writer
```

Para que todo mundo que abre um repositório use a mesma versão, ponha isto no `.claude/settings.json`
dele:

```json
{
  "extraKnownMarketplaces": {
    "ptbr-writer": {
      "source": { "source": "github", "repo": "h0ffmann/ptbr-writer", "ref": "v0.2.0" }
    }
  },
  "enabledPlugins": { "ptbr-writer@ptbr-writer": true }
}
```

**Atualizar:** troque a tag (`/plugin marketplace remove ptbr-writer` e o `add` de novo; num
repositório, o `ref`). O que muda em cada versão está no [`CHANGELOG.md`](CHANGELOG.md).

**A `main`, sem release:** para testar o que ainda não saiu numa versão, dentro do Claude Code:

```console
! curl -fsSL https://raw.githubusercontent.com/h0ffmann/ptbr-writer/main/scripts/install_current.sh | bash
```

O script remove a instalação anterior e o marketplace, instala da `main` e mostra o commit; depois,
`/reload-plugins` carrega o plugin na sessão. `PTBR_REF=<branch ou tag>` instala outra ref. A `main`
não é uma versão fixa: rode de novo para pegar o que mudou, e volte à tag para ficar estável.

### OpenCode

Uma linha, que baixa a tag, copia as skills `ptbr-writer`, `ptbr` e `cientifico` como arquivos
(sem links) para `~/.config/opencode/skills/` e instala o plugin do `ask-levels`:

```console
curl -fsSL https://raw.githubusercontent.com/h0ffmann/ptbr-writer/v0.3.0/scripts/install-opencode.sh | bash -s -- v0.3.0
```

`--project` no fim instala em `./.opencode/`. ⚠ `v0.3.0` ainda não existe: o script entra na
próxima release; até lá, rode `scripts/install-opencode.sh vX.Y.Z` de um clone da tag que você usa.
**Atualizar:** rode de novo com a tag nova. O plugin faz o que o hook faz no Claude Code: um pedido que começa com
`ptbr` recebe a instrução de perguntar níveis, formatos, pasta e idiomas, pela ferramenta
`question`; qualquer outro pedido não recebe nada. Testado no OpenCode 1.19.0: as três skills
carregam e a instrução chega ao modelo; ⚠ que o modelo responda com a ferramenta `question` não foi
medido (`opencode run` não a oferece). Precisa de `bash`, `curl` e `tar`; os binários são
`linux-x86_64` e `linux-aarch64`. A `ptbr-routing` não vai: ela depende de um hook do Claude Code.

### Chrome

Uma extensão (fora do plugin, ainda sem release) sublinha no campo onde você escreve o que o nível
pede para mudar. Instale do código-fonte:

1. `just ext-build`
2. Abra `chrome://extensions` e ligue o **Modo do desenvolvedor**.
3. Em **Carregar sem compactação**, escolha `extension/dist/`.
4. No popup da extensão, escolha o provider (Claude ou DeepSeek) e o modelo, cole a chave e clique em
   **Salvar**. Recarregue as abas que já estavam abertas. **Todas as configurações** abre a página
   com a lista de sites.

No Firefox (128 ou mais novo), `just ext-build-firefox` gera `extension/dist-firefox/`: em
`about:debugging#/runtime/this-firefox`, **Carregar extensão temporária…** e escolha o
`manifest.json` dessa pasta; em `about:addons`, na aba **Permissões** da extensão, libere os sites e
as APIs. Ela some quando o Firefox fecha. ⚠ Ainda não testada num Firefox de verdade.

## usar

| Comando | Faz |
| :------ | :-- |
| `/ptbr-writer:ptbr` | reescreve no nível pedido, ou classifica o texto e declara o nível |
| `/ptbr-writer:ptbr-writer` | o mesmo, com o nome completo |
| `/ptbr-writer:cientifico` | texto científico com fontes: artigo, proposta, relatório (regras CIE) |
| `/ptbr-writer:bipr` | abre ou atualiza o PR do branch atual, com o corpo em pt-BR e o espelho em inglês, a partir dos commits (`--dry-run` só mostra; [exemplo e custo](docs/bipr/exemplo-pr-49.md)) |

Ou descreva a tarefa: "reescreva este runbook em português controlado".

A partir da v0.2.0, um pedido que começa com `ptbr` ("ptbr: reescreva este texto", `/ptbr`)
usa o `ptbr-writer.toml`. Citar ptbr-writer no meio de uma frase não dispara nada. Na primeira
vez, sem esse arquivo, o Claude pergunta uma vez só, em múltipla escolha:

- **níveis:** um texto por nível;
- **formatos:** `.txt`, `.md`, `.pdf` e `notes` (smart-note);
- **pasta:** a raiz da sessão (o padrão), `docs/` ou outra, digitada;
- **idiomas:** pt-BR (o padrão) e/ou ASD-STE100, o inglês técnico simplificado; um texto por idioma.

e grava as respostas em `./ptbr-writer.toml`. Daí em diante não pergunta mais: edite o arquivo,
ou escreva `ptbr reconfigurar`.

```toml
levels = ["estrito"]     # vazio: a skill classifica e declara o nível
formats = ["md", "pdf"]  # txt, md, pdf, notes; vazio: só a resposta
folder = "docs"          # relativa à raiz da sessão, sem ".."
print = ["txt", "md"]    # o que aparece inteiro no console, além do arquivo
languages = ["pt-BR"]    # e/ou "asd-ste100"; o arquivo em ASD-STE100 termina em -ste100
```

Cada camada sobrescreve a anterior: os padrões do plugin
([`ptbr-writer.default.toml`](plugins/ptbr-writer/ptbr-writer.default.toml)),
`~/.config/ptbr-writer/ptbr-writer.toml`, `./ptbr-writer.toml` e, por último, o pedido
(`ptbr leve: ...` usa `leve`). Um valor desconhecido ou uma pasta fora da raiz é ignorado, com
aviso. O nome dos arquivos é um só:

```text
ptbr-writer-<AAAAMMDDThhmmZ>-<objetivo>-<nível>.<ext>
ptbr-writer-20261020T2215Z-reinicio-servico-estrito.md
```

O carimbo é em UTC e o mesmo para todos os arquivos de um pedido. Com 2 níveis e 3 formatos, são 6
arquivos. Para um `git add -A` não levá-los, ponha `ptbr-writer-*` no `.gitignore`. As notas de
`notes` seguem as regras de smart-note (`<id>-<slug>.md`). O `.pdf` usa o pandoc da máquina. Sem
ele, o Claude pergunta antes de baixar, pelo Nix, o
[`labs/ptbr-pdf`](https://github.com/h0ffmann/picasso/tree/main/labs/ptbr-pdf) do picasso (um binário Rust, `to-pdf`, com Typst e fontes embutidas). O binário ainda não está no cache.nixos.org: o Nix o compila do código-fonte. Medido com `just pdf-bench` (x86_64-linux), no picasso v0.2.0, com o binário enxuto ([execução](https://github.com/h0ffmann/ptbr-writer/actions/runs/39537436728)):

| download | em disco | montado na máquina | primeira vez, runner novo do GitHub (4 núcleos) |
| :------- | :------- | :----------------- | :---------------------------------------------- |
| 14 MiB, 6 caminhos do cache.nixos.org | 72 MiB, 7 caminhos | 25 MiB, 1 caminho (o `to-pdf`, compilado) | 892 s (15 min) |

O tempo depende da rede. Da segunda vez em diante, o Nix reusa o store.

![De onde vem o .pdf: o to-pdf.sh usa pandoc e xelatex se houver; senão baixa pelo Nix o picasso no commit que o to-pdf.sh fixa (PICASSO_REV), cujo labs/ptbr-pdf compila o crate to-pdf 0.1.0, com pulldown-cmark 0.13.4, typst 0.15.1 e typst-as-lib 0.17.0 e 6 fontes embutidas.](docs/img/pdf-dependencias.png)

<details open>
<summary>Como ler esta figura</summary>

**Em uma frase.** O `.pdf` sai do pandoc da máquina, se ele existir com o xelatex; senão, de um
binário Rust que o Nix compila a partir de um commit fixo do picasso, com cada biblioteca na versão
do `Cargo.lock`.

**Como ler.** Da esquerda para a direita. A faixa do meio é o caminho sem pandoc; a de baixo, as
bibliotecas Rust do binário; a de cima, o caminho local e o arquivo pronto. Caixa com borda laranja é
onde o caminho começa ou termina.

**Fora da figura.** As dependências das dependências (o `Cargo.lock` tem mais de 3 mil linhas), o
`flake.lock` do `labs/ptbr-pdf` e o Nix em si.

**Evidência.** [`plugins/ptbr-writer/scripts/to-pdf.sh`](plugins/ptbr-writer/scripts/to-pdf.sh)
(`PICASSO_REV`, os dois caminhos). No picasso, no commit `4bc03e5`: `labs/ptbr-pdf/Cargo.toml`
(as cinco dependências), `labs/ptbr-pdf/Cargo.lock` (`pulldown-cmark` 0.13.4, `typst`,
`typst-layout` e `typst-pdf` 0.15.1, `typst-as-lib` 0.17.0, `to-pdf` 0.1.0) e `labs/ptbr-pdf/fonts/`
(seis arquivos: Libertinus Serif e DejaVu Sans Mono). Fonte da figura:
[`docs/archify/pdf-dependencias.json`](docs/archify/pdf-dependencias.json). Que o `to-pdf.sh` e o
commit fixado em `forks/picasso` coincidem, `just pins` confere.

</details>

**Extensão do Chrome.** Depois de uma pausa de 1 s na digitação, o texto do campo vai ao provider
escolhido (Claude, modelo padrão `claude-haiku-5-5`; ou DeepSeek, `deepseek-flash` ⚠ lido na
documentação da DeepSeek em 2026-10-21, não medido). Só ele recebe o texto, e as chaves ficam só em
`chrome.storage.local`. Os trechos a mudar aparecem sublinhados; parar o mouse na palavra, ou clicar no número
no canto do campo, abre um cartão com **Aceitar** e **Ignorar**, e nada muda sem você aceitar.
Aceitar uma sugestão mantém as outras; voltar a digitar confere o texto de novo depois da pausa. Ligados de início: `web.whatsapp.com`
(`leve`), `mail.google.com` (`descritivo`) e `github.com` (`descritivo`). Os outros sites ficam
desligados até você clicar em **Ativar neste site** no popup (só em sites https), ou liberar nas
opções; os níveis são
`estrito`, `descritivo`, `leve` e `científico`. Campos com menos de 3 palavras e campos de senha são
ignorados; o histórico citado do Gmail não é enviado; acima de 4000 caracteres vai só o parágrafo do
cursor. Testes: `just ext-test` (Vitest) e `just ext-e2e` (Playwright, Chromium de verdade, servidor
falso, nenhuma API real); no CI, `.github/workflows/extension.yml`. ⚠ Em 2026-10-21 o dono do repo testou à
mão no WhatsApp Web, com uma chave real de um dos providers; o Gmail e o GitHub ainda não foram
testados com o editor de verdade.

## níveis

| Nível | Uso |
| :---- | :-- |
| `estrito` | runbooks, procedimentos, saídas para agentes |
| `descritivo` | documentação de sistema |
| `leve` | comunicados internos |
| `científico` | artigos, propostas, relatórios técnicos com fontes |
| `smart-note` | notas para um segundo cérebro; combina com outro nível (`smart-note + estrito`) |

Planejados, fora da `SKILL.md` e sem custo de contexto: `retórico`, `ensaístico`, `crônico`,
`anedótico`, `machado-writer`, `saramago-writer` (fonte: [`site/data/planned.tsv`](site/data/planned.tsv)).

> [!WARNING]
> Um nível de autor imita o estilo e nunca copia texto. Regras de citação e de domínio público em
> [`REUSE.md`](REUSE.md).

## regras

O texto de cada regra está na [`SKILL.md`](SKILL.md) e [no site](https://h0ffmann.github.io/ptbr-writer/#regras).
A origem de cada uma vem de [`site/data/origins.tsv`](site/data/origins.tsv). A skill `cientifico`
aplica PTW-1 a PTW-13 e acrescenta CIE-1 a CIE-12.

| id | regra | origem | criador |
| :- | :---- | :----- | :------ |
| PTW-1 | Quem faz, aparece | PTC-1 | [@kayquer](https://github.com/kayquer) |
| PTW-2 | Uma proposição por frase | PTC-2 | [@kayquer](https://github.com/kayquer) |
| PTW-3 | Modalidade e quantidade explícitas | PTC-3 | [@kayquer](https://github.com/kayquer) |
| PTW-4 | Verbo pleno | PTC-4 | [@kayquer](https://github.com/kayquer) |
| PTW-5 | Sintaxe plana | PTC-5 | [@kayquer](https://github.com/kayquer) |
| PTW-6 | Termos congelados | PTC-6 | [@kayquer](https://github.com/kayquer) |
| PTW-7 | Números, datas e unidades | PTC-7 | [@kayquer](https://github.com/kayquer) |
| PTW-8 | Ortografia PT-BR vigente | PTC-8 | [@kayquer](https://github.com/kayquer) |
| PTW-9 | Toda afirmação tem marca | ouro-paper; Audience and evidence | [@h0ffmann](https://github.com/h0ffmann) |
| PTW-10 | Nenhuma referência inventada | ouro-paper | [@h0ffmann](https://github.com/h0ffmann) |
| PTW-11 | Número com unidade e origem | ouro-paper; Audience and evidence | [@h0ffmann](https://github.com/h0ffmann) |
| PTW-12 | Sem alegação de pioneirismo | estilo Vale `PriorityClaims` | [@h0ffmann](https://github.com/h0ffmann) |
| PTW-13 | Fato separado de análise | ouro-paper | [@h0ffmann](https://github.com/h0ffmann) |

## tempo de leitura

Todo texto que o plugin gera leva uma linha como `Leitura estimada: ~3 min (612 palavras, 4 links, 2 blocos de código)`,
calculada por `ptbr-reading-time` (binário estático, sem dependências), nunca de cabeça.
É uma estimativa: o mesmo texto dá sempre o mesmo número, mas pessoas leem de 175 a 300 palavras por minuto.

| Parte | Como conta |
|-------|------------|
| Prosa | palavras (sem blocos de código, URLs e sintaxe Markdown; o código inline conta como palavra) ÷ 238 por minuto |
| Complexidade | texto técnico é mais lento: ×1,15 se a frase média passa de 20 palavras e ×1,15 se mais de 25% das palavras têm 10 letras ou mais (de 1,0 a 1,3) |
| Código | 10 s por linha não vazia de bloco de código |
| Links | 5 s por link, para decidir se o segue; o tempo de ler o destino não entra |
| Resultado | arredondado para cima, em minutos; até 60 s sai `< 1 min` |

A taxa de 238 palavras por minuto é a leitura silenciosa de não ficção em inglês na meta-análise de Brysbaert (2019).
⚠ O artigo não traz uma taxa para o português que pudemos ler (diz só que outras línguas se estimam pelo número de palavras
que precisam para dizer o mesmo que o inglês); o pt-BR usa as mesmas 238, sem escala. Os limiares de complexidade e os
tempos de código e de link são escolhas deste projeto, não do artigo.

```console
bash plugins/ptbr-writer/bin/run.sh reading-time README.md   # --lang en, --json, --self-test
```

## ELI5

Três pedidos reais, gerados pelo próprio plugin; o PDF sai do `.md` pelo `to-pdf.sh`. Os números vêm do `claude -p --output-format json` ([`RESULTS.tsv`](docs/eli5/RESULTS.tsv)).

| Prompt | PDF | Tokens | Nível |
| :-- | :-- | --: | :-- |
| [Runbook para refazer o deploy do site](docs/eli5/1-runbook-deploy.prompt.md) | [PDF](docs/eli5/ptbr-writer-20261021T0434Z-runbook-deploy-site-estrito.pdf) | 350.529 (entrada 16, saída 8.184, cache 342.329) | estrito |
| [Comunicado do tempo de leitura](docs/eli5/2-comunicado-tempo-leitura.prompt.md) | [PDF](docs/eli5/ptbr-writer-20261021T0436Z-comunicado-tempo-leitura-leve.pdf) | 184.688 (entrada 10, saída 2.401, cache 182.277) | leve |
| [Explicar o ptbr-writer a uma criança de 5 anos](docs/eli5/3-explique-ptbr-writer.prompt.md) | [PDF](docs/eli5/ptbr-writer-20261021T0436Z-explicacao-para-criancas-descritivo.pdf) | 232.366 (entrada 12, saída 3.434, cache 228.920) | descritivo |

## figuras

Cada figura é uma página do Archify, publicada no site:

- [O que acontece com um pedido ptbr](https://h0ffmann.github.io/ptbr-writer/figuras/em-um-minuto.html)
- [De onde vem o .pdf](https://h0ffmann.github.io/ptbr-writer/figuras/pdf-dependencias.html)

E o grafo do código deste repositório, gerado a cada build do site pelo `code-graph` do
[h0ffmann/picasso](https://github.com/h0ffmann/picasso) (tree-sitter, sem modelo):
[Grafo do código](https://h0ffmann.github.io/ptbr-writer/grafo/graph.html). Os nós que usam o picasso
levam ao [grafo do picasso](https://h0ffmann.github.io/ptbr-writer/grafo/picasso.html), gerado do
submódulo `forks/picasso` no mesmo commit dos outros pins.
A extensão do Chrome, em TypeScript, tem o seu:
[Grafo da extensão](https://h0ffmann.github.io/ptbr-writer/grafo/extensao.html), só de `extension/`.

## roadmap

Fonte: [`site/data/roadmap.tsv`](site/data/roadmap.tsv).

| Peça | Status | O que falta ou o que há |
| :--- | :----- | :---------------------- |
| [`SKILL.md`](SKILL.md) | em andamento | regras PTW e níveis de rigor; as regras ainda estão marcadas como em desenvolvimento |
| [`evals/`](evals/cientifico.md) | em andamento | casos de regressão escritos, ainda não rodados por um harness ⚠ |
| [skill científica](skills/cientifico/SKILL.md) | em andamento | regras CIE; casos em `evals/cientifico-cie.md`, ainda não rodados ⚠ |
| [roteamento](skills/ptbr-routing/SKILL.md) | feito | `route` escolhe a skill e o nível antes de carregar uma skill |
| [plugin](plugins/ptbr-writer/.claude-plugin/plugin.json) | feito | plugin do Claude Code, instalado com um comando na versão da release; o OpenCode lê o `SKILL.md` direto |
| [distribuição](CHANGELOG.md) | feito | release por tag com `just release`; a v0.2.0 é a versão oficial, instalada por tag |
| [opencode](scripts/install-opencode.sh) | em andamento | skills e plugin do `ask-levels` para o OpenCode, instalados por tag; testado no OpenCode 1.19.0 (skills e instrução), a ferramenta `question` não foi medida ⚠; o script entra na próxima release ⚠ |
| [retórica](skills/retorica/SKILL.md) | planejado | só o esqueleto, sem regras |

## awesome-ptbr-writer

Outros projetos para quem escreve texto técnico em português (fonte: [`site/data/awesome.tsv`](site/data/awesome.tsv)).

| Repositório | O que dá |
| :---------- | :------- |
| [kayquer/portugues-tecnico-controlado](https://github.com/kayquer/portugues-tecnico-controlado) | o original: 8 regras de português técnico controlado |
| [fabricioctelles/skills](https://github.com/fabricioctelles/skills) | skill "humanizar": remove padrões de texto de IA em pt-BR |
| [vale-cli/vale](https://github.com/vale-cli/vale) | linter de prosa com estilos próprios |
| [languagetool-org/languagetool](https://github.com/languagetool-org/languagetool) | corretor gramatical e de estilo com pt-BR |
| [hunspell/hunspell](https://github.com/hunspell/hunspell) | ortografia com dicionários pt-BR |
| [ajdavidl/Portuguese-NLP](https://github.com/ajdavidl/Portuguese-NLP) | índice de recursos de PLN para português |

## desenvolvimento

Tempo de execução do plugin: binários Rust estáticos (linux x86_64 e aarch64), sem python3 nem node. Requisitos para desenvolver: [Nix](https://nixos.org/download/) com flakes. Regras para agentes e para quem
contribui: [`AGENTS.md`](AGENTS.md).

```console
nix develop           # o shell de desenvolvimento
just quality          # tudo o que o CI checa, inclusive este README
just site             # o site em site/dist
just release-due      # o plugin mudou desde a última tag?
just labels           # os rótulos das issues, de ISSUE_LABELS.md, no GitHub
```

Sem o Claude Code (OpenCode ou outro agente que lê skills): copie [`SKILL.md`](SKILL.md) para a
pasta de skills do agente.

## créditos

- Regras PTW-1 a PTW-8: o PTC de [@kayquer](https://github.com/kayquer) (MIT).
- PTW-9 a PTW-13: a skill `ouro-paper` e o estilo `.vale/styles/research` de
  [marola-dev/marola-research](https://github.com/marola-dev/marola-research) (`041cedd`) e
  "Audience and evidence" de [h0ffmann/ww3-gpu](https://github.com/h0ffmann/ww3-gpu) (`319e4e2`).
- Skill científica: `scientific-writing` de
  [K-Dense-AI/scientific-agent-skills](https://github.com/K-Dense-AI/scientific-agent-skills/tree/a1043ef544e3a100c28e70a736b6cf32554d0677/skills/scientific-writing)
  (`a1043ef`, MIT, Copyright (c) 2025 K-Dense Inc.) e `artigo` de
  [labdaps/labskills](https://github.com/labdaps/labskills/tree/246c1646a51a5a754aedf786fef0544d528064e7/skills/artigo)
  (`246c164`, MIT, Copyright (c) 2026 LABDAPS), em palavras próprias. Citação pedida: Kassis, T.,
  Agarwal, V., He, Y., Patel, D., & Brueckner, A. M. (2026). *Scientific Agent Skills: A Library of
  Procedural Knowledge for Research Agents*. arXiv:2609.00065.
  <https://doi.org/10.48550/arXiv.2609.00065>. As regras CIE de nada inventado e de humano decide
  também vêm dos invariantes de [h0ffmann/closed-source](https://github.com/h0ffmann/closed-source)
  (`563c339`) e de [ouroboric-dev/ouroboric](https://github.com/ouroboric-dev/ouroboric) (`2858c11`).
- smart-note: *How to Take Smart Notes*, de Sönke Ahrens; o Zettelkasten de Niklas Luhmann; o
  fichário de Robert M. Pirsig para *Lila* ⚠.
- Retórica: a epígrafe é de Pirsig, *Zen and the Art of Motorcycle Maintenance* (1974), conferida em
  duas reproduções públicas, não na edição impressa ⚠. Base: o *Fédon* e o *Fedro*, de Platão.
- Nível por autor: a skill [orwell-writing](https://github.com/tamdogood/builder-essential-skills/tree/1be9984/skills/orwell-writing), de @tamdogood (MIT).
- Tempo de leitura: Brysbaert, M. (2019). How many words do we read per minute? A review and meta-analysis of reading rate. *Journal of Memory and Language*, 109, 104047. <https://doi.org/10.1016/j.jml.2019.104047> (238 palavras por minuto, não ficção em inglês).
- Extensão do Chrome (`extension/`): [@robkler](https://github.com/robkler), no #51. Ela não empacota código de terceiros: o `package.json` só tem dependências de desenvolvimento (build e testes), e `extension/src` não importa pacotes.
- Figuras: [Archify](https://github.com/tt-a1i/archify) (@tt-a1i, MIT), submódulo `vendor/archify` fixado em v3.0.1
  (`2ab3cae`), só para desenhar `docs/archify/`; não faz parte do plugin. Aparência: [h0ffmann/picasso](https://github.com/h0ffmann/picasso).
- Plugin e roteamento no molde do plugin e do `routing` de [marola-dev/marola-devkit](https://github.com/marola-dev/marola-devkit),
  de [@brunogbv](https://github.com/brunogbv).
  Layout do README: [h0ffmann/picasso](https://github.com/h0ffmann/picasso) e ww3-gpu. Stack do
  site: closed-source e ww3-gpu; Lustre (MIT) e a stdlib do Gleam (Apache-2.0) só no build; fontes
  do Google Fonts (SIL OFL); bandeira e assinatura de Machado de Assis do Wikimedia Commons, em
  domínio público.

## licença

MIT. Veja [`LICENSE`](LICENSE).

<sub>ptbr-writer é um fork conceitual e independente do <a href="https://github.com/kayquer/portugues-tecnico-controlado">Português Técnico Controlado (PTC)</a>, de <a href="https://github.com/kayquer">kayquer</a> (MIT). A metodologia e as regras originais são de lá.</sub>
