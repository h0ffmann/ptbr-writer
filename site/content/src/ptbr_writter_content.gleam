//// `gleam run` in site/content: reads SKILL.md, the justfile and site/data/*.tsv and writes
//// ../src/ptbr_writter_site/content.gleam, the only place the site takes its facts from. Nothing in
//// site/src is typed by hand: a rule, a level, a step or a recipe that the page shows is read here.
//// The pattern is h0ffmann/ww3-gpu's `site/content`.

import gleam/io
import gleam/list
import gleam/string

/// The repository's root, seen from site/content.
const root = "../../"

/// A rule of SKILL.md: `### PTW-1: Quem faz, aparece` and the paragraph under it.
pub type Rule {
  Rule(id: String, title: String, body: String)
}

/// A level of rigor: `- \`estrito\`: runbooks, …`.
pub type Level {
  Level(name: String, use_: String)
}

/// A recipe of the justfile and the comment line above it.
pub type Recipe {
  Recipe(name: String, doc: String)
}

/// A row of site/data/awesome.tsv: a related repository and the curated list it was found in.
pub type Link {
  Link(name: String, url: String, what: String, source: String)
}

/// A row of site/data/roadmap.tsv: a piece of production readiness, its status and its file.
pub type Step {
  Step(item: String, status: String, what: String, evidence: String)
}

/// A row of site/data/origins.tsv: where a rule comes from, and who made that.
pub type Origin {
  Origin(
    id: String,
    origin: String,
    repo: String,
    creator: String,
    profile: String,
  )
}

/// The statuses a roadmap row may have, in the order of the work.
pub const statuses = ["feito", "em andamento", "planejado"]

@external(javascript, "./ptbr_writter_content_ffi.mjs", "read")
fn read(path: String) -> String

@external(javascript, "./ptbr_writter_content_ffi.mjs", "write")
fn write(path: String, text: String) -> Nil

@external(javascript, "./ptbr_writter_content_ffi.mjs", "revision")
fn revision() -> String

@external(javascript, "./ptbr_writter_content_ffi.mjs", "exit")
fn exit(code: Int) -> Nil

// ---- reading SKILL.md --------------------------------------------------------------------------

/// The lines of the `## ` section whose heading starts with `name`, up to the next `## `.
pub fn section(text: String, name: String) -> List(String) {
  text
  |> string.split("\n")
  |> list.drop_while(fn(line) { !string.starts_with(line, "## " <> name) })
  |> list.drop(1)
  |> list.take_while(fn(line) { !string.starts_with(line, "## ") })
}

/// The text of the first `# ` heading.
pub fn title(text: String) -> String {
  text
  |> string.split("\n")
  |> list.find(fn(line) { string.starts_with(line, "# ") })
  |> fn(found) {
    case found {
      Ok(line) -> string.trim(string.drop_start(line, 2))
      Error(_) -> ""
    }
  }
}

/// The text of a `## ` section as one paragraph, or "" when SKILL.md has no such section.
pub fn paragraph(text: String, name: String) -> String {
  section(text, name)
  |> list.map(string.trim)
  |> list.filter(fn(line) { line != "" })
  |> string.join(" ")
}

/// The first paragraph after the `# ` heading.
pub fn lede(text: String) -> String {
  text
  |> string.split("\n")
  |> list.drop_while(fn(line) { !string.starts_with(line, "# ") })
  |> list.drop(1)
  |> list.drop_while(fn(line) { string.trim(line) == "" })
  |> list.take_while(fn(line) { string.trim(line) != "" })
  |> string.join(" ")
}

