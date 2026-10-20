# DESIGN.md

The look of the website (`site/`, `just site`), in rules. It follows the manner of the author's
sister sites, h0ffmann/closed-source and h0ffmann/ww3-gpu, with its own palette.

## One sentence

Uma folha, a caneta de quem escreve e a caneta da revisão.

## The two pens

Each pen means one thing, everywhere on the page. Words always back up the colour: a reader who
does not see colour loses nothing.

| Pen | Hex | It means |
|---|---|---|
| `--pen` | `#1D3FD0` | the writer: the title, the links, the main button |
| `--review` | `#C8102E` | the correction: a rule's id (`PTW-1`), a level of rigor |

No third pen. A new colour is a new meaning, and it starts here.

## The ground

Light only, like paper: `--paper #FBF8F1` behind everything, `--sheet #FFFFFF` under code,
`--rule #E4DDCD` for the lines between sections, `--ink #1B1A17` for the text and
`--muted #6B665C` for what is secondary.

## Type

- Fraunces (800 and 600) for the title and the headings.
- Source Serif 4 for the prose.
- JetBrains Mono for data: rule ids, levels, commands, the navigation.

The fonts load from Google Fonts, as on the sister sites. No font file is stored here.

## Shape and words

- **Portuguese, lower case** for what the page says in its own voice: headings, buttons, intros.
  A fact read from a file keeps its own case (a rule's title, a recipe's comment).
- **Round.** Buttons, level chips and roadmap statuses are pills. A section is set apart by a
  line, not a box.
- **Status by shape, then colour.** A roadmap row says its status in words first. `feito` is a
  filled `--pen` pill, `em andamento` an outlined one, `planejado` a **dashed** `--muted` one:
  dashed means "not there yet".
- **Star me.** The second button is the GitHub pill, "★ star me no github", with the star count
  drawn by shields.io as an image. It is the only image from another host.

## Facts

Every fact on the page comes from a file of this repository, through `site/content`: the lede,
the usage, the levels, the rules and the output format from `SKILL.md`, the recipes from the
`justfile` (each recipe with a comment block above it, its first line), and the limits and the
origin of the rules from `## Limites` and `## Origem` when `SKILL.md` has them. Three tables hold
what no other file holds: `site/data/awesome.tsv`, the related repositories, each with the URL of
the curated list it was found in; `site/data/origins.tsv`, where each rule comes from (the rule of the source repository, the
repository and its creator, linked; a rule may have more than one row, and every rule needs one);
`site/data/roadmap.tsv`, production readiness, each row with a
status of `feito`, `em andamento` or `planejado` and the file that proves it. Never type a rule, a
level, a recipe, a repository or a status into `site/src`. A `SKILL.md` that loses one of its
sections, or a table row that is malformed, stops the build.

## Credit

The fork notice is the last line of the page, small, in the footer: it credits the Português
Técnico Controlado without leading the page.

## Logos and pictures

No logo is stored. The favicon, also the README's picture, is an original one-line drawing of a
writing quill.

## Motion

None. The page is prerendered and loads no JavaScript.

## Before you change the look

1. Change this file in the same pull request.
2. `just site-serve`, then look at the page at 1280 px and at 390 px.
3. Tab through the page: every link shows its focus ring.
