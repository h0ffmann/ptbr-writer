//! The few DOM calls the options page and the popup make, as short helpers over web-sys: an
//! element by tag, a builder for text and children, a listener, a page task.

use std::future::Future;
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, EventTarget, HtmlButtonElement, HtmlElement, HtmlOptionElement, Node};

pub fn document() -> Document {
    web_sys::window().expect("a window").document().expect("a document")
}

/// `document.createElement(tag)`, as the element type the tag gives.
pub fn create<T: JsCast>(tag: &str) -> T {
    document().create_element(tag).expect("createElement").unchecked_into()
}

pub fn el(tag: &str) -> Element {
    create(tag)
}

/// Text and children appended in order: `el("label").text("Nível ").child(&select)`.
pub trait Build: AsRef<Node> + Sized {
    fn child(self, node: &impl AsRef<Node>) -> Self {
        self.as_ref().append_child(node.as_ref()).expect("appendChild");
        self
    }

    fn text(self, s: &str) -> Self {
        self.child(&document().create_text_node(s))
    }

    fn id(self, id: &str) -> Self
    where
        Self: AsRef<Element>,
    {
        AsRef::<Element>::as_ref(&self).set_id(id);
        self
    }
}

impl<T: AsRef<Node>> Build for T {}

/// `root.querySelector(sel)`, which the page's own markup guarantees to exist.
pub fn query<T: JsCast>(root: &Element, sel: &str) -> T {
    root.query_selector(sel).ok().flatten().unwrap_or_else(|| panic!("{sel}")).unchecked_into()
}

pub fn option(value: &str, text: &str) -> HtmlOptionElement {
    let o: HtmlOptionElement = create("option");
    o.set_value(value);
    o.set_text_content(Some(text));
    o
}

pub fn button(text: &str, on_click: impl FnMut() + 'static) -> HtmlButtonElement {
    let b: HtmlButtonElement = create("button");
    b.set_type("button");
    b.set_text_content(Some(text));
    on(&b, "click", on_click);
    b
}

/// Runs `f` on each `event`. The closure is leaked on purpose: a page's listeners live as long as
/// the page, and nothing ever removes one.
pub fn on(target: &EventTarget, event: &str, f: impl FnMut() + 'static) {
    let cb = Closure::<dyn FnMut()>::new(f);
    target.add_event_listener_with_callback(event, cb.as_ref().unchecked_ref()).expect("addEventListener");
    cb.forget();
}

/// A page task; its error goes to the console, as an unhandled rejection would.
pub fn spawn(task: impl Future<Output = Result<(), JsValue>> + 'static) {
    wasm_bindgen_futures::spawn_local(async move {
        if let Err(e) = task.await {
            web_sys::console::error_1(&e);
        }
    });
}

/// A `<p>` with `text`, appended to `parent`.
pub fn line(parent: &HtmlElement, text: &str) {
    parent.child(&el("p").text(text));
}
