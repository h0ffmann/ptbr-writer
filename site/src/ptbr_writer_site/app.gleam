//// The page. Every fact it shows comes from `content`, which site/content writes from SKILL.md,
//// the justfile and site/data/*.tsv; the sentences that tell the story are written here, in lower case (DESIGN.md).
//// The page is prerendered; its one script, in page.gleam, only adds the copy button.

import gleam/list
import gleam/result
import gleam/string
import lustre/attribute
import lustre/element.{type Element}
import lustre/element/html
import ptbr_writer_site/content.{
  type Level, type Link, type Origin, type Planned, type Recipe, type Rule,
  type Step,
}

pub const repo = "https://github.com/h0ffmann/ptbr-writer"

const blob = "https://github.com/h0ffmann/ptbr-writer/blob/main/"

/// The two public-domain images of the cover, credited to their Wikimedia Commons pages.
const commons = "https://commons.wikimedia.org/wiki/File:"

/// The skill that applies Orwell's six rules, pinned: the model of an author level.
const orwell =
  "https://github.com/tamdogood/builder-essential-skills/tree/1be9984/skills/orwell-writing"

/// The star count, drawn by shields.io: an image, so the page still loads no JavaScript.
const stars =
  "https://img.shields.io/github/stars/h0ffmann/ptbr-writer?style=social&label=star"

