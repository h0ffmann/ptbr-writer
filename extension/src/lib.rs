//! The ptbr-writer browser extension in Rust. The pure modules (types, rules, suggestions,
//! decisions, settings, hosts, providers, background's check, content::text) build and test on the
//! host; the browser side (`chrome`, the wasm parts of settings, sites, providers, background) only
//! on wasm32. One wasm binary, four entry points, one per extension context.

pub mod background;
pub mod decisions;
pub mod hosts;
pub mod popup;
pub mod provider_fields;
pub mod providers;
pub mod rules;
pub mod settings;
pub mod sites;
pub mod skills;
pub mod suggestions;
pub mod types;
pub mod utf16;

pub mod content;

#[cfg(target_arch = "wasm32")]
pub mod chrome;
#[cfg(target_arch = "wasm32")]
pub mod dom;
#[cfg(target_arch = "wasm32")]
pub mod options;

#[cfg(test)]
pub(crate) mod test_util;

#[cfg(target_arch = "wasm32")]
mod entry {
    use wasm_bindgen::JsValue;
    use wasm_bindgen::prelude::wasm_bindgen;

    // A panic's message goes to the console before the abort; console_error_panic_hook is not needed.
    fn install_panic_hook() {
        std::panic::set_hook(Box::new(|info| web_sys::console::error_1(&info.to_string().into())));
    }

    /// The `#app` of options.html and popup.html.
    fn app() -> web_sys::HtmlElement {
        use wasm_bindgen::JsCast;
        crate::dom::document().get_element_by_id("app").expect("#app").unchecked_into()
    }

    /// Once the wasm is loaded; background.js owns the listeners (see `handle_message`).
    #[wasm_bindgen]
    pub fn start_background() {
        install_panic_hook();
    }

    #[wasm_bindgen]
    pub async fn handle_message(msg: JsValue, sender: JsValue) -> Result<JsValue, JsValue> {
        crate::background::handle_message(msg, sender).await
    }

    #[wasm_bindgen]
    pub async fn sync_sites() {
        crate::background::sync_sites_now().await;
    }

    #[wasm_bindgen]
    pub fn start_content() {
        install_panic_hook();
        crate::content::controller::start();
    }

    #[wasm_bindgen]
    pub fn start_options() {
        install_panic_hook();
        crate::dom::spawn(crate::options::render(app()));
    }

    #[wasm_bindgen]
    pub fn start_popup() {
        install_panic_hook();
        crate::dom::spawn(crate::popup::render(app()));
    }
}
