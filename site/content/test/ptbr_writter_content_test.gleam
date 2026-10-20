import gleam/list
import gleeunit
import ptbr_writter_content.{
  type Origin, Level, Link, Planned, Recipe, Rule, Step,
} as c

pub fn main() -> Nil {
  gleeunit.main()
}

const skill =
  "# ptbr-writter

Reescreva texto técnico
em duas linhas.

## Uso

```
/ptbr-writter
```

- \"tire a ambiguidade deste texto\"

## Níveis de rigor

- `estrito`: runbooks.
- `leve`: comunicados.

## Regras (em desenvolvimento)

Texto solto antes da primeira regra.

### PTW-1: Quem faz, aparece

Sujeito explícito.
Sem `-se`.

### PTW-2: Uma proposição por frase

Corte gerúndios.

## Formato de saída

1. Nível assumido.
2. Texto final.
"

pub fn title_and_lede_test() {
  assert c.title(skill) == "ptbr-writter"
  assert c.lede(skill) == "Reescreva texto técnico em duas linhas."
}

pub fn levels_test() {
  assert c.levels(skill)
    == [Level("estrito", "runbooks."), Level("leve", "comunicados.")]
}

// A rule's body joins its lines; text before the first rule belongs to none.
pub fn rules_test() {
  assert c.rules(skill)
    == [
      Rule("PTW-1", "Quem faz, aparece", "Sujeito explícito. Sem `-se`."),
      Rule("PTW-2", "Uma proposição por frase", "Corte gerúndios."),
    ]
}

pub fn output_test() {
  assert c.output(skill) == ["Nível assumido.", "Texto final."]
}

pub fn usage_test() {
  assert c.usage(skill) == ["/ptbr-writter", "tire a ambiguidade deste texto"]
}

pub fn recipes_test() {
  let justfile =
    "set shell := [\"bash\"]

# lists
default:
    @just --list

# Run it.
# More about it.
opencode *args:
    opencode

no-doc:
    true

# Serve it.
site-serve: site
    true
"
  assert c.recipes(justfile)
    == [Recipe("opencode", "Run it."), Recipe("site-serve", "Serve it.")]
}

pub fn recipe_name_test() {
  assert c.recipe_name("site-serve: site") == Ok("site-serve")
  assert c.recipe_name("    echo: no") == Error(Nil)
  assert c.recipe_name("x := 1") == Error(Nil)
}

pub fn lit_escapes_test() {
  assert c.lit("a \"b\" \\ c") == "\"a \\\"b\\\" \\\\ c\""
}

// A SKILL.md that lost a section stops the build.
pub fn errors_test() {
  assert c.errors(skill, "# Run it.\nopencode:\n") == []
  assert c.errors("# t\n\nlede\n", "") != []
}

pub fn paragraph_test() {
  assert c.paragraph(skill, "Níveis")
    == "- `estrito`: runbooks. - `leve`: comunicados."
  assert c.paragraph(skill, "Limites") == ""
}

const awesome =
  "name\turl\twhat\tsource
a/b\thttps://github.com/a/b\tum linter\thttps://github.com/x/awesome
"

const roadmap =
  "item\tstatus\twhat\tevidence
evals\tem andamento\tcasos\tevals/x.md
plugin\tplanejado\tum comando\t
"

// The evidence cell may be empty: a planned step has no file yet.
pub fn tables_test() {
  assert c.links(awesome)
    == [
      Link(
        "a/b",
        "https://github.com/a/b",
        "um linter",
        "https://github.com/x/awesome",
      ),
    ]
  assert c.steps(roadmap)
    == [
      Step("evals", "em andamento", "casos", "evals/x.md"),
      Step("plugin", "planejado", "um comando", ""),
    ]
  assert c.data_errors(awesome, roadmap) == []
}

pub fn data_errors_test() {
  assert c.data_errors("h\n", roadmap) == ["awesome.tsv: no row"]
  assert c.data_errors(
      "h\na/b\thttps://github.com/z/q\tw\tlist\n",
      "h\nx\tquase\tw\t\n",
    )
    == [
      "awesome.tsv: a/b is not at https://github.com/z/q",
      "awesome.tsv: a/b has no source list",
      "roadmap.tsv: x: status is \"quase\"",
    ]
}

const origin_rows =
  "id\torigin\trepo\tcreator
PTW-1\tPTC-1\tk/ptc\tkayquer
PTW-2\tPTC-2\tk/ptc\tkayquer
PTW-2\tregra\th/w\th0ffmann
"

// A rule may have more than one origin; every rule needs one.
pub fn origins_test() {
  assert c.origins(origin_rows)
    |> list.map(fn(o: Origin) { o.id })
    == ["PTW-1", "PTW-2", "PTW-2"]
  assert c.origin_errors(skill, origin_rows) == []
}

// A creator is a GitHub login, never a full name.
pub fn origin_errors_test() {
  assert c.origin_errors(
      skill,
      "h\nPTW-1\tPTC-1\tk/ptc\tKayque R\nPTW-9\tx\tk/ptc\tkayquer\n",
    )
    == [
      "origins.tsv: PTW-2 has no origin",
      "origins.tsv: \"Kayque R\" is not a GitHub login",
      "origins.tsv: SKILL.md has no rule PTW-9",
    ]
  assert c.login("h0ffmann")
  assert !c.login("@kayquer")
}

// A planned level is not in SKILL.md; once it is, its row must go.
pub fn planned_test() {
  let rows = "name\twhat\ncrônico\tcrônica\n"
  assert c.planned(rows) == [Planned("crônico", "crônica")]
  assert c.planned_errors(skill, rows) == []
  assert c.planned_errors(skill, "name\twhat\nleve\tx\n")
    == ["planned.tsv: leve is in SKILL.md now; delete its row"]
  assert c.planned_errors(skill, "name\twhat\n") == ["planned.tsv: no row"]
}

// A section with a paragraph and a numbered list: the smart-note policy's shape.
pub fn numbered_and_preamble_test() {
  let text =
    "## Smart note\n\nCombina com outro nível.\n\n1. Uma ideia, uma nota.\n2. Palavras próprias.\n\n## Fim\n"
  assert c.numbered(text, "Smart note")
    == ["Uma ideia, uma nota.", "Palavras próprias."]
  assert c.preamble(text, "Smart note") == "Combina com outro nível."
  assert c.numbered(text, "Ausente") == []
}