pub fn view() -> Element(msg) {
  html.main([], [
    cover(),
    usage(),
    levels(),
    rules(),
    output(),
    smart_note(),
    limits(),
    recipes(),
    figuras(),
    eli5(),
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

pub fn section(
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
    html.p([attribute.class("kicker")], [
      html.img([
        attribute.src("img/bandeira-do-brasil.svg"),
        attribute.alt("bandeira do Brasil"),
        attribute.class("flag"),
      ]),
      html.span([attribute.class("first")], [html.text("pt-BR first")]),
      html.text("skill de escrita técnica · português brasileiro"),
    ]),
    // Calligraphy on the page: Machado de Assis's signature, public domain, credited in the
    // footer as REUSE.md asks.
    html.img([
      attribute.src("img/assinatura-machado-de-assis.png"),
      attribute.alt("assinatura de Machado de Assis"),
      attribute.class("calligraphy"),
    ]),
    html.div([attribute.class("brand")], [
      html.img([
        attribute.src("favicon.svg"),
        attribute.alt("uma pena de escrever, numa linha só"),
        attribute.class("mark"),
      ]),
      html.h1([], [html.text(content.title)]),
    ]),
    html.p([attribute.class("lede")], inline(content.lede)),
    install(),
    html.nav([attribute.class("jump")], [
      html.a([attribute.href("#instalar")], [html.text("instalar")]),
      html.a([attribute.href("#uso")], [html.text("uso")]),
      html.a([attribute.href("#niveis")], [html.text("níveis")]),
      html.a([attribute.href("#regras")], [html.text("regras")]),
      html.a([attribute.href("#saida")], [html.text("saída")]),
      html.a([attribute.href("#smart-note")], [html.text("smart-note")]),
      html.a([attribute.href("#receitas")], [html.text("receitas")]),
      html.a([attribute.href("#figuras")], [html.text("figuras")]),
      html.a([attribute.href("#eli5")], [html.text("ELI5")]),
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
      grafo_pill(
        "grafo/graph.html",
        "grafo interativo do código, gerado pelo code-graph do picasso",
        "Grafo do código",
      ),
      grafo_pill(
        "grafo/picasso.html",
        "grafo do código do picasso, no commit que este repositório fixa",
        "Grafo do picasso",
      ),
      grafo_pill(
        "grafo/extensao.html",
        "grafo do código TypeScript da extensão de navegador, Chrome e Firefox (extension/)",
        "Grafo da extensão",
      ),
      html.a(
        [
          attribute.class("ghost figure-pill"),
          attribute.href("bipr.html"),
          attribute.title("a skill /bipr e o PR que ela abriu"),
        ],
        [
          html.img([
            attribute.src("favicon.svg"),
            attribute.alt(""),
            attribute.width(20),
            attribute.height(20),
          ]),
          html.text("/bipr"),
        ],
      ),
      // The whitepaper, one pill per language; `just site` copies WHITEPAPER.pdf and WHITEPAPER.en.pdf.
      pdf_pill(
        "whitepaper/WHITEPAPER.pdf",
        "o whitepaper do ptbr-writer, em português do Brasil",
        "Whitepaper (pt-BR)",
      ),
      pdf_pill(
        "whitepaper/WHITEPAPER.en.pdf",
        "the ptbr-writer whitepaper, in English",
        "Whitepaper (EN)",
      ),
      // Each Archify page of docs/archify/, named by its title and marked with Archify's icon,
      // beside the GitHub pill.
      ..list.map(content.figures, fn(f) {
        html.a(
          [
            attribute.class("ghost figure-pill"),
            attribute.href("figuras/" <> f.slug <> ".html"),
            attribute.title("figura interativa, desenhada com o Archify"),
          ],
          [
            html.img([
              attribute.src(archify_icon),
              attribute.alt("Archify"),
              attribute.width(20),
              attribute.height(18),
            ]),
            html.text(f.title),
          ],
        )
      })
    ]),
    made_with_line("made-with"),
  ])
}

// Each project's own site icon, loaded from where the project publishes it and never copied
// here: the names and icons are their owners' marks, not licensed with this repository. The
// pattern and the two URLs are h0ffmann/closed-source's.
const gleam_icon =
  "https://raw.githubusercontent.com/gleam-lang/branding/01ae432a99a83216c23c34dfe666ea4c4e651582/lucy/lucy-circle.svg"

const claude_code_icon =
  "https://code.claude.com/docs/_mintlify/favicons/claude-code/pLsy-mRpNksna2sx/_generated/favicon/favicon-32x32.png"

const just_icon = "https://just.systems/favicon.ico"

// The official GNU Bash logo, at the commit of odb/official-bash-logo it was read from.
const shell_icon =
  "https://raw.githubusercontent.com/odb/official-bash-logo/e44dab9f89aadd410ff04825b2692eab16711211/assets/Logos/Icons/PNG/64x64.png"

// JavaScript is an Ecma TC39 standard and HTML a WHATWG one: each body's mark. CSS uses the community
// CSS logo (CSS-Next/logo.css, CC0), pinned: w3.org's favicon did not always load.
const css_icon =
  "https://raw.githubusercontent.com/CSS-Next/logo.css/48f24dccd4e169118d17bab998c3d276e95167df/css.square.small.svg"

const javascript_icon = "https://tc39.es/favicon.ico"

const html_icon = "https://resources.whatwg.org/logo.svg"

const rust_icon = "https://www.rust-lang.org/static/images/favicon.svg"

const nix_icon = "https://nixos.org/favicon.svg"

const opencode_icon = "https://opencode.ai/favicon.svg"

// The site serves its icon under a content hash: a new release may move it.
const opencodereview_icon = "https://open-codereview.ai/dbd8996c7419925db8e3.svg"

const ollama_icon = "https://ollama.com/public/ollama.png"

const ollaya_icon = "https://ollaya.dev/favicon.svg"

// Archify's mark at the revision vendor/archify pins.
const archify_icon =
  "https://raw.githubusercontent.com/tt-a1i/archify/2ab3cae/docs/assets/archify-mark.svg"

/// A pill: the name, with its owner's icon when the owner publishes one (`""`: none).
fn made_with(name: String, icon: String, href: String) -> Element(msg) {
  let img = case icon {
    "" -> []
    _ -> [
      html.img([
        attribute.src(icon),
        attribute.alt(""),
        attribute.width(18),
        attribute.height(18),
      ]),
    ]
  }
  html.a(
    [attribute.href(href), attribute.rel("noopener")],
    list.append(img, [html.text(" " <> name)]),
  )
}

/// The code graphs (built by `just site` with picasso's code-graph), beside the Archify pills, with
/// their own icon: they are not Archify pages. This repository's, then picasso's at the pinned
/// forks/picasso, which this one's external nodes link into.
fn pdf_pill(href: String, title: String, name: String) -> Element(msg) {
  html.a(
    [
      attribute.class("ghost figure-pill"),
      attribute.href(href),
      attribute.title(title),
    ],
    [
      html.img([
        attribute.src("pdf.svg"),
        attribute.alt("PDF"),
        attribute.width(20),
        attribute.height(20),
      ]),
      html.text(name),
    ],
  )
}

fn grafo_pill(href: String, title: String, name: String) -> Element(msg) {
  html.a(
    [
      attribute.class("ghost figure-pill"),
      attribute.href(href),
      attribute.title(title),
    ],
    [
      html.img([
        attribute.src("grafo.svg"),
        attribute.alt(""),
        attribute.width(20),
        attribute.height(20),
      ]),
      html.text(name),
    ],
  )
}

/// "feito com" the languages of this repository, each with its owner's mark, then Rust, planned, then
/// "com" the agents and the local model servers it runs on. On the cover and again in the footer.
pub fn made_with_line(class: String) -> Element(msg) {
  html.p([attribute.class(class)], [
    html.text("feito com "),
    made_with("Gleam", gleam_icon, "https://gleam.run"),
    made_with("Just", just_icon, "https://just.systems"),
    made_with("Shell", shell_icon, "https://www.gnu.org/software/bash/"),
    made_with("Nix", nix_icon, "https://nixos.org"),
    made_with("CSS", css_icon, "https://www.w3.org/Style/CSS/"),
    made_with("JavaScript", javascript_icon, "https://tc39.es"),
    made_with("HTML", html_icon, "https://html.spec.whatwg.org"),
    html.text(" e, planejado, "),
    made_with("Rust", rust_icon, "https://www.rust-lang.org"),
    html.text("; com "),
    made_with("Claude Code", claude_code_icon, "https://claude.com/claude-code"),
    made_with("OpenCode", opencode_icon, "https://opencode.ai"),
    made_with(
      "OpenCodeReview",
      opencodereview_icon,
      "https://github.com/alibaba/open-code-review",
    ),
    made_with("Ollama", ollama_icon, "https://ollama.com"),
    html.text(" e "),
    made_with("Ollaya", ollaya_icon, "https://github.com/ollaya-dev/ollaya"),
  ])
}

/// The install command first, with a copy button: one copy, one paste in a terminal. The button
/// is hidden until page.gleam's script shows it, so without JavaScript the command is still there
/// to select.
fn install() -> Element(msg) {
  html.section([attribute.id("instalar"), attribute.class("install")], [
    html.p([attribute.class("install-title")], [
      html.text("instalar o plugin do Claude Code, v" <> content.version),
    ]),
    html.div([attribute.class("install-box")], [
      html.pre([], [html.code([], [html.text(content.install)])]),
      html.button(
        [
          attribute.class("copy"),
          attribute.attribute("data-copy", content.install),
          attribute.attribute("hidden", ""),
          attribute.type_("button"),
        ],
        [html.text("copiar")],
      ),
    ]),
    html.p([attribute.class("install-note")], [
      html.text(
        "cole num terminal; depois, numa sessão nova do Claude Code, escreva ",
      ),
      html.code([], [html.text("ptbr: reescreva este texto")]),
      html.text(". para fixar a versão num repositório, veja o "),
      html.a([attribute.href(blob <> "README.md#instalar")], [
        html.text("README"),
      ]),
      html.text("."),
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
    "sem nível pedido, a skill escolhe um e diz qual assumiu. um nível planejado ainda não está na skill, e não custa contexto a quem a instala.",
    [
      html.dl(
        [attribute.class("levels")],
        list.flatten([
          list.flat_map(content.levels, level),
          list.flat_map(content.planned, planned),
        ]),
      ),
      html.p([attribute.class("intro")], [
        html.text("talvez venha um orwell-writer, como a skill "),
        html.a([attribute.href(orwell)], [html.text("orwell-writing")]),
        html.text(
          ", mas feito para os grandes escritores da língua portuguesa: um nível por autor, para imitar estilos idiossincráticos de escrita. o estilo de saramago já tem um linter de prosa, em ",
        ),
        html.a([attribute.href(repo <> "/tree/main/lint/saramago")], [
          html.code([], [html.text("lint/saramago")]),
        ]),
        html.text("."),
      ]),
      html.p([attribute.class("warn")], [
        html.text(
          "⚠ aviso: um nível de autor imita um estilo, nunca copia texto, e respeita os direitos do autor. obra em domínio público, de quem morreu há muito tempo, pode servir de base, e o autor é citado sempre que o texto for usado. obra protegida serve só de modelo de estilo. as regras estão no ",
        ),
        html.a([attribute.href(blob <> "REUSE.md")], [html.text("REUSE.md")]),
        html.text("."),
      ]),
    ],
  )
}

fn level(l: Level) -> List(Element(msg)) {
  [
    html.dt([], [
      html.span([attribute.class("chip")], [html.text(l.name)]),
      html.span([attribute.class("state done")], [html.text("IMPLEMENTADO")]),
    ]),
    html.dd([], inline(l.use_)),
  ]
}

fn planned(p: Planned) -> List(Element(msg)) {
  [
    html.dt([], [
      html.span([attribute.class("chip todo")], [html.text(p.name)]),
      html.span([attribute.class("state todo")], [html.text("PLANEJADO")]),
    ]),
    html.dd([], inline(p.what)),
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

/// The smart-note level's policy, read from SKILL.md's `## Smart note`; absent there, absent here.
fn smart_note() -> Element(msg) {
  case content.smart_note {
    [] -> element.none()
    steps ->
      section(
        "smart-note",
        "smart-note",
        "para quem mantém um segundo cérebro. a ideia vem de how to take smart notes, do zettelkasten e do fichário analógico de pirsig: o texto sai cru, e também como conhecimento indexável, com tags e links.",
        [
          html.p([], inline(content.smart_note_intro)),
          html.ol(
            [attribute.class("steps")],
            list.map(steps, fn(s) { html.li([], inline(s)) }),
          ),
        ],
      )
  }
}

fn limits() -> Element(msg) {
  section(
    "limites",
    "limites",
    "o que a skill não faz, em nenhum nível.",
    optional("limits", content.limits),
  )
}

/// docs/eli5/: PDFs the plugin generated inside this repository, each with its prompt and its tokens
/// (docs/eli5/RESULTS.tsv); the PDFs are copied to eli5/ by `just site`.
fn eli5() -> Element(msg) {
  section(
    "eli5",
    "ELI5",
    "pedidos reais, gerados pelo próprio plugin: o prompt, o PDF que ele escreveu e os tokens que gastou.",
    [
      html.table([attribute.class("eli5")], [
        html.thead([], [
          html.tr([], [
            html.th([], [html.text("prompt")]),
            html.th([], [html.text("PDF")]),
            html.th([], [html.text("tokens")]),
            html.th([], [html.text("nível")]),
          ]),
        ]),
        html.tbody(
          [],
          list.map(content.eli5, fn(e) {
            html.tr([], [
              html.td([], [
                html.a([attribute.href(blob <> "docs/eli5/" <> e.prompt)], [
                  html.text(e.title),
                ]),
              ]),
              html.td([], [
                html.a([attribute.href("eli5/" <> e.pdf)], [html.text("PDF")]),
              ]),
              html.td([attribute.class("num")], [
                html.text(e.total <> " (saída " <> e.output <> ")"),
              ]),
              html.td([], [
                html.span([attribute.class("chip")], [html.text(e.level)]),
              ]),
            ])
          }),
        ),
      ]),
    ],
  )
}

/// The drawings of docs/archify/, each a page of its own (figuras/<slug>.html, built by `just site`)
/// with the picture the README shows.
fn figuras() -> Element(msg) {
  section(
    "figuras",
    "figuras",
    "os desenhos do README, como páginas que se abrem e se leem sozinhas. cada uma sai de um arquivo em docs/archify.",
    [
      html.ul(
        [attribute.class("figures")],
        list.map(content.figures, fn(f) {
          html.li([], [
            html.a([attribute.href("figuras/" <> f.slug <> ".html")], [
              html.img([
                attribute.src("figuras/" <> f.slug <> ".png"),
                attribute.alt(f.title),
                attribute.width(560),
              ]),
              html.text(f.title),
            ]),
          ])
        }),
      ),
      html.p([], [
        html.text(
          "e o grafo do código deste repositório, gerado a cada build: ",
        ),
        html.a([attribute.href("grafo/graph.html")], [
          html.text("grafo do código"),
        ]),
        html.text(
          " (busca, zoom e comunidades; sai do code-graph do h0ffmann/picasso). os nós que usam o picasso levam ao ",
        ),
        html.a([attribute.href("grafo/picasso.html")], [
          html.text("grafo do picasso"),
        ]),
        html.text(", no commit que este repositório fixa."),
      ]),
    ],
  )
}

fn awesome() -> Element(msg) {
  section(
    "awesome",
    "awesome-ptbr-writer",
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
      html.text(" · MPL-2.0 · "),
      html.a([attribute.href(repo)], [html.text("h0ffmann/ptbr-writer")]),
    ]),
    made_with_line("made-with"),
    html.p([], [
      html.text(
        "os nomes e os ícones das linguagens e das ferramentas acima são marcas dos seus donos, não licenciadas aqui; os ícones carregam dos sites deles. a ",
      ),
      html.a([attribute.href(commons <> "Flag_of_Brazil.svg")], [
        html.text("bandeira do Brasil"),
      ]),
      html.text(" e a "),
      html.a([attribute.href(commons <> "Assinatura_de_Machado_de_Assis.png")], [
        html.text("assinatura de Machado de Assis (1839–1908)"),
      ]),
      html.text(" vêm do Wikimedia Commons, em domínio público."),
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
