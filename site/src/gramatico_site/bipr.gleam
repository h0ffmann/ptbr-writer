//// `gleam run -m gramatico_site/bipr > dist/bipr.html`: the /bipr page. Its description, steps
//// and footer come from skills/bipr/ through `content`; the screenshots are of PR #49, the PR the
//// skill opened for itself.

import gleam/io
import gleam/list
import gramatico_site/app
import gramatico_site/content
import gramatico_site/page
import lustre/attribute
import lustre/element.{type Element}
import lustre/element/html

const pr = "https://github.com/piabrota/gramatico/pull/49"

const blob = "https://github.com/piabrota/gramatico/blob/main/"

const install_current =
  "! curl -fsSL https://raw.githubusercontent.com/piabrota/gramatico/main/scripts/install_current.sh | bash"

pub fn view() -> Element(msg) {
  html.main([], [
    html.header([attribute.class("cover wrap")], [
      html.p([attribute.class("kicker")], [
        html.a([attribute.href("index.html")], [html.text("← gramatico")]),
      ]),
      html.div([attribute.class("brand")], [
        html.img([
          attribute.src("favicon.svg"),
          attribute.alt(
            "um G grande, como o do esquadro e compasso, nas cores da bandeira do Brasil",
          ),
          attribute.class("mark"),
        ]),
        html.h1([], [html.text("/bipr")]),
      ]),
      html.p([attribute.class("lede")], app.inline(content.bipr_description)),
    ]),
    app.section(
      "passos",
      "passos",
      "o que a skill faz, na ordem do skills/bipr/SKILL.md.",
      [
        html.ol(
          [],
          list.map(content.bipr_steps, fn(s) { html.li([], app.inline(s)) }),
        ),
      ],
    ),
    app.section(
      "exemplo",
      "exemplo: o PR #49",
      "a skill abriu e atualizou o próprio PR. o corpo em pt-BR, o espelho em inglês e o rodapé, como o GitHub mostra.",
      [
        html.div([attribute.class("figures")], [
          html.a([attribute.href(pr)], [
            html.img([
              attribute.src("img/bipr-pr49-corpo.png"),
              attribute.alt(
                "o PR #49 no GitHub: Resumo, Testado e O que mudou em pt-BR; Summary, Tested e What changed em inglês; o rodapé no fim",
              ),
            ]),
          ]),
        ]),
      ],
    ),
    app.section(
      "rodape",
      "rodapé",
      "a última linha do corpo vem de skills/bipr/footer.md, copiada sem mudança, no lugar do \"Generated with\" do agente.",
      [
        html.pre([], [html.code([], [html.text(content.bipr_footer)])]),
        html.div([attribute.class("figures")], [
          html.img([
            attribute.src("img/bipr-pr49-rodape.png"),
            attribute.alt(
              "o rodapé no GitHub: o favicon do gramatico e Generated with piabrota/gramatico bipr skill",
            ),
          ]),
        ]),
      ],
    ),
    app.section(
      "custo",
      "custo em tokens",
      "o custo de cada chamada ao modelo que abriu o PR #49, lido do transcript da sessão, está na página do exemplo.",
      [
        html.p([], [
          html.a([attribute.href(blob <> "docs/bipr/exemplo-pr-49.md")], [
            html.text("docs/bipr/exemplo-pr-49.md"),
          ]),
        ]),
      ],
    ),
    app.section(
      "instalar",
      "instalar",
      "a /bipr ainda não está numa versão. para usar a main, dentro do Claude Code, e depois /reload-plugins:",
      [html.pre([], [html.code([], [html.text(install_current)])])],
    ),
    html.footer([attribute.class("foot wrap")], [
      app.made_with_line("made-with"),
    ]),
  ])
}

pub fn render() -> String {
  view()
  |> page.titled("/bipr · gramatico", content.bipr_description, _)
  |> element.to_document_string
}

pub fn main() -> Nil {
  io.println(render())
}
