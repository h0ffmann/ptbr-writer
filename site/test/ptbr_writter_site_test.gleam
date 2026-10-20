import gleam/list
import gleam/string
import gleeunit
import lustre/element
import lustre/element/html
import ptbr_writter_site/app
import ptbr_writter_site/content
import ptbr_writter_site/prerender

pub fn main() -> Nil {
  gleeunit.main()
}

pub fn every_section_test() {
  let page = prerender.render()
  list.each(["uso", "niveis", "regras", "saida", "receitas"], fn(id) {
    assert string.contains(page, "id=\"" <> id <> "\"")
  })
  assert string.contains(page, "<html lang=\"pt-BR\">")
}

// Every fact of SKILL.md and of the justfile reaches the page.
pub fn every_fact_test() {
  let page = prerender.render()
  assert content.rules != []
  list.each(content.rules, fn(r) {
    assert string.contains(page, r.id)
    assert string.contains(page, r.title)
  })
  list.each(content.levels, fn(l) {
    assert string.contains(page, l.name)
  })
  list.each(content.recipes, fn(r) {
    assert string.contains(page, "just " <> r.name)
  })
  assert content.output != []
}

pub fn inline_code_test() {
  let shown =
    html.p([], app.inline("evite `-se` e `o mesmo`"))
    |> element.to_string
  assert shown == "<p>evite <code>-se</code> e <code>o mesmo</code></p>"
}

// The fork notice is in the footer, the last thing on the page.
pub fn fork_notice_last_test() {
  let page = prerender.render()
  let assert Ok(#(_, after)) = string.split_once(page, "<footer")
  assert string.contains(after, "kayquer/portugues-tecnico-controlado")
  assert !string.contains(page, "<script")
}
