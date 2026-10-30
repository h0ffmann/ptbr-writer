//// The HTML document around the page.

import lustre/attribute
import lustre/element.{type Element}
import lustre/element/html
import gramatico_site/content

pub fn document(body: Element(msg)) -> Element(msg) {
  titled(content.title, content.lede, body)
}

/// The document of a page with its own title and description.
pub fn titled(
  title: String,
  description: String,
  body: Element(msg),
) -> Element(msg) {
  let meta = fn(name, value) {
    html.meta([attribute.name(name), attribute.content(value)])
  }
  let stylesheet = fn(href) {
    html.link([attribute.rel("stylesheet"), attribute.href(href)])
  }
  html.html([attribute.lang("pt-BR")], [
    html.head([], [
      html.meta([attribute.charset("utf-8")]),
      meta("viewport", "width=device-width, initial-scale=1"),
      html.title([], title),
      meta("description", description),
      html.link([
        attribute.rel("icon"),
        attribute.href("favicon.svg"),
        attribute.type_("image/svg+xml"),
      ]),
      html.link([
        attribute.rel("preconnect"),
        attribute.href("https://fonts.googleapis.com"),
      ]),
      html.link([
        attribute.rel("preconnect"),
        attribute.href("https://fonts.gstatic.com"),
        attribute.crossorigin(""),
      ]),
      stylesheet(
        "https://fonts.googleapis.com/css2?family=Fraunces:opsz,wght@9..144,600;9..144,800&family=Source+Serif+4:ital,wght@0,400;0,600;1,400&family=JetBrains+Mono&display=swap",
      ),
      stylesheet("style.css"),
    ]),
    html.body([], [body, html.script([], copy_script)]),
  ])
}

/// The page's only script: show each copy button and put its command on the clipboard.
const copy_script =
  "document.querySelectorAll('[data-copy]').forEach(function (b) {
  b.hidden = false;
  b.addEventListener('click', function () {
    navigator.clipboard.writeText(b.dataset.copy).then(function () {
      b.textContent = 'copiado ✓';
      setTimeout(function () { b.textContent = 'copiar'; }, 1500);
    });
  });
});"
