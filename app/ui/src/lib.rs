//! The window: a text, its level and Revisar; the text underlined where a rule asks for a change,
//! one card per underline (Aceitar, Ignorar); the provider settings. Every decision is the Rust
//! side's (../src-tauri): this draws the `Boot` and `View` it returns and sends back what was clicked.

use ptbr_app_wire::{Boot, Card, Prefs, View};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{
    Document, Element, HtmlButtonElement, HtmlElement, HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement,
    KeyboardEvent,
};

#[wasm_bindgen]
extern "C" {
    // `app.withGlobalTauri` (tauri.conf.json) puts Tauri's invoke on the window.
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], js_name = invoke, catch)]
    async fn tauri_invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

/// A command of the Rust side; its error is the message it returned.
async fn invoke<A: Serialize, R: DeserializeOwned>(cmd: &str, args: &A) -> Result<R, String> {
    let args = args.serialize(&serde_wasm_bindgen::Serializer::json_compatible()).map_err(|e| e.to_string())?;
    let out = tauri_invoke(cmd, args).await.map_err(|e| e.as_string().unwrap_or_else(|| format!("{e:?}")))?;
    serde_wasm_bindgen::from_value(out).map_err(|e| e.to_string())
}

#[derive(Serialize)]
struct NoArgs {}

#[derive(Serialize)]
struct CheckArgs<'a> {
    text: &'a str,
    level: &'a str,
}

#[derive(Serialize)]
struct IdArgs {
    id: usize,
}

#[derive(Serialize)]
struct PrefsArgs<'a> {
    prefs: &'a Prefs,
}

fn document() -> Document {
    web_sys::window().expect("a window").document().expect("a document")
}

fn by_id<T: JsCast>(id: &str) -> T {
    document().get_element_by_id(id).unwrap_or_else(|| panic!("#{id}")).unchecked_into()
}

fn create<T: JsCast>(tag: &str, class: &str, text: &str) -> T {
    let e = document().create_element(tag).expect("createElement");
    if !class.is_empty() {
        e.set_class_name(class);
    }
    if !text.is_empty() {
        e.set_text_content(Some(text));
    }
    e.unchecked_into()
}

/// Runs `f` on each `event`; the listeners live as long as the window, so the closure is leaked.
fn on<E: JsCast + 'static>(target: &web_sys::EventTarget, event: &str, mut f: impl FnMut(E) + 'static) {
    let cb = Closure::<dyn FnMut(web_sys::Event)>::new(move |e: web_sys::Event| f(e.unchecked_into()));
    target.add_event_listener_with_callback(event, cb.as_ref().unchecked_ref()).expect("addEventListener");
    cb.forget();
}

fn fill_select(select: &HtmlSelectElement, items: &[(String, String)], selected: &str) {
    select.set_inner_html("");
    for (value, label) in items {
        let o: web_sys::HtmlOptionElement = create("option", "", label);
        o.set_value(value);
        o.set_selected(value == selected);
        select.append_child(&o).expect("appendChild");
    }
}

struct Ui {
    boot: Boot,
    prefs: Prefs,
}

type Shared = Rc<RefCell<Ui>>;

fn status(text: &str, error: bool) {
    let s: HtmlElement = by_id("status");
    s.set_text_content(Some(text));
    s.class_list().toggle_with_force("error", error).expect("classList");
}

/// The provider's model list and key field, from the prefs of the chosen provider.
fn show_provider(ui: &Ui) {
    let p = ui.boot.providers.iter().find(|p| p.id == ui.prefs.provider).unwrap_or(&ui.boot.providers[0]);
    let model = ui.prefs.models.get(&p.id).cloned().unwrap_or_default();
    let mut items: Vec<(String, String)> = p.models.iter().map(|m| (m.clone(), m.clone())).collect();
    if !model.is_empty() && !p.models.contains(&model) {
        items.insert(0, (model.clone(), model.clone()));
    }
    fill_select(&by_id("model"), &items, &model);
    let key: HtmlInputElement = by_id("key");
    key.set_value(ui.prefs.keys.get(&p.id).map_or("", String::as_str));
    by_id::<HtmlElement>("key-row").set_hidden(p.local);
}

/// The settings form into the prefs; the key and the model belong to the chosen provider.
fn read_form(ui: &mut Ui) {
    let p = by_id::<HtmlSelectElement>("provider").value();
    ui.prefs.models.insert(p.clone(), by_id::<HtmlSelectElement>("model").value());
    ui.prefs.keys.insert(p.clone(), by_id::<HtmlInputElement>("key").value().trim().to_string());
    ui.prefs.provider = p;
    ui.prefs.tone = by_id::<HtmlSelectElement>("tone").value();
    ui.prefs.rewrite = by_id::<HtmlInputElement>("rewrite").checked();
    ui.prefs.level = by_id::<HtmlSelectElement>("level").value();
}

