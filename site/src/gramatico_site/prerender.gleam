//// `gleam run -m gramatico_site/prerender > dist/index.html`: the whole page, rendered once at
//// build time. It needs no JavaScript.

import gleam/io
import lustre/element
import gramatico_site/app
import gramatico_site/page

pub fn render() -> String {
  app.view()
  |> page.document
  |> element.to_document_string
}

pub fn main() -> Nil {
  io.println(render())
}