pub fn levels(text: String) -> List(Level) {
  section(text, "Níveis")
  |> list.filter_map(fn(line) {
    case string.split_once(line, "`: ") {
      Ok(#("- `" <> name, use_)) -> Ok(Level(name, use_))
      _ -> Error(Nil)
    }
  })
}

pub fn rules(text: String) -> List(Rule) {
  let #(found, last) =
    section(text, "Regras")
    |> list.fold(#([], Error(Nil)), fn(acc, line) {
      let #(found, current) = acc
      case string.starts_with(line, "### "), current {
        True, _ ->
          case string.split_once(string.drop_start(line, 4), ": ") {
            Ok(#(id, title)) -> #(push(found, current), Ok(Rule(id, title, "")))
            Error(_) -> #(push(found, current), Error(Nil))
          }
        False, Ok(rule) ->
          case string.trim(line) {
            "" -> acc
            words -> #(found, Ok(Rule(..rule, body: join(rule.body, words))))
          }
        False, Error(_) -> acc
      }
    })
  list.reverse(push(found, last))
}

fn push(found: List(Rule), current: Result(Rule, Nil)) -> List(Rule) {
  case current {
    Ok(rule) -> [rule, ..found]
    Error(_) -> found
  }
}

fn join(a: String, b: String) -> String {
  case a {
    "" -> b
    _ -> a <> " " <> b
  }
}

/// The steps of `## Formato de saída`: `1. Nível e destinatário assumidos.`
pub fn output(text: String) -> List(String) {
  section(text, "Formato de saída")
  |> list.filter_map(fn(line) {
    case string.split_once(line, ". ") {
      Ok(#(n, step)) if n != "" ->
        case string.to_graphemes(n) |> list.all(is_digit) {
          True -> Ok(step)
          False -> Error(Nil)
        }
      _ -> Error(Nil)
    }
  })
}

fn is_digit(g: String) -> Bool {
  string.contains("0123456789", g)
}

/// The ways to call the skill in `## Uso`: the lines of its code block, then the quoted examples.
pub fn usage(text: String) -> List(String) {
  let lines = section(text, "Uso")
  let code =
    lines
    |> list.drop_while(fn(l) { !string.starts_with(l, "```") })
    |> list.drop(1)
    |> list.take_while(fn(l) { !string.starts_with(l, "```") })
    |> list.filter(fn(l) { string.trim(l) != "" })
  let quoted =
    list.filter_map(lines, fn(l) {
      case string.starts_with(l, "- \""), string.ends_with(l, "\"") {
        True, True -> Ok(l |> string.drop_start(3) |> string.drop_end(1))
        _, _ -> Error(Nil)
      }
    })
  list.append(code, quoted)
}

// ---- reading the justfile ----------------------------------------------------------------------

/// Each recipe that has a comment block right above it, with the block's first line; `default`
/// only lists the others.
pub fn recipes(justfile: String) -> List(Recipe) {
  let #(found, _) =
    justfile
    |> string.split("\n")
    |> list.fold(#([], Error(Nil)), fn(acc, line) {
      let #(found, block) = acc
      case string.starts_with(line, "# "), block, recipe_name(line) {
        True, Ok(_), _ -> acc
        True, Error(_), _ -> #(found, Ok(string.drop_start(line, 2)))
        False, Ok(doc), Ok(name) if name != "default" -> #(
          [Recipe(name, doc), ..found],
          Error(Nil),
        )
        False, _, _ -> #(found, Error(Nil))
      }
    })
  list.reverse(found)
}

