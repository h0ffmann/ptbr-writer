---
name: figure
description: "Add, change or explain a Mermaid diagram in this repo's Markdown (README.md, README.en.md, docs). Use when asked for a diagram, flowchart, mind map or 'a picture of' something, or to update one after its prose changed."
---

# figure

A diagram is a ```` ```mermaid ```` fence in the page it illustrates; GitHub draws it there. Every
fence has a header and a reading card.

## Steps

1. **Read the prose the figure sits beside.** Draw only what it says; the text is the source of
   truth and a figure that disagrees with it is a bug in the figure. If the page has a twin in
   the other language (`README.md` and `README.en.md`), both figures say the same.
2. **Write the fence.** First two lines, which Mermaid ignores:

   ```
   %% figure: <id>          lowercase words joined by '-', unique in the repo
   %% title: <question>     the question the figure answers
   ```

   Then the diagram, at most about 15 nodes; split it rather than grow it.
3. **Write the reading card** right after the closing fence, in the page's language:

   ```
   <details open>
   <summary>How to read this figure</summary>     (pt-BR: Como ler esta figura)

   **Takeaway.** one plain sentence, what to remember        (Em uma frase.)
   **How to read.** direction, what shapes and arrows mean   (Como ler.)
   **Not shown.** what was left out on purpose               (Fora da figura.)
   **Evidence.** file, paragraph or command, with ⚠ if unchecked  (Evidência.)

   </details>
   ```

   `open` keeps the explanation visible under the diagram; a reader should never have to find it.
   Each label is its own paragraph (blank lines between). The takeaway has no jargon. Evidence
   names the file or section the figure summarises; a discrepancy found between figure and text,
   or a claim not checked, is stated there with ⚠, not hidden.
4. **Check** that every node and label traces to a file you read. Never remove a ⚠ without doing
   the check.

## Visual grammar, the same in every figure

- Rectangle: a thing or a step. Diamond: a test, written as a question. Every test labels both
  its *yes* and its *no* edge.
- Solid arrow: makes or leads to the next thing. Dashed arrow (`-. label .->`): checks, tests or
  compares against. Label every arrow that is not obvious from its two ends.
- No self-loops: an arrow from a box back to itself says nothing about what it is compared with.
  Draw the check as a dashed arrow to the thing it is checked against.
- Number boxes that happen in order (`1 ·`, `2 ·`) so the sequence reads without the arrows.
- One flow direction per figure (`LR` or `TD`); groups (`subgraph`) for stages.
- Leave `classDef`, `style` and `%%{init}%%` out. A colour never carries meaning on its own; the
  label or shape says it.

## An interactive twin or a picture

Draw it with the `archify` skill, from the person's own install. It is not part of this repo's
plugin: never vendor or copy it here. Commit only its JSON source, `docs/archify/<slug>.json`,
never the HTML it renders. The Mermaid fence with its card stays the figure; the twin is an extra
and carries no claim the card lacks.
