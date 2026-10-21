import gleam/list
import gleam/string
import gleeunit
import lustre/element
import lustre/element/html
import ptbr_writer_site/app
import ptbr_writer_site/content
import ptbr_writer_site/prerender

pub fn main() -> Nil {
  gleeunit.main()
}

pub fn every_section_test() {
  let page = prerender.render()
  list.each(
    [
      "instalar",
      "uso",
      "niveis",
      "regras",
      "saida",
      "limites",
      "receitas",
      "figuras",
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
  assert content.figures != []
  list.each(content.figures, fn(f) {
    assert string.contains(page, "figuras/" <> f.slug <> ".html")
    assert string.contains(page, f.title)
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

// awesome-ptbr-writer opens with the original PTC, the source of this fork.
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
    "img.shields.io/github/stars/h0ffmann/ptbr-writer",
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
  // One script only: the copy button's (page.gleam, DESIGN.md).
  assert list.length(string.split(page, "<script")) == 2
}

// Install comes first, with the one command and a copy button the script shows.
pub fn install_first_test() {
  let html = prerender.render()
  assert string.contains(html, content.install)
  assert string.contains(html, "data-copy=\"" <> content.install <> "\"")
  let assert Ok(#(before, _)) = string.split_once(html, "id=\"instalar\"")
  assert !string.contains(before, "id=\"uso\"")
  assert string.contains(html, "navigator.clipboard.writeText")
}

// Each Archify page is a named pill in the cover's actions, after the GitHub pill.
pub fn figure_pills_test() {
  let html = prerender.render()
  let assert Ok(#(_, after_star)) = string.split_once(html, "star me no github")
  let assert Ok(#(actions, _)) = string.split_once(after_star, "</div>")
  list.each(content.figures, fn(f) {
    assert string.contains(
      actions,
      "href=\"figuras/" <> f.slug <> ".html\">◇ " <> f.title,
    )
  })
  assert content.figures != []
}
