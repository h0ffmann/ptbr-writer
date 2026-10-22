//! Runs the content script on this page. It follows the settings: when this site's level changes,
//! the fields start over with the new level; when the site or the extension is switched off, it
//! stops and clears what it drew. Each focused text field gets the watcher, and what it shows (its
//! suggestions, its badge) is kept here and drawn by the overlay.

use crate::chrome;
use crate::content::ElMap;
use crate::content::apply::accept_suggestion;
use crate::content::overlay::{Action, Overlay};
use crate::content::text::text_model;
use crate::content::watcher::{Hooks, Status, Watcher};
use crate::settings::{level_for, load_settings};
use crate::suggestions::apply_to_string;
use crate::types::{Level, Located};
use crate::utf16::len16;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::spawn_local;
use web_sys::{Document, FocusEvent, HtmlElement, HtmlInputElement, HtmlTextAreaElement, Window};

const TEXT_INPUTS: [&str; 4] = ["text", "search", "email", "url"];

/// The editable root a focus event belongs to, or `None` when it is not a text field.
pub fn editable_root(target: &JsValue) -> Option<HtmlElement> {
    if let Some(t) = target.dyn_ref::<HtmlTextAreaElement>() {
        return Some(t.clone().into());
    }
    if let Some(i) = target.dyn_ref::<HtmlInputElement>() {
        return TEXT_INPUTS.contains(&i.type_().as_str()).then(|| i.clone().into());
    }
    let el = target.dyn_ref::<HtmlElement>()?;
    if !el.is_content_editable() {
        return None;
    }
    let mut root = el.clone();
    while let Some(p) = root.parent_element().and_then(|p| p.dyn_into::<HtmlElement>().ok())
        && p.is_content_editable()
    {
        root = p;
    }
    Some(root)
}

#[derive(Default)]
struct FieldState {
    located: Vec<Located>,
    offset: u32,
    status: Status,
}

thread_local! {
    // The content script lives as long as the page.
    static CONTENT: RefCell<Option<Content>> = const { RefCell::new(None) };
}

/// Starts the content script; its failure to read the settings goes to the console.
pub fn start() {
    spawn_local(async {
        match Content::start().await {
            Ok(c) => CONTENT.with(|s| *s.borrow_mut() = Some(c)),
            Err(e) => web_sys::console::error_1(&e),
        }
    });
}

/// The running content script; dropping it stops it and removes its listeners.
pub struct Content {
    _alive: Rc<RefCell<Inner>>,
}

struct Inner {
    doc: Document,
    win: Window,
    fields: ElMap<FieldState>,
    level: Option<Level>,
    watcher: Option<Watcher>,
    overlay: Overlay,
    frame: Option<i32>,
    on_focus: Closure<dyn FnMut(FocusEvent)>,
    on_scroll: Closure<dyn FnMut()>,
    on_frame: Closure<dyn FnMut()>,
    on_changed: Closure<dyn FnMut(JsValue, JsValue)>,
}

impl Content {
    pub async fn start() -> Result<Content, JsValue> {
        let win = web_sys::window().ok_or("no window")?;
        let doc = win.document().ok_or("no document")?;
        let inner = Rc::new_cyclic(|weak: &Weak<RefCell<Inner>>| {
            let w = weak.clone();
            let overlay = Overlay::new(doc.clone(), win.clone(), Rc::new(move |el, l, action| act(&w, el, l, action)));
            let w = weak.clone();
            let on_focus = Closure::new(move |e: FocusEvent| {
                let first = e.composed_path().get(0);
                let target =
                    if first.is_undefined() { e.target().map_or(JsValue::UNDEFINED, JsValue::from) } else { first };
                if let Some(i) = w.upgrade() {
                    i.borrow_mut().attach(&target);
                }
            });
            let w = weak.clone();
            let on_scroll = Closure::new(move || schedule(&w));
            let w = weak.clone();
            let on_frame = Closure::new(move || frame(&w));
            let w = weak.clone();
            let on_changed = Closure::new(move |changes: JsValue, area: JsValue| {
                if area.as_string().as_deref() == Some("local")
                    && js_sys::Reflect::has(&changes, &"settings".into()).unwrap_or(false)
                {
                    let w = w.clone();
                    spawn_local(async move {
                        let _ = follow(&w).await;
                    });
                }
            });
            RefCell::new(Inner {
                doc: doc.clone(),
                win: win.clone(),
                fields: ElMap::new(),
                level: None,
                watcher: None,
                overlay,
                frame: None,
                on_focus,
                on_scroll,
                on_frame,
                on_changed,
            })
        });
        follow(&Rc::downgrade(&inner)).await?;
        let i = inner.borrow();
        chrome::STORAGE_ON_CHANGED.with(|e| {
            if let Some(e) = e {
                e.add_listener(i.on_changed.as_ref().unchecked_ref());
            }
        });
        doc.add_event_listener_with_callback_and_bool("focusin", i.on_focus.as_ref().unchecked_ref(), true)?;
        win.add_event_listener_with_callback_and_bool("scroll", i.on_scroll.as_ref().unchecked_ref(), true)?;
        win.add_event_listener_with_callback("resize", i.on_scroll.as_ref().unchecked_ref())?;
        drop(i);
        Ok(Content { _alive: inner })
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        self.stop();
        chrome::STORAGE_ON_CHANGED.with(|e| {
            if let Some(e) = e {
                e.remove_listener(self.on_changed.as_ref().unchecked_ref());
            }
        });
        let _ = self.doc.remove_event_listener_with_callback_and_bool(
            "focusin",
            self.on_focus.as_ref().unchecked_ref(),
            true,
        );
        let _ = self.win.remove_event_listener_with_callback_and_bool(
            "scroll",
            self.on_scroll.as_ref().unchecked_ref(),
            true,
        );
        let _ = self.win.remove_event_listener_with_callback("resize", self.on_scroll.as_ref().unchecked_ref());
    }
}