/// "site-serve" from "site-serve: site", "opencode" from "opencode *args:".
pub fn recipe_name(line: String) -> Result(String, Nil) {
  let first = string.slice(line, 0, 1)
  case
    first == "" || first == " " || first == "\t" || first == "#",
    string.starts_with(line, "set "),
    string.contains(line, ":="),
    string.split_once(line, ":")
  {
    False, False, False, Ok(#(head, _)) ->
      case string.split(head, " ") {
        [name, ..] -> Ok(name)
        [] -> Error(Nil)
      }
    _, _, _, _ -> Error(Nil)
  }
}

// ---- reading site/data ------------------------------------------------------------------------

/// The rows of a TSV file, its header dropped; a blank line is no row.
pub fn tsv(text: String) -> List(List(String)) {
  text
  |> string.split("\n")
  |> list.drop(1)
  |> list.filter(fn(line) { string.trim(line) != "" })
  |> list.map(fn(line) { string.split(line, "\t") })
}

pub fn links(text: String) -> List(Link) {
  tsv(text)
  |> list.filter_map(fn(cells) {
    case cells {
      [name, url, what, source] -> Ok(Link(name, url, what, source))
      _ -> Error(Nil)
    }
  })
}

pub fn steps(text: String) -> List(Step) {
  tsv(text)
  |> list.filter_map(fn(cells) {
    case cells {
      [item, status, what, evidence] -> Ok(Step(item, status, what, evidence))
      _ -> Error(Nil)
    }
  })
}

pub fn origins(text: String) -> List(Origin) {
  tsv(text)
  |> list.filter_map(fn(cells) {
    case cells {
      [id, origin, repo, creator, profile] ->
        Ok(Origin(id, origin, repo, creator, profile))
      _ -> Error(Nil)
    }
  })
}

/// What is wrong with origins.tsv: a row that is not five cells, a rule of SKILL.md with no
/// origin, a row for a rule SKILL.md does not have, or a creator with no GitHub profile.
pub fn origin_errors(skill: String, text: String) -> List(String) {
  let ids = list.map(rules(skill), fn(r) { r.id })
  let found = origins(text)
  list.flatten([
    list.filter_map(tsv(text), fn(cells) {
      case list.length(cells) {
        5 -> Error(Nil)
        _ ->
          Ok("origins.tsv: a row needs 5 cells: " <> string.join(cells, " | "))
      }
    }),
    list.filter_map(ids, fn(id) {
      case list.any(found, fn(o) { o.id == id }) {
        True -> Error(Nil)
        False -> Ok("origins.tsv: " <> id <> " has no origin")
      }
    }),
    list.flat_map(found, fn(o) {
      list.flatten([
        case list.contains(ids, o.id) {
          True -> []
          False -> ["origins.tsv: SKILL.md has no rule " <> o.id]
        },
        case string.starts_with(o.profile, "https://github.com/") {
          True -> []
          False -> ["origins.tsv: " <> o.creator <> " has no GitHub profile"]
        },
      ])
    }),
  ])
}

/// What is wrong with the two tables: a row that is not four cells, a link that is not a GitHub
/// repository, a source that is not a URL, a status outside `statuses`, or a table with no row.
pub fn data_errors(awesome: String, roadmap: String) -> List(String) {
  let shape = fn(file, text) {
    case tsv(text) {
      [] -> [file <> ": no row"]
      rows ->
        list.filter_map(rows, fn(cells) {
          case list.length(cells) {
            4 -> Error(Nil)
            _ ->
              Ok(file <> ": a row needs 4 cells: " <> string.join(cells, " | "))
          }
        })
    }
  }
  list.flatten([
    shape("awesome.tsv", awesome),
    shape("roadmap.tsv", roadmap),
    list.flat_map(links(awesome), fn(l) {
      list.flatten([
        case string.starts_with(l.url, "https://github.com/" <> l.name) {
          True -> []
          False -> ["awesome.tsv: " <> l.name <> " is not at " <> l.url]
        },
        case string.starts_with(l.source, "https://") {
          True -> []
          False -> ["awesome.tsv: " <> l.name <> " has no source list"]
        },
      ])
    }),
    list.filter_map(steps(roadmap), fn(s) {
      case list.contains(statuses, s.status) {
        True -> Error(Nil)
        False ->
          Ok("roadmap.tsv: " <> s.item <> ": status is " <> lit(s.status))
      }
    }),
  ])
}

// ---- rendering ---------------------------------------------------------------------------------

/// A Gleam string literal.
pub fn lit(s: String) -> String {
  "\""
  <> s
  |> string.replace("\\", "\\\\")
  |> string.replace("\"", "\\\"")
  |> string.replace("\n", "\\n")
  <> "\""
}

fn call(name: String, args: List(String)) -> String {
  name <> "(" <> string.join(list.map(args, lit), ", ") <> ")"
}

fn items(name: String, type_: String, rows: List(String)) -> String {
  let body = case rows {
    [] -> "[]"
    _ -> "[\n  " <> string.join(rows, ",\n  ") <> ",\n]"
  }
  "pub const " <> name <> ": List(" <> type_ <> ") = " <> body <> "\n"
}

const types =
  "pub type Rule {
  Rule(id: String, title: String, body: String)
}

