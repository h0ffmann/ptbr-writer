//! Everything drawn for an editor, in one shadow root outside it: the underlines, the badge and
//! the card (Aceitar / Ignorar / Sempre ignorar). The editor's DOM is never touched. One listener
//! each on the shadow root (a click, a mousedown) serves every underline, badge and card button;
//! three on the document close the card and find the hovered word.

use crate::content::card::{card_heading, offers_never_fix};
use crate::content::text::{TextModel, field_value, is_field, range_for, text_model};
use crate::content::watcher::Status;
use crate::content::{ElMap, Timer, same};
use crate::rules::rule_titles;
use crate::skills::{CIENTIFICO, SKILL};
use crate::types::{CheckError, Located, Severity};
use crate::utf16::len16;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::rc::{Rc, Weak};
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    CssStyleDeclaration, Document, DomRect, DomRectList, Element, Event, HtmlElement, KeyboardEvent, MouseEvent,
    ShadowRoot, ShadowRootInit, ShadowRootMode, Window,
};

const HOST_ID: &str = "ptbr-writer-root";

const STYLE: &str = r#"
:host { all: initial; }
.ptbr-layer, .ptbr-mirror { position: absolute; top: 0; left: 0; }
.ptbr-underline { pointer-events: none; }
.ptbr-seg { position: absolute; box-sizing: border-box; border-bottom: 2px solid; }
.ptbr-hit { cursor: pointer; }
.ptbr-underline-erro .ptbr-seg { border-color: #d93025; }
.ptbr-underline-aviso .ptbr-seg { border-color: #e8a400; }
.ptbr-underline:hover .ptbr-seg { background: rgba(232, 164, 0, 0.15); }
.ptbr-underline-erro:hover .ptbr-seg { background: rgba(217, 48, 37, 0.12); }
.ptbr-mirror { visibility: hidden; overflow: hidden; box-sizing: border-box; border-style: solid; border-color: transparent; }
.ptbr-card, .ptbr-badge {
  pointer-events: auto;
  position: absolute;
  font: 13px/1.4 system-ui, -apple-system, "Segoe UI", sans-serif;
  color: #1f1f1f;
}
.ptbr-card {
  box-sizing: border-box;
  max-width: 320px;
  padding: 10px 12px;
  background: #fff;
  border: 1px solid #d0d0d0;
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.18);
}
.ptbr-rule { margin: 0 0 6px; font-size: 12px; color: #5f5f5f; }
.ptbr-change { margin: 0 0 8px; }
.ptbr-change s { color: #a0332b; }
.ptbr-change strong { color: #146c2e; }
.ptbr-actions { display: flex; gap: 6px; }
.ptbr-actions button {
  font: inherit;
  padding: 3px 10px;
  border-radius: 6px;
  border: 1px solid #c4c4c4;
  background: #f5f5f5;
  color: inherit;
  cursor: pointer;
}
.ptbr-actions button[data-action="accept"] { background: #146c2e; border-color: #146c2e; color: #fff; }
.ptbr-badge {
  box-sizing: border-box;
  min-width: 20px;
  height: 20px;
  padding: 0 6px;
  border-radius: 10px;
  background: #146c2e;
  color: #fff;
  font-size: 11px;
  line-height: 20px;
  text-align: center;
  white-space: nowrap;
  cursor: pointer;
}
.ptbr-badge[data-state="checking"] { background: #5f5f5f; }
.ptbr-badge[data-state="error"] { background: #a0332b; }
@media (prefers-color-scheme: dark) {
  .ptbr-card { background: #2b2b2b; border-color: #4a4a4a; color: #ececec; }
  .ptbr-rule { color: #b0b0b0; }
  .ptbr-change s { color: #ff8a80; }
  .ptbr-change strong { color: #81c995; }
  .ptbr-actions button { background: #3a3a3a; border-color: #555; }
}
"#;

const HOVER_MS: i32 = 200;
const LEAVE_MS: i32 = 300;

// Only a thin strip at the bottom of a segment takes the mouse: the word itself stays clickable
// for the editor.
const HIT_STYLE: &str = "position: absolute; left: 0px; right: 0px; bottom: 0px; height: 4px; pointer-events: auto;";

fn badge_label(e: CheckError) -> &'static str {
    match e {
        CheckError::NoKey => "sem chave",
        CheckError::Auth => "chave inválida",
        CheckError::RateLimit => "limite atingido",
        CheckError::Request => "modelo recusado",
        CheckError::Busy => "provider indisponível",
        CheckError::Truncated => "resposta cortada",
        CheckError::Provider | CheckError::Network => "erro do provider",
        CheckError::Local => "Ollama inacessível",
        CheckError::LocalOllaya => "Ollaya inacessível",
        CheckError::Reload => "recarregue a página",
    }
}

/// What a card's button asks of the suggestion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Accept,
    Ignore,
    /// "Sempre ignorar": drop it and add the original to the never-fix list.
    NeverFix,
}

/// Called with no borrow of the overlay held: it may render again.
pub type OnAction = Rc<dyn Fn(&HtmlElement, &Located, Action)>;

/// Client coordinates.
#[derive(Clone, Copy, Debug)]
struct Rect {
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
    width: f64,
    height: f64,
}

impl From<&DomRect> for Rect {
    fn from(r: &DomRect) -> Rect {
        Rect {
            left: r.left(),
            top: r.top(),
            right: r.right(),
            bottom: r.bottom(),
            width: r.width(),
            height: r.height(),
        }
    }
}

impl Rect {
    fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.left && x <= self.right && y >= self.top && y <= self.bottom
    }

    fn visible_in(&self, clip: &Rect) -> bool {
        self.width > 0.0
            && self.bottom > clip.top
            && self.top < clip.bottom
            && self.right > clip.left
            && self.left < clip.right
    }
}

// Where an underlined span is on screen, for the hover test; `idx` into `Drawn::located`.
struct Hit {
    rect: Rect,
    idx: usize,
}

struct Drawn {
    located: Vec<Located>,
    offset: u32,
    layer: HtmlElement,
    badge: Option<HtmlElement>,
    hits: Vec<Hit>,
}

// `hover`: opened by resting the mouse on a word, so it closes when the mouse leaves. A card
// opened by a click (underline strip or badge) stays until Escape or a click outside.
struct Card {
    el: HtmlElement,
    node: HtmlElement,
    idx: usize,
    hover: bool,
    start: u32,
    end: u32,
}

/// The overlay of a page; cloning it shares it.
#[derive(Clone)]
pub struct Overlay(Rc<RefCell<Inner>>);

struct Inner {
    weak: Weak<RefCell<Inner>>,
    doc: Document,
    win: Window,
    root: Option<ShadowRoot>,
    drawn: ElMap<Drawn>,
    card: Option<Card>,
    on_action: OnAction,
    // The rule titles of both skills, by code.
    titles: BTreeMap<String, String>,
    // The word under the mouse, to open when HOVER_MS pass.
    pending_open: Option<(HtmlElement, usize)>,
    open_timer: Option<Timer>,
    close_timer: Option<Timer>,
    on_mousedown: Closure<dyn FnMut(Event)>,
    on_keydown: Closure<dyn FnMut(KeyboardEvent)>,
    on_mousemove: Closure<dyn FnMut(MouseEvent)>,
    on_root_click: Closure<dyn FnMut(Event)>,
    on_root_mousedown: Closure<dyn FnMut(Event)>,
    // Scratch, reused across draws.
    rects: Vec<Rect>,
    buf: String,
}

impl Overlay {
    pub fn new(doc: Document, win: Window, on_action: OnAction) -> Overlay {
        let inner = Rc::new_cyclic(|weak: &Weak<RefCell<Inner>>| {
            let w = weak.clone();
            let on_mousedown = Closure::new(move |e: Event| {
                if let Some(o) = w.upgrade() {
                    o.borrow_mut().close_on_outside(&e);
                }
            });
            let w = weak.clone();
            let on_keydown = Closure::new(move |e: KeyboardEvent| {
                if let Some(o) = w.upgrade()
                    && e.key() == "Escape"
                {
                    o.borrow_mut().close_card();
                }
            });
            let w = weak.clone();
            let on_mousemove = Closure::new(move |e: MouseEvent| {
                if let Some(o) = w.upgrade() {
                    o.borrow_mut().on_mouse_move(&e);
                }
            });
            let w = weak.clone();
            let on_root_click = Closure::new(move |e: Event| {
                let Some(o) = w.upgrade() else { return };
                let action = o.borrow_mut().on_root_click(&e);
                if let Some((el, l, action)) = action {
                    let on_action = o.borrow().on_action.clone();
                    on_action(&el, &l, action);
                }
            });
            let on_root_mousedown = Closure::new(|e: Event| e.prevent_default());
            RefCell::new(Inner {
                weak: weak.clone(),
                doc,
                win,
                root: None,
                drawn: ElMap::new(),
                card: None,
                on_action,
                titles: rule_titles(SKILL).into_iter().chain(rule_titles(CIENTIFICO)).collect(),
                pending_open: None,
                open_timer: None,
                close_timer: None,
                on_mousedown,
                on_keydown,
                on_mousemove,
                on_root_click,
                on_root_mousedown,
                rects: vec![],
                buf: String::new(),
            })
        });
        {
            let o = inner.borrow();
            let listen = |event: &str, f: &js_sys::Function| {
                let _ = o.doc.add_event_listener_with_callback_and_bool(event, f, true);
            };
            listen("mousedown", o.on_mousedown.as_ref().unchecked_ref());
            listen("keydown", o.on_keydown.as_ref().unchecked_ref());
            listen("mousemove", o.on_mousemove.as_ref().unchecked_ref());
        }
        Overlay(inner)
    }

    /// The underlines of `el`, replacing the previous ones; its open card closes.
    pub fn render_suggestions(&self, el: &HtmlElement, located: Vec<Located>, offset: u32) {
        let mut o = self.0.borrow_mut();
        let i = o.entry(el);
        {
            let d = &mut o.drawn.at_mut(i).1;
            d.located = located;
            d.offset = offset;
        }
        if o.card.as_ref().is_some_and(|c| same(&c.el, el)) {
            o.close_card();
        }
        o.draw(i);
    }

    pub fn render_badge(&self, el: &HtmlElement, status: Status, count: usize) {
        self.0.borrow_mut().render_badge(el, status, count);
    }

    /// Removes everything drawn for an editor: its underlines, its card and its badge.
    pub fn forget(&self, el: &HtmlElement) {
        self.0.borrow_mut().forget(el);
    }

    /// Redraws every editor; called on scroll and resize. Forgets editors that left the page.
    pub fn reposition_all(&self) {
        self.0.borrow_mut().reposition_all();
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        let doc = &self.doc;
        let _ = doc.remove_event_listener_with_callback_and_bool(
            "mousedown",
            self.on_mousedown.as_ref().unchecked_ref(),
            true,
        );
        let _ =
            doc.remove_event_listener_with_callback_and_bool("keydown", self.on_keydown.as_ref().unchecked_ref(), true);
        let _ = doc.remove_event_listener_with_callback_and_bool(
            "mousemove",
            self.on_mousemove.as_ref().unchecked_ref(),
            true,
        );
    }
}

impl Inner {
    /// The shadow root; made again when the site removed the host.
    fn root(&mut self) -> ShadowRoot {
        if let Some(r) = &self.root
            && r.host().is_connected()
        {
            return r.clone();
        }
        let existing = self.doc.get_element_by_id(HOST_ID);
        let root = match existing.as_ref().and_then(Element::shadow_root) {
            Some(r) => r,
            None => {
                if let Some(h) = existing {
                    h.remove();
                }
                let host = self.doc.create_element("div").expect("createElement");
                host.set_id(HOST_ID);
                let root = host.attach_shadow(&ShadowRootInit::new(ShadowRootMode::Open)).expect("attachShadow");
                let style = self.doc.create_element("style").expect("createElement");
                style.set_text_content(Some(STYLE));
                let _ = root.append_child(&style);
                let parent: Element = match self.doc.document_element() {
                    Some(e) => e,
                    None => self.doc.body().expect("a body").into(),
                };
                let _ = parent.append_child(&host);
                root
            }
        };
        let _ = root.add_event_listener_with_callback("click", self.on_root_click.as_ref().unchecked_ref());
        let _ = root.add_event_listener_with_callback("mousedown", self.on_root_mousedown.as_ref().unchecked_ref());
        self.root = Some(root.clone());
        root
    }

    fn div(&self, class: &str) -> HtmlElement {
        let d: HtmlElement = self.doc.create_element("div").expect("createElement").unchecked_into();
        d.set_class_name(class);
        d
    }

    // The index of `el` in `drawn`, with a layer in the root.
    fn entry(&mut self, el: &HtmlElement) -> usize {
        let root = self.root();
        if let Some(i) = self.drawn.position(el)
            && root.contains(Some(&self.drawn.at(i).1.layer))
        {
            return i;
        }
        let layer = self.div("ptbr-layer");
        let _ = root.append_child(&layer);
        let badge = self.drawn.get_mut(el).and_then(|d| d.badge.take());
        self.drawn.insert(el.clone(), Drawn { located: vec![], offset: 0, layer, badge, hits: vec![] })
    }

    fn draw(&mut self, i: usize) {
        let root = self.root();
        let scroll = scroll(&self.win);
        let Inner { drawn, doc, win, rects, buf, .. } = self;
        let (el, d) = drawn.at_mut(i);
        d.layer.replace_children_with_node_0();
        d.hits.clear();
        let clip = Rect::from(&el.get_bounding_client_rect());
        let source = RectSource::new(el);
        for (idx, l) in d.located.iter().enumerate() {
            let u: HtmlElement = doc.create_element("div").expect("createElement").unchecked_into();
            u.set_class_name(match l.suggestion.severity {
                Severity::Erro => "ptbr-underline ptbr-underline-erro",
                Severity::Aviso => "ptbr-underline ptbr-underline-aviso",
            });
            u.set_title(&l.suggestion.rule);
            buf.clear();
            let _ = write!(buf, "{idx}");
            let _ = u.set_attribute("data-i", buf);
            source.rects_into(doc, win, &root, l.start + d.offset, l.end + d.offset, rects);
            for r in rects.iter().filter(|r| r.visible_in(&clip)) {
                d.hits.push(Hit { rect: *r, idx });
                let seg: HtmlElement = doc.create_element("div").expect("createElement").unchecked_into();
                seg.set_class_name("ptbr-seg");
                let st = seg.style();
                place(&st, r, scroll, buf);
                let _ = st.set_property("width", px(buf, r.width));
                let _ = st.set_property("height", px(buf, r.height));
                let _ = st.set_property("pointer-events", "none");
                let hit = doc.create_element("div").expect("createElement");
                hit.set_class_name("ptbr-hit");
                let _ = hit.set_attribute("style", HIT_STYLE);
                let _ = seg.append_child(&hit);
                let _ = u.append_child(&seg);
            }
            let _ = d.layer.append_child(&u);
        }
    }

    fn render_badge(&mut self, el: &HtmlElement, status: Status, count: usize) {
        let i = self.entry(el);
        let root = self.root();
        let scroll = scroll(&self.win);
        let Inner { drawn, doc, buf, .. } = self;
        let (el, d) = drawn.at_mut(i);
        if d.badge.as_ref().is_none_or(|b| !b.is_connected()) {
            let b: HtmlElement = doc.create_element("div").expect("createElement").unchecked_into();
            b.set_class_name("ptbr-badge");
            let _ = root.append_child(&b);
            d.badge = Some(b);
        }
        let b = d.badge.as_ref().expect("the badge");
        buf.clear();
        let (state, text) = match status {
            Status::Idle => {
                let _ = write!(buf, "{count}");
                ("idle", buf.as_str())
            }
            Status::Checking => ("checking", "…"),
            Status::Failed(e) => ("error", badge_label(e)),
        };
        let _ = b.set_attribute("data-state", state);
        b.set_text_content(Some(text));
        b.set_title(&format!("ptbr-writer: {text}"));
        place_badge(el, b, scroll, buf);
    }

    fn forget(&mut self, el: &HtmlElement) {
        let Some(d) = self.drawn.remove(el) else { return };
        d.layer.remove();
        if let Some(b) = d.badge {
            b.remove();
        }
        if self.card.as_ref().is_some_and(|c| same(&c.el, el)) {
            self.close_card();
        }
    }

    fn reposition_all(&mut self) {
        let mut i = 0;
        while i < self.drawn.len() {
            let el = self.drawn.at(i).0.clone();
            if !el.is_connected() {
                self.forget(&el);
                continue;
            }
            self.draw(i);
            let scroll = scroll(&self.win);
            let (el, d) = self.drawn.at_mut(i);
            if let Some(b) = &d.badge {
                place_badge(el, b, scroll, &mut self.buf);
            }
            i += 1;
        }
        if self.card.is_some() {
            self.place_card();
        }
    }

    fn open_card(&mut self, el: &HtmlElement, idx: usize, hover: bool) {
        self.close_card();
        let Some(d) = self.drawn.get(el) else { return };
        let Some(l) = d.located.get(idx) else { return };
        let (start, end) = (l.start + d.offset, l.end + d.offset);
        let node = self.div("ptbr-card");
        let _ = node.set_attribute("role", "dialog");
        let rule = self.p("ptbr-rule");
        rule.set_text_content(Some(&card_heading(l, &self.titles)));
        let change = self.p("ptbr-change");
        // A detection (Ollaya) has no replacement: the card names the rule and the sentence, with
        // only Ignorar.
        let detect = l.suggestion.detect_only;
        if detect {
            let _ = change.append_with_str_1(&l.suggestion.original);
        } else {
            let s = self.doc.create_element("s").expect("createElement");
            s.set_text_content(Some(&l.suggestion.original));
            let strong = self.doc.create_element("strong").expect("createElement");
            strong.set_text_content(Some(&l.suggestion.replacement));
            let _ = change.append_with_node_1(&s);
            let _ = change.append_with_str_1(" → ");
            let _ = change.append_with_node_1(&strong);
        }
        let actions = self.div("ptbr-actions");
        if !detect {
            let _ = actions.append_child(&self.button("accept", "Aceitar"));
        }
        let _ = actions.append_child(&self.button("ignore", "Ignorar"));
        if offers_never_fix(l) {
            let _ = actions.append_child(&self.button("never", "Sempre ignorar"));
        }
        let _ = node.append_with_node_3(&rule, &change, &actions);
        let _ = self.root().append_child(&node);
        self.card = Some(Card { el: el.clone(), node, idx, hover, start, end });
        self.place_card();
    }

    fn p(&self, class: &str) -> HtmlElement {
        let p: HtmlElement = self.doc.create_element("p").expect("createElement").unchecked_into();
        p.set_class_name(class);
        p
    }

    fn button(&self, action: &str, label: &str) -> Element {
        let b = self.doc.create_element("button").expect("createElement");
        let _ = b.set_attribute("type", "button");
        let _ = b.set_attribute("data-action", action);
        b.set_text_content(Some(label));
        b
    }

    fn close_card(&mut self) {
        self.close_timer = None;
        if let Some(c) = self.card.take() {
            c.node.remove();
        }
    }

    fn place_card(&mut self) {
        let root = self.root();
        let scroll = scroll(&self.win);
        let Inner { card, doc, win, rects, buf, .. } = self;
        let Some(c) = card else { return };
        RectSource::new(&c.el).rects_into(doc, win, &root, c.start, c.end, rects);
        let r = rects.last().copied().unwrap_or_else(|| Rect::from(&c.el.get_bounding_client_rect()));
        let h = f64::from(c.node.offset_height());
        let w = f64::from(c.node.offset_width());
        let vh = win.inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(f64::INFINITY);
        let vw = win.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(f64::INFINITY);
        // Below the text when it fits, else above it: a composer at the bottom of the window
        // (WhatsApp) would push the card off screen.
        let top = if r.bottom + 4.0 + h > vh && r.top - 4.0 - h >= 0.0 { r.top - 4.0 - h } else { r.bottom + 4.0 };
        let left = r.left.min(vw - w - 4.0).max(4.0);
        let st = c.node.style();
        let _ = st.set_property("left", px(buf, left + scroll.0));
        let _ = st.set_property("top", px(buf, top + scroll.1));
    }

    fn hit_at(&self, x: f64, y: f64) -> Option<(usize, usize)> {
        self.drawn
            .iter()
            .enumerate()
            .find_map(|(di, (_, d))| d.hits.iter().find(|h| h.rect.contains(x, y)).map(|h| (di, h.idx)))
    }

    // The words stay clickable for the editor (no pointer events on them), so hover is found by
    // comparing the mouse position with the drawn rects.
    fn on_mouse_move(&mut self, e: &MouseEvent) {
        let (x, y) = (f64::from(e.client_x()), f64::from(e.client_y()));
        // A card whose node left the page (the site rebuilt the overlay host) no longer counts.
        if self.card.as_ref().is_some_and(|c| !c.node.is_connected()) {
            self.card = None;
        }
        if let Some(c) = &self.card
            && in_path(e, &c.node)
        {
            self.close_timer = None;
            return;
        }
        let hit = self.hit_at(x, y);
        self.open_timer = None;
        if let Some((di, idx)) = hit {
            let el = &self.drawn.at(di).0;
            if self.card.as_ref().is_some_and(|c| c.idx == idx && same(&c.el, el)) {
                self.close_timer = None;
                return;
            }
            self.pending_open = Some((el.clone(), idx));
            let w = self.weak.clone();
            self.open_timer = Some(Timer::new(HOVER_MS, move || {
                if let Some(o) = w.upgrade() {
                    o.borrow_mut().open_pending();
                }
            }));
            return;
        }
        if self.card.as_ref().is_some_and(|c| c.hover) {
            let w = self.weak.clone();
            self.close_timer = Some(Timer::new(LEAVE_MS, move || {
                if let Some(o) = w.upgrade() {
                    let mut o = o.borrow_mut();
                    if o.card.as_ref().is_some_and(|c| c.hover) {
                        o.close_card();
                    }
                }
            }));
        }
    }

    fn open_pending(&mut self) {
        if self.card.as_ref().is_some_and(|c| !c.hover) {
            return;
        }
        if let Some((el, idx)) = self.pending_open.take() {
            self.open_card(&el, idx, true);
        }
    }

    fn close_on_outside(&mut self, e: &Event) {
        if let Some(c) = &self.card
            && !in_path(e, &c.node)
        {
            self.close_card();
        }
    }

    // A click in the shadow root: a card button gives its action back, an underline or the badge
    // opens a card.
    fn on_root_click(&mut self, e: &Event) -> Option<(HtmlElement, Located, Action)> {
        let target = e.target()?.dyn_into::<Element>().ok()?;
        if let Some(b) = target.closest("button[data-action]").ok().flatten() {
            e.stop_propagation();
            let action = match b.get_attribute("data-action").as_deref() {
                Some("accept") => Action::Accept,
                Some("ignore") => Action::Ignore,
                Some("never") => Action::NeverFix,
                _ => return None,
            };
            let c = self.card.take()?;
            self.close_timer = None;
            c.node.remove();
            let l = self.drawn.get(&c.el)?.located.get(c.idx)?.clone();
            return Some((c.el, l, action));
        }
        if let Some(u) = target.closest(".ptbr-underline").ok().flatten() {
            e.stop_propagation();
            let idx: usize = u.get_attribute("data-i")?.parse().ok()?;
            let layer = u.parent_element()?;
            let di = self.drawn.iter().position(|(_, d)| same(&d.layer, &layer))?;
            let el = self.drawn.at(di).0.clone();
            self.open_card(&el, idx, false);
            return None;
        }
        if let Some(b) = target.closest(".ptbr-badge").ok().flatten() {
            e.stop_propagation();
            // The badge opens the first suggestion, the same card a click on its underline opens.
            let di = self.drawn.iter().position(|(_, d)| d.badge.as_ref().is_some_and(|x| same(x, &b)))?;
            let (el, d) = self.drawn.at(di);
            if !d.located.is_empty() {
                let el = el.clone();
                self.open_card(&el, 0, false);
            }
        }
        None
    }
}

fn scroll(win: &Window) -> (f64, f64) {
    (win.scroll_x().unwrap_or(0.0), win.scroll_y().unwrap_or(0.0))
}

/// `v` as a CSS length, in `buf`.
fn px(buf: &mut String, v: f64) -> &str {
    buf.clear();
    let _ = write!(buf, "{v}px");
    buf
}

fn place(st: &CssStyleDeclaration, r: &Rect, scroll: (f64, f64), buf: &mut String) {
    let _ = st.set_property("left", px(buf, r.left + scroll.0));
    let _ = st.set_property("top", px(buf, r.top + scroll.1));
}

fn place_badge(el: &HtmlElement, b: &HtmlElement, scroll: (f64, f64), buf: &mut String) {
    let r = el.get_bounding_client_rect();
    let st = b.style();
    let _ = st.set_property("display", if r.width() != 0.0 || r.height() != 0.0 { "" } else { "none" });
    // Bottom-right corner, inside the field.
    let _ = st.set_property("left", px(buf, r.right() - 28.0 + scroll.0));
    let _ = st.set_property("top", px(buf, r.bottom() - 26.0 + scroll.1));
}

fn push_all(list: Option<DomRectList>, out: &mut Vec<Rect>) {
    let Some(list) = list else { return };
    for i in 0..list.length() {
        if let Some(r) = list.get(i) {
            out.push(Rect::from(&r));
        }
    }
}

// Client rects of [start, end) in the editor's text.
enum RectSource<'a> {
    Field(&'a HtmlElement),
    Editable(TextModel),
}

impl<'a> RectSource<'a> {
    fn new(el: &'a HtmlElement) -> Self {
        if is_field(el) { RectSource::Field(el) } else { RectSource::Editable(text_model(el)) }
    }

    fn rects_into(&self, doc: &Document, win: &Window, root: &ShadowRoot, start: u32, end: u32, out: &mut Vec<Rect>) {
        out.clear();
        match self {
            RectSource::Field(el) => field_rects(doc, win, root, el, start, end, out),
            RectSource::Editable(m) => {
                if let Some(r) = range_for(m, start, end) {
                    push_all(r.get_client_rects(), out);
                }
            }
        }
    }
}

// A textarea or input has no Range over its text: a hidden mirror div with the same box and font
// lays out the same text, and the rects of the mirror's Range stand in for the field's.
const COPIED: [&str; 22] = [
    "font-family",
    "font-size",
    "font-weight",
    "font-style",
    "font-variant",
    "font-stretch",
    "line-height",
    "letter-spacing",
    "word-spacing",
    "text-transform",
    "text-indent",
    "text-align",
    "tab-size",
    "direction",
    "padding-top",
    "padding-right",
    "padding-bottom",
    "padding-left",
    "border-top-width",
    "border-right-width",
    "border-bottom-width",
    "border-left-width",
];

fn field_rects(
    doc: &Document,
    win: &Window,
    root: &ShadowRoot,
    el: &HtmlElement,
    start: u32,
    end: u32,
    out: &mut Vec<Rect>,
) {
    let mirror: HtmlElement = doc.create_element("div").expect("createElement").unchecked_into();
    mirror.set_class_name("ptbr-mirror");
    let st = mirror.style();
    let (mut border_x, mut border_y) = (0.0, 0.0);
    if let Some(cs) = win.get_computed_style(el).ok().flatten() {
        for p in COPIED {
            if let Ok(v) = cs.get_property_value(p) {
                let _ = st.set_property(p, &v);
            }
        }
        let length = |p: &str| cs.get_property_value(p).ok().and_then(|v| v.trim_end_matches("px").parse::<f64>().ok());
        border_x = length("border-left-width").unwrap_or(0.0) + length("border-right-width").unwrap_or(0.0);
        border_y = length("border-top-width").unwrap_or(0.0) + length("border-bottom-width").unwrap_or(0.0);
    }
    let mut buf = String::new();
    // clientWidth excludes the scrollbar, so the mirror wraps where the field wraps.
    let _ = st.set_property("width", px(&mut buf, f64::from(el.client_width()) + border_x));
    let _ = st.set_property("height", px(&mut buf, f64::from(el.client_height()) + border_y));
    let _ = st
        .set_property("white-space", if el.is_instance_of::<web_sys::HtmlInputElement>() { "pre" } else { "pre-wrap" });
    let _ = st.set_property("overflow-wrap", "break-word");
    place(&st, &Rect::from(&el.get_bounding_client_rect()), scroll(win), &mut buf);
    let value = field_value(el).unwrap_or_default();
    let text = doc.create_text_node(&format!("{value}\u{200B}"));
    let _ = mirror.append_child(&text);
    let _ = root.append_child(&mirror);
    mirror.set_scroll_top(el.scroll_top());
    mirror.set_scroll_left(el.scroll_left());
    if end > start
        && end <= len16(&value)
        && let Ok(r) = doc.create_range()
        && r.set_start(&text, start).is_ok()
        && r.set_end(&text, end).is_ok()
    {
        push_all(r.get_client_rects(), out);
    }
    mirror.remove();
}

/// Whether `node` is on the event's composed path (the mouse is on the card).
fn in_path(e: &Event, node: &HtmlElement) -> bool {
    e.composed_path().includes(AsRef::<JsValue>::as_ref(node), 0)
}