impl Inner {
    fn field(&mut self, el: &HtmlElement) -> &mut FieldState {
        let i = match self.fields.position(el) {
            Some(i) => i,
            None => self.fields.insert(el.clone(), FieldState::default()),
        };
        &mut self.fields.at_mut(i).1
    }

    fn render(&self, el: &HtmlElement) {
        let Some(f) = self.fields.get(el) else { return };
        self.overlay.render_suggestions(el, f.located.clone(), f.offset);
        self.overlay.render_badge(el, f.status, f.located.len());
    }

    fn attach(&mut self, target: &JsValue) {
        let Some(el) = editable_root(target) else { return };
        if self.watcher.is_none() || self.fields.contains(&el) {
            return;
        }
        self.field(&el);
        if let Some(w) = &self.watcher {
            w.attach(&el);
        }
    }

    fn stop(&mut self) {
        for (el, _) in self.fields.iter() {
            if let Some(w) = &self.watcher {
                w.detach(el);
            }
            self.overlay.forget(el);
        }
        self.fields.clear();
        self.watcher = None;
    }

    fn drop_suggestion(&mut self, el: &HtmlElement, l: &Located) {
        self.field(el).located.retain(|x| x != l);
        self.render(el);
    }
}

// Reads the settings and starts over when this site's level changed.
async fn follow(weak: &Weak<RefCell<Inner>>) -> Result<(), JsValue> {
    let settings = load_settings().await?;
    let Some(inner) = weak.upgrade() else { return Ok(()) };
    let host = inner.borrow().win.location().hostname()?;
    let next = level_for(&host, &settings);
    let mut i = inner.borrow_mut();
    if next == i.level {
        return Ok(());
    }
    i.stop();
    i.level = next;
    let Some(level) = next else { return Ok(()) };
    let w = weak.clone();
    let on_result = Box::new(move |el: &HtmlElement, located: Vec<Located>, offset: u32| {
        if let Some(i) = w.upgrade() {
            let mut i = i.borrow_mut();
            let f = i.field(el);
            f.located = located;
            f.offset = offset;
            i.render(el);
        }
    });
    let w = weak.clone();
    let on_state = Box::new(move |el: &HtmlElement, status: Status| {
        if let Some(i) = w.upgrade() {
            let mut i = i.borrow_mut();
            i.field(el).status = status;
            i.render(el);
        }
    });
    i.watcher = Some(Watcher::new(level, Hooks { on_result, on_state }));
    // The focused field gets no new focusin: before the script ran, or when the site was switched on.
    let active = i.doc.active_element().map_or(JsValue::UNDEFINED, JsValue::from);
    i.attach(&active);
    Ok(())
}

// A card's button. The accepted edit is the extension's own: the other suggestions stay, shifted
// by the change in length, and the API is not called again until the user types.
fn act(weak: &Weak<RefCell<Inner>>, el: &HtmlElement, l: &Located, action: Action) {
    let Some(inner) = weak.upgrade() else { return };
    if action == Action::Ignore {
        return inner.borrow_mut().drop_suggestion(el, l);
    }
    let offset = {
        let mut i = inner.borrow_mut();
        let offset = i.field(el).offset;
        let at = Located { suggestion: l.suggestion.clone(), start: l.start + offset, end: l.end + offset };
        let expected = apply_to_string(&text_model(el).text, &at);
        if let Some(w) = &i.watcher {
            w.expect_edit(el, Some(expected));
        }
        offset
    };
    // The field's input event fires inside: no borrow is held across it.
    let ok = accept_suggestion(el, l, offset);
    let mut i = inner.borrow_mut();
    if !ok {
        if let Some(w) = &i.watcher {
            w.expect_edit(el, None);
        }
        return i.drop_suggestion(el, l);
    }
    let delta = i64::from(len16(&l.suggestion.replacement)) - i64::from(l.end - l.start);
    let shift = |at: u32| (i64::from(at) + delta) as u32;
    let f = i.field(el);
    f.located.retain(|x| x != l);
    for x in f.located.iter_mut().filter(|x| x.start >= l.end) {
        x.start = shift(x.start);
        x.end = shift(x.end);
    }
    i.render(el);
}

fn schedule(weak: &Weak<RefCell<Inner>>) {
    let Some(inner) = weak.upgrade() else { return };
    let mut i = inner.borrow_mut();
    if i.frame.is_some() {
        return;
    }
    i.frame = i.win.request_animation_frame(i.on_frame.as_ref().unchecked_ref()).ok();
}

// Forgets the fields that left the page, then redraws.
fn frame(weak: &Weak<RefCell<Inner>>) {
    let Some(inner) = weak.upgrade() else { return };
    let mut i = inner.borrow_mut();
    i.frame = None;
    let Inner { fields, watcher, .. } = &mut *i;
    fields.retain(|el, _| {
        let keep = el.is_connected();
        if !keep && let Some(w) = watcher {
            w.detach(el);
        }
        keep
    });
    i.overlay.reposition_all();
}
