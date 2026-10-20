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
  list.each(
    [
      "uso",
      "niveis",
      "regras",
      "saida",
      "limites",
      "receitas",
      "awesome",
      "roadmap",
    ],
    fn(id) {
      assert string.contains(page, "id=\"" <> id <> "\"")
    },
  )
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
  list.each(content.awesome, fn(l) {
    assert string.contains(page, l.url)
    assert string.contains(page, l.source)
  })
  list.each(content.roadmap, fn(s) {
    assert string.contains(page, s.item)
  })
}

// Every rule names where it comes from and who made it, with links.
pub fn rule_origins_test() {
  let page = prerender.render()
  list.each(content.origins, fn(o) {
    assert string.contains(page, "https://github.com/" <> o.repo)
    assert string.contains(page, "https://github.com/" <> o.creator)
    assert string.contains(page, "@" <> o.creator)
  })
  assert string.contains(page, "PTC-1, de ")
}

pub fn star_pill_test() {
  let page = prerender.render()
  assert string.contains(page, "star me")
  assert string.contains(
    page,
    "img.shields.io/github/stars/h0ffmann/ptbr-writter",
  )
}

pub fn inline_link_test() {
  let shown =
    html.p([], app.inline("de [a/b](https://x.y/a) (`041cedd`)"))
    |> element.to_string
  assert shown
    == "<p>de <a href=\"https://x.y/a\">a/b</a> (<code>041cedd</code>)</p>"
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
