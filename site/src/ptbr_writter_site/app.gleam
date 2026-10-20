//// The page. Every fact it shows comes from `content`, which site/content writes from SKILL.md,
//// the justfile and site/data/*.tsv; the sentences that tell the story are written here, in lower case (DESIGN.md).
//// The page is prerendered and needs no JavaScript.

import gleam/list
import gleam/result
import gleam/string
import lustre/attribute
import lustre/element.{type Element}
import lustre/element/html
import ptbr_writter_site/content.{
  type Level, type Link, type Origin, type Recipe, type Rule, type Step,
}

pub const repo = "https://github.com/h0ffmann/ptbr-writter"

const blob = "https://github.com/h0ffmann/ptbr-writter/blob/main/"

/// The star count, drawn by shields.io: an image, so the page still loads no JavaScript.
const stars =
  "https://img.shields.io/github/stars/h0ffmann/ptbr-writter?style=social&label=star"

pub fn view() -> Element(msg) {
  html.main([], [
    cover(),
    usage(),
    levels(),
    rules(),
    output(),
    limits(),
    recipes(),
    awesome(),
    roadmap(),
    footer(),
  ])
}

/// The markup SKILL.md uses inside a sentence: `a` → <code>a</code>, [t](u) → <a href="u">t</a>.
pub fn inline(text: String) -> List(Element(msg)) {
  text
  |> string.split("`")
  |> list.index_map(fn(part, i) {
    case i % 2 {
      1 -> [html.code([], [html.text(part)])]
      _ -> anchors(part)
    }
  })
  |> list.flatten
}

fn anchors(text: String) -> List(Element(msg)) {
  let found = {
    use #(before, rest) <- result.try(string.split_once(text, "["))
    use #(label, rest) <- result.try(string.split_once(rest, "]("))
    use #(url, after) <- result.map(string.split_once(rest, ")"))
    #(before, label, url, after)
  }
  case found {
    Ok(#(before, label, url, after)) -> [
      html.text(before),
      html.a([attribute.href(url)], [html.text(label)]),
      ..anchors(after)
    ]
    Error(_) -> [html.text(text)]
  }
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
      html.a([attribute.href("#awesome")], [html.text("awesome")]),
      html.a([attribute.href("#roadmap")], [html.text("roadmap")]),
    ]),
    html.div([attribute.class("actions")], [
      html.a([attribute.class("button"), attribute.href(blob <> "SKILL.md")], [
        html.text("ler a SKILL.md"),
      ]),
      html.a([attribute.class("ghost star"), attribute.href(repo)], [
        html.text("★ star me no github"),
        html.img([
          attribute.src(stars),
          attribute.alt(""),
          attribute.class("count"),
        ]),
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
    [
      html.ol([attribute.class("rules")], list.map(content.rules, rule)),
      ..optional("origin", content.origin)
    ],
  )
}

/// A paragraph of SKILL.md that may not be there: no text, no element.
fn optional(class: String, text: String) -> List(Element(msg)) {
  case text {
    "" -> []
    _ -> [html.p([attribute.class(class)], inline(text))]
  }
}

fn rule(r: Rule) -> Element(msg) {
  html.li([attribute.id(string.lowercase(r.id))], [
    html.span([attribute.class("id")], [html.text(r.id)]),
    html.h3([], [html.text(r.title)]),
    html.p([], inline(r.body)),
    html.p(
      [attribute.class("from")],
      content.origins
        |> list.filter(fn(o) { o.id == r.id })
        |> list.map(origin)
        |> list.intersperse([html.text(" · ")])
        |> list.flatten
        |> list.prepend(html.text("origem: ")),
    ),
  ])
}

/// "PTC-1, de kayquer/portugues-tecnico-controlado, por @kayquer", each name a link.
fn origin(o: Origin) -> List(Element(msg)) {
  [
    html.text(o.origin <> ", de "),
    html.a([attribute.href("https://github.com/" <> o.repo)], [
      html.text(o.repo),
    ]),
    html.text(", por "),
    html.a([attribute.href("https://github.com/" <> o.creator)], [
      html.text("@" <> o.creator),
    ]),
  ]
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

fn limits() -> Element(msg) {
  section(
    "limites",
    "limites",
    "o que a skill não faz, em nenhum nível.",
    optional("limits", content.limits),
  )
}

fn awesome() -> Element(msg) {
  section(
    "awesome",
    "awesome-ptbr-writter",
    "outros projetos para quem escreve texto técnico em português. o primeiro é o PTC original, de onde este fork vem; os outros foram achados numa lista curada, que vai ao lado.",
    [html.ul([attribute.class("awesome")], list.map(content.awesome, link))],
  )
}

fn link(l: Link) -> Element(msg) {
  html.li([], [
    html.a([attribute.href(l.url)], [html.code([], [html.text(l.name)])]),
    html.span([], [html.text(" " <> l.what <> " ")]),
    html.a([attribute.class("source"), attribute.href(l.source)], [
      html.text("fonte"),
    ]),
  ])
}

fn roadmap() -> Element(msg) {
  section(
    "roadmap",
    "roadmap: prontidão para produção",
    "o objetivo: outro repositório instala a skill e mantém sua documentação técnica em PT-BR e EN gastando menos tokens, sem reescrever regras de estilo no próprio código.",
    [html.ol([attribute.class("roadmap")], list.map(content.roadmap, step))],
  )
}

/// The status is a word first; its class only draws it (DESIGN.md).
fn step(s: Step) -> Element(msg) {
  let class = case s.status {
    "feito" -> "done"
    "em andamento" -> "doing"
    _ -> "todo"
  }
  html.li([attribute.class(class)], [
    html.span([attribute.class("status")], [html.text(s.status)]),
    html.h3([], [html.text(s.item)]),
    html.p([], [
      html.text(s.what),
      ..case s.evidence {
        "" -> []
        file -> [
          html.text(" · "),
          html.a([attribute.href(blob <> file)], [
            html.code([], [html.text(file)]),
          ]),
        ]
      }
    ]),
  ])
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