pub type Level {
  Level(name: String, use_: String)
}

pub type Recipe {
  Recipe(name: String, doc: String)
}

pub type Link {
  Link(name: String, url: String, what: String, source: String)
}

pub type Step {
  Step(item: String, status: String, what: String, evidence: String)
}

pub type Origin {
  Origin(id: String, origin: String, repo: String, creator: String, profile: String)
}
"

/// The content module, from the text of SKILL.md and of the justfile.
pub fn module(
  skill: String,
  justfile: String,
  awesome: String,
  roadmap: String,
  origin_rows: String,
  rev: String,
) -> String {
  string.join(
    [
      "//// Written by site/content from SKILL.md and the justfile. Do not edit: `just site`.\n",
      types,
      "pub const title = " <> lit(title(skill)) <> "\n",
      "pub const lede = " <> lit(lede(skill)) <> "\n",
      "pub const revision = " <> lit(rev) <> "\n",
      "pub const limits = " <> lit(paragraph(skill, "Limites")) <> "\n",
      "pub const origin = " <> lit(paragraph(skill, "Origem")) <> "\n",
      items(
        "levels",
        "Level",
        list.map(levels(skill), fn(l) { call("Level", [l.name, l.use_]) }),
      ),
      items(
        "rules",
        "Rule",
        list.map(rules(skill), fn(r) { call("Rule", [r.id, r.title, r.body]) }),
      ),
      items("output", "String", list.map(output(skill), lit)),
      items("usage", "String", list.map(usage(skill), lit)),
      items(
        "recipes",
        "Recipe",
        list.map(recipes(justfile), fn(r) { call("Recipe", [r.name, r.doc]) }),
      ),
      items(
        "awesome",
        "Link",
        list.map(links(awesome), fn(l) {
          call("Link", [l.name, l.url, l.what, l.source])
        }),
      ),
      items(
        "origins",
        "Origin",
        list.map(origins(origin_rows), fn(o) {
          call("Origin", [o.id, o.origin, o.repo, o.creator, o.profile])
        }),
      ),
      items(
        "roadmap",
        "Step",
        list.map(steps(roadmap), fn(s) {
          call("Step", [s.item, s.status, s.what, s.evidence])
        }),
      ),
    ],
    "\n",
  )
}

/// What the page cannot do without: an empty list means SKILL.md changed shape.
pub fn errors(skill: String, justfile: String) -> List(String) {
  [
    #(title(skill) == "", "SKILL.md has no `# ` title"),
    #(lede(skill) == "", "SKILL.md has no paragraph under its title"),
    #(levels(skill) == [], "SKILL.md: no `- `level`: use` under `## Níveis`"),
    #(rules(skill) == [], "SKILL.md: no `### ID: title` under `## Regras`"),
    #(
      output(skill) == [],
      "SKILL.md: no numbered step in `## Formato de saída`",
    ),
    #(usage(skill) == [], "SKILL.md: no example in `## Uso`"),
    #(recipes(justfile) == [], "justfile: no recipe with a comment above it"),
  ]
  |> list.filter_map(fn(check) {
    case check.0 {
      True -> Ok(check.1)
      False -> Error(Nil)
    }
  })
}

pub fn main() -> Nil {
  let skill = read(root <> "SKILL.md")
  let justfile = read(root <> "justfile")
  let awesome = read("../data/awesome.tsv")
  let roadmap = read("../data/roadmap.tsv")
  let origin_rows = read("../data/origins.tsv")
  case
    list.flatten([
      errors(skill, justfile),
      data_errors(awesome, roadmap),
      origin_errors(skill, origin_rows),
    ])
  {
    [] ->
      write(
        "../src/ptbr_writter_site/content.gleam",
        module(skill, justfile, awesome, roadmap, origin_rows, revision()),
      )
    found -> {
      list.each(found, io.println_error)
      exit(1)
    }
  }
}
