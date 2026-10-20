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

// awesome-ptbr-writter opens with the original PTC, the source of this fork.
pub fn awesome_starts_with_ptc_test() {
  let assert [first, ..] = content.awesome
  assert first.name == "kayquer/portugues-tecnico-controlado"
}

// Implemented levels come from SKILL.md, planned ones from planned.tsv; each says which it is,
// and the authors' rights warning points to REUSE.md.
pub fn level_states_test() {
  let page = prerender.render()
  list.each(content.planned, fn(p) {
    assert string.contains(page, p.name)
  })
  assert string.contains(page, "IMPLEMENTADO")
  assert string.contains(page, "PLANEJADO")
  assert string.contains(page, "Machado de Assis")
  assert string.contains(page, "José Saramago")
  assert string.contains(page, "orwell-writer")
  assert string.contains(
    page,
    "builder-essential-skills/tree/1be9984/skills/orwell-writing",
  )
  assert string.contains(page, "lint/saramago")
  assert string.contains(page, "blob/main/REUSE.md")
}

// Made with Gleam and Claude Code, each with its owner's icon, as on h0ffmann/closed-source.
pub fn made_with_test() {
  let page = prerender.render()
  assert string.contains(page, "gleam-lang/branding")
  assert string.contains(page, "favicons/claude-code")
  assert string.contains(page, "https://gleam.run")
  assert string.contains(page, "https://claude.com/claude-code")
}

// pt-BR first: the flag and the calligraphy are on the cover, and both are credited, Machado de
// Assis by name (REUSE.md).
pub fn cover_images_test() {
  let page = prerender.render()
  assert string.contains(page, "img/bandeira-do-brasil.svg")
  assert string.contains(page, "pt-BR first")
  assert string.contains(page, "img/assinatura-machado-de-assis.png")
  assert string.contains(page, "File:Flag_of_Brazil.svg")
  assert string.contains(page, "File:Assinatura_de_Machado_de_Assis.png")
  assert string.contains(page, "assinatura de Machado de Assis (1839–1908)")
}

// smart-note is a level of SKILL.md, and its policy has its own section.
pub fn smart_note_test() {
  let page = prerender.render()
  assert string.contains(page, "id=\"smart-note\"")
  assert content.smart_note != []
  list.each(content.levels, fn(l) {
    assert string.contains(page, l.name)
  })
  assert list.any(content.levels, fn(l) { l.name == "smart-note" })
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
