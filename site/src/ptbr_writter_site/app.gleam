//// The page. Every fact it shows comes from `content`, which site/content writes from SKILL.md and
//// the justfile; the sentences that tell the story are written here, in lower case (DESIGN.md).
//// The page is prerendered and needs no JavaScript.

import gleam/list
import gleam/string
import lustre/attribute
import lustre/element.{type Element}
import lustre/element/html
import ptbr_writter_site/content.{type Level, type Recipe, type Rule}

pub const repo = "https://github.com/h0ffmann/ptbr-writter"

const blob = "https://github.com/h0ffmann/ptbr-writter/blob/main/"

pub fn view() -> Element(msg) {
  html.main([], [
    cover(),
    usage(),
    levels(),
    rules(),
    output(),
    recipes(),
    footer(),
  ])
}

/// Markdown's inline code, the only markup SKILL.md uses inside a sentence: `a` → <code>a</code>.
pub fn inline(text: String) -> List(Element(msg)) {
  text
  |> string.split("`")
  |> list.index_map(fn(part, i) {
    case i % 2 {
      1 -> html.code([], [html.text(part)])
      _ -> html.text(part)
    }
  })
}

fn section(
  id: String,
  heading: String,
  intro: String,
  body: List(Element(msg)),
) -> Element(msg) {
  html.section([attribute.id(id), attribute.class("wrap")], [
    html.h2([], [html.text(heading)]),
    html.p([attribute.class("intro")], [html.text(intro)]),
    ..body
  ])
}

fn cover() -> Element(msg) {
  html.header([attribute.class("cover wrap")], [
    html.img([
      attribute.src("favicon.svg"),
      attribute.alt("uma pena de escrever, numa linha só"),
      attribute.class("mark"),
    ]),
    html.h1([], [html.text(content.title)]),
    html.p([attribute.class("lede")], inline(content.lede)),
    html.nav([attribute.class("jump")], [
      html.a([attribute.href("#uso")], [html.text("uso")]),
      html.a([attribute.href("#niveis")], [html.text("níveis")]),
      html.a([attribute.href("#regras")], [html.text("regras")]),
      html.a([attribute.href("#saida")], [html.text("saída")]),
      html.a([attribute.href("#receitas")], [html.text("receitas")]),
    ]),
    html.div([attribute.class("actions")], [
      html.a([attribute.class("button"), attribute.href(blob <> "SKILL.md")], [
        html.text("ler a SKILL.md"),
      ]),
      html.a([attribute.class("ghost"), attribute.href(repo)], [
        html.text("código no github"),
      ]),
    ]),
  ])
}

fn usage() -> Element(msg) {
  section(
    "uso",
    "uso",
    "chame a skill pelo nome, ou descreva a tarefa com suas palavras.",
    [
      html.ul(
        [attribute.class("asks")],
        list.map(content.usage, fn(u) {
          html.li([], [html.code([], [html.text(u)])])
        }),
      ),
    ],
  )
}

fn levels() -> Element(msg) {
  section(
    "niveis",
    "níveis de rigor",
    "sem nível pedido, a skill escolhe um e diz qual assumiu.",
    [html.dl([attribute.class("levels")], list.flat_map(content.levels, level))],
  )
}

fn level(l: Level) -> List(Element(msg)) {
  [
    html.dt([], [html.span([attribute.class("chip")], [html.text(l.name)])]),
    html.dd([], inline(l.use_)),
  ]
}

fn rules() -> Element(msg) {
  section(
    "regras",
    "regras",
    "cada regra corrige um tipo de ambiguidade. a tabela de violações da saída cita a regra pelo id.",
    [html.ol([attribute.class("rules")], list.map(content.rules, rule))],
  )
}

fn rule(r: Rule) -> Element(msg) {
  html.li([attribute.id(string.lowercase(r.id))], [
    html.span([attribute.class("id")], [html.text(r.id)]),
    html.h3([], [html.text(r.title)]),
    html.p([], inline(r.body)),
  ])
}

fn output() -> Element(msg) {
  section(
    "saida",
    "formato de saída",
    "toda resposta da skill vem nesta ordem.",
    [
      html.ol(
        [attribute.class("steps")],
        list.map(content.output, fn(s) { html.li([], inline(s)) }),
      ),
    ],
  )
}

fn recipes() -> Element(msg) {
  section(
    "receitas",
    "receitas",
    "o desenvolvimento roda com nix e just. cada linha é uma receita do justfile.",
    [
      html.dl(
        [attribute.class("recipes")],
        list.flat_map(content.recipes, recipe),
      ),
    ],
  )
}

fn recipe(r: Recipe) -> List(Element(msg)) {
  [
    html.dt([], [html.code([], [html.text("just " <> r.name)])]),
    html.dd([], inline(r.doc)),
  ]
}

fn footer() -> Element(msg) {
  html.footer([attribute.class("foot wrap")], [
    html.p([], [
      html.text("feito a partir de "),
      html.a([attribute.href(repo <> "/commit/" <> content.revision)], [
        html.text(content.revision),
      ]),
      html.text(" · MIT · "),
      html.a([attribute.href(repo)], [html.text("h0ffmann/ptbr-writter")]),
    ]),
    html.p([attribute.class("fork")], [
      html.text("fork conceitual do "),
      html.a(
        [
          attribute.href(
            "https://github.com/kayquer/portugues-tecnico-controlado",
          ),
        ],
        [html.text("Português Técnico Controlado")],
      ),
      html.text(", de kayquer (MIT). a metodologia original é de lá."),
    ]),
  ])
}
