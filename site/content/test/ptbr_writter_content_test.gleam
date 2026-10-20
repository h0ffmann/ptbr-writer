import gleeunit
import ptbr_writter_content.{Level, Recipe, Rule} as c

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
