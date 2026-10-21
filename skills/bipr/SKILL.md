---
name: bipr
description: Abre ou atualiza o PR do branch atual com título e corpo bilíngues, pt-BR primeiro e o espelho em inglês, gerados a partir dos commits. Use quando pedirem /bipr ou um PR bilíngue.
disable-model-invocation: true
---

# bipr

Do último commit a um PR publicado, com o corpo em pt-BR e o espelho em inglês. Argumentos: `--dry-run` (mostra o título e o corpo, não muda nada), `--base <branch>` (padrão: o branch padrão do repositório) e um número de PR (`#84`), que atualiza esse PR.

## 1. Recuse antes de mexer

Pare e diga o motivo quando:

- o HEAD está destacado, ou o branch é o branch padrão;
- há mudança em arquivo rastreado (`git status --porcelain --untracked-files=no` não vazio): peça o commit antes; arquivo não rastreado não bloqueia;
- não há commit à frente de `origin/<base>` depois de `git fetch origin <base>`;
- o PR do branch já foi mergeado ou fechado (`gh pr view --json state`): proponha um branch novo a partir de `origin/<base>`, não empurre.

## 2. Leia os commits

`git log --reverse --format='%H%n%s%n%b%n--' origin/<base>..HEAD`. Para cada commit, guarde o assunto, o corpo e os trailers (`Tested:`, `Closes #N`, `Fixes #N`, `Co-Authored-By:`). Não invente o que os commits não dizem: o que falta vira `⚠ não informado`.

## 3. Escreva o corpo

Duas metades com as mesmas seções e os mesmos fatos, pt-BR primeiro, separadas por `---`. Nenhuma metade tem fato que a outra não tem; não encha uma para igualar o tamanho da outra.

```
**Resumo**: <uma ou duas frases: o que mudou e por quê>

**Testado**: <os trailers Tested:, um por linha; ou ⚠ não informado>

**O que mudou**
- <uma linha por commit>

Closes #N

---

**Summary**: <the same, in English>

**Tested**: <the same>

**What changed**
- <the same, one line per commit>
```

- **pt-BR:** aplique as regras PTW da skill `ptbr-writer` no nível `estrito`: sujeito explícito, uma proposição por frase, termos consistentes.
- **Inglês:** frases curtas e voz ativa, um termo por conceito, no espírito do ASD-STE100.
- Código, caminhos, comandos, hashes e números ficam iguais nas duas metades; não traduza identificadores.
- `Closes #N` e `Fixes #N` aparecem uma vez, antes do `---`.
- A última linha do corpo é o conteúdo de `footer.md`, nesta pasta, copiado sem mudança. Ele substitui qualquer linha "Generated with" do agente; outra linha que o repositório exige (AGENTS.md, CLAUDE.md ou o modelo de PR) vem logo antes dele.

## 4. Escreva o título

O assunto do commit mais antigo, se houver um só tema, em pt-BR, com o prefixo do tipo (`feat:`, `fix:`) mantido. Até 72 caracteres. O título fica só em pt-BR; o corpo leva o inglês.

## 5. Publique

Com `--dry-run`, mostre o título e o corpo e pare.

Sem `--dry-run`:

1. `git push -u origin <branch>`; nunca `--force`, salvo se você reescreveu o histórico nesta execução, e então `--force-with-lease`.
2. Grave o corpo num arquivo temporário. Se não há PR aberto para o branch, `gh pr create --base <base> --title <título> --body-file <arquivo>`; se há, `gh pr edit <n> --title <título> --body-file <arquivo>`.
3. Mostre a URL do PR (`gh pr view --json url -q .url`).