async fn save(ui: Shared, quiet: bool) {
    read_form(&mut ui.borrow_mut());
    let prefs = ui.borrow().prefs.clone();
    match invoke::<_, Prefs>("save_prefs", &PrefsArgs { prefs: &prefs }).await {
        Ok(saved) => {
            ui.borrow_mut().prefs = saved;
            show_provider(&ui.borrow());
            if !quiet {
                status("Configurações salvas.", false);
            }
        }
        Err(e) => status(&e, true),
    }
}

/// The underlined text and the cards; an empty view clears both.
fn render(view: &View) {
    let preview: HtmlElement = by_id("preview");
    preview.set_inner_html("");
    for s in &view.segments {
        let span: HtmlElement = match s.card.and_then(|id| view.cards.get(id)) {
            Some(card) => {
                let m: HtmlElement = create("mark", &card.severity, &s.text);
                m.set_attribute("data-card", &card.id.to_string()).expect("setAttribute");
                m.set_title(&card.rule);
                on(&m, "click", {
                    let id = card.id;
                    move |_: web_sys::Event| focus_card(id)
                });
                m
            }
            None => create("span", "", &s.text),
        };
        preview.append_child(&span).expect("appendChild");
    }
    let cards: HtmlElement = by_id("cards");
    cards.set_inner_html("");
    if view.cards.is_empty() {
        let empty = if view.text.is_empty() { "Revise um texto para ver as correções." } else { "Nada a corrigir." };
        cards.append_child(&create::<Element>("p", "muted", empty)).expect("appendChild");
    }
    for card in &view.cards {
        cards.append_child(&card_element(card)).expect("appendChild");
    }
    by_id::<HtmlElement>("count").set_text_content(Some(&view.cards.len().to_string()));
}

fn card_element(card: &Card) -> Element {
    let root: HtmlElement = create("article", &format!("card {}", card.severity), "");
    root.set_id(&format!("card-{}", card.id));
    let head: Element = create("header", "", "");
    head.append_child(&create::<Element>("span", "rule", &card.rule)).expect("appendChild");
    head.append_child(&create::<Element>("span", "title", &card.title)).expect("appendChild");
    root.append_child(&head).expect("appendChild");
    root.append_child(&create::<Element>("p", "old", &card.original)).expect("appendChild");
    if !card.detect_only {
        let new = if card.replacement.is_empty() { "(remover)" } else { &card.replacement };
        root.append_child(&create::<Element>("p", "new", new)).expect("appendChild");
    }
    let actions: Element = create("div", "actions", "");
    if !card.detect_only {
        actions.append_child(&action("Aceitar", "accept", card.id, true)).expect("appendChild");
    }
    actions.append_child(&action("Ignorar", "ignore", card.id, false)).expect("appendChild");
    root.append_child(&actions).expect("appendChild");
    for (event, hot) in [("mouseenter", true), ("mouseleave", false)] {
        let id = card.id;
        on(&root, event, move |_: web_sys::Event| highlight(id, hot));
    }
    root.unchecked_into()
}

fn action(label: &str, cmd: &'static str, id: usize, primary: bool) -> HtmlButtonElement {
    let b: HtmlButtonElement = create("button", if primary { "primary" } else { "" }, label);
    b.set_type("button");
    on(&b, "click", move |_: web_sys::Event| {
        wasm_bindgen_futures::spawn_local(async move {
            match invoke::<_, View>(cmd, &IdArgs { id }).await {
                Ok(view) => {
                    by_id::<HtmlTextAreaElement>("text").set_value(&view.text);
                    render(&view);
                }
                Err(e) => status(&e, true),
            }
        })
    });
    b
}

fn marks_of(id: usize) -> Vec<HtmlElement> {
    let list = document().query_selector_all(&format!("mark[data-card=\"{id}\"]")).expect("querySelectorAll");
    (0..list.length()).filter_map(|i| list.item(i)).map(|n| n.unchecked_into()).collect()
}

fn highlight(id: usize, hot: bool) {
    for m in marks_of(id) {
        m.class_list().toggle_with_force("hot", hot).expect("classList");
    }
}

fn focus_card(id: usize) {
    if let Some(card) = document().get_element_by_id(&format!("card-{id}")) {
        card.scroll_into_view();
        card.class_list().add_1("flash").expect("classList");
    }
}

async fn run_check(ui: Shared) {
    let text = by_id::<HtmlTextAreaElement>("text").value();
    if text.trim().is_empty() {
        status("Escreva ou cole um texto primeiro.", true);
        return;
    }
    let level = by_id::<HtmlSelectElement>("level").value();
    let button: HtmlButtonElement = by_id("check");
    button.set_disabled(true);
    let provider = ui.borrow().prefs.provider.clone();
    let name = ui.borrow().boot.providers.iter().find(|p| p.id == provider).map(|p| p.name.clone()).unwrap_or(provider);
    status(&format!("Revisando com {name}, nível {level}…"), false);
    let started = js_sys::Date::now();
    match invoke::<_, View>("check", &CheckArgs { text: &text, level: &level }).await {
        Ok(view) => {
            // The time a check took, start to end: what a "slow" report needs to say where.
            let took = format!("{:.1}", (js_sys::Date::now() - started) / 1000.0).replace('.', ",");
            let said = match view.cards.len() {
                0 => format!("Nada a corrigir ({took} s)."),
                1 => format!("1 correção ({took} s)."),
                n => format!("{n} correções ({took} s)."),
            };
            status(&said, false);
            render(&view);
        }
        Err(e) => status(&e, true),
    }
    button.set_disabled(false);
}

fn wire(ui: &Shared) {
    on(&by_id::<HtmlElement>("check"), "click", {
        let ui = ui.clone();
        move |_: web_sys::Event| wasm_bindgen_futures::spawn_local(run_check(ui.clone()))
    });
    // ⌘↩ or Ctrl+Enter in the text: Revisar.
    on(&by_id::<HtmlElement>("text"), "keydown", {
        let ui = ui.clone();
        move |e: KeyboardEvent| {
            if e.key() == "Enter" && (e.meta_key() || e.ctrl_key()) {
                e.prevent_default();
                wasm_bindgen_futures::spawn_local(run_check(ui.clone()));
            }
        }
    });
    // An edited text is no longer the checked one: its underlines would point at other words.
    on(&by_id::<HtmlElement>("text"), "input", |_: web_sys::Event| {
        if !by_id::<HtmlElement>("preview").inner_html().is_empty() {
            render(&View::default());
            status("Texto alterado: revise de novo.", false);
        }
    });
    on(&by_id::<HtmlElement>("provider"), "change", {
        let ui = ui.clone();
        move |_: web_sys::Event| {
            let mut u = ui.borrow_mut();
            u.prefs.provider = by_id::<HtmlSelectElement>("provider").value();
            show_provider(&u);
        }
    });
    on(&by_id::<HtmlElement>("level"), "change", {
        let ui = ui.clone();
        move |_: web_sys::Event| wasm_bindgen_futures::spawn_local(save(ui.clone(), true))
    });
    on(&by_id::<HtmlElement>("save"), "click", {
        let ui = ui.clone();
        move |_: web_sys::Event| wasm_bindgen_futures::spawn_local(save(ui.clone(), false))
    });
}

/// `window.ptbrAndroid`, the Android app's bridge (gen/android MainActivity); absent on the desktop.
fn android() -> Option<JsValue> {
    let w = web_sys::window()?;
    js_sys::Reflect::get(&w, &"ptbrAndroid".into()).ok().filter(|b| b.is_object())
}

fn call(bridge: &JsValue, method: &str) -> Option<JsValue> {
    let f: js_sys::Function = js_sys::Reflect::get(bridge, &method.into()).ok()?.dyn_into().ok()?;
    f.call0(bridge).ok()
}

/// Whether the overlay is on, read again whenever the window comes back (from Android's settings).
fn show_overlay(bridge: &JsValue) {
    let on = call(bridge, "overlayOn").and_then(|v| v.as_bool()).unwrap_or(false);
    let state: HtmlElement = by_id("overlay-state");
    state.set_text_content(Some(if on { "Sobreposição ligada." } else { "Sobreposição desligada." }));
    state.class_list().toggle_with_force("on", on).expect("classList");
    by_id::<HtmlElement>("overlay").set_hidden(on);
}

fn wire_android() {
    let Some(bridge) = android() else { return };
    by_id::<HtmlElement>("android").set_hidden(false);
    show_overlay(&bridge);
    on(&by_id::<HtmlElement>("overlay"), "click", {
        let bridge = bridge.clone();
        move |_: web_sys::Event| {
            call(&bridge, "openSettings");
        }
    });
    on(&document(), "visibilitychange", move |_: web_sys::Event| show_overlay(&bridge));
}

async fn start() -> Result<(), String> {
    let boot: Boot = invoke("boot", &NoArgs {}).await?;
    let pairs = |v: &[String]| v.iter().map(|x| (x.clone(), x.clone())).collect::<Vec<_>>();
    fill_select(&by_id("level"), &pairs(&boot.levels), &boot.prefs.level);
    fill_select(&by_id("tone"), &pairs(&boot.tones), &boot.prefs.tone);
    let providers: Vec<_> = boot
        .providers
        .iter()
        .map(|p| (p.id.clone(), if p.local { format!("{} (local)", p.name) } else { p.name.clone() }))
        .collect();
    fill_select(&by_id("provider"), &providers, &boot.prefs.provider);
    by_id::<HtmlInputElement>("rewrite").set_checked(boot.prefs.rewrite);
    by_id::<HtmlElement>("version").set_text_content(Some(&format!("v{}", boot.version)));
    let demo = boot.demo.clone();
    let ui: Shared = Rc::new(RefCell::new(Ui { prefs: boot.prefs.clone(), boot }));
    show_provider(&ui.borrow());
    render(&View::default());
    wire(&ui);
    wire_android();
    if let Some(text) = demo {
        by_id::<HtmlTextAreaElement>("text").set_value(&text);
        run_check(ui).await;
    }
    Ok(())
}

#[wasm_bindgen(start)]
pub fn main() {
    std::panic::set_hook(Box::new(|info| web_sys::console::error_1(&info.to_string().into())));
    wasm_bindgen_futures::spawn_local(async {
        if let Err(e) = start().await {
            status(&e, true);
        }
    });
}
