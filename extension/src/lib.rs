//! The ptbr-writer browser extension in Rust. The pure modules (types, rules, suggestions,
//! decisions, settings, hosts, providers, background's check, content::text) build and test on the
//! host; the browser side (`chrome`, the wasm parts of settings, sites, providers, background) only
//! on wasm32. One wasm binary, four entry points, one per extension context.

pub mod background;
pub mod decisions;
pub mod hosts;
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

#[cfg(test)]
pub(crate) mod test_util;

#[cfg(target_arch = "wasm32")]
mod entry {
    use wasm_bindgen::prelude::wasm_bindgen;

    // A panic's message goes to the console before the abort; console_error_panic_hook is not needed.
    fn install_panic_hook() {
        std::panic::set_hook(Box::new(|info| web_sys::console::error_1(&info.to_string().into())));
    }

    #[wasm_bindgen]
    pub fn start_background() {
        install_panic_hook();
    }

    #[wasm_bindgen]
    pub fn start_content() {
        install_panic_hook();
    }

    #[wasm_bindgen]
    pub fn start_options() {
        install_panic_hook();
    }

    #[wasm_bindgen]
    pub fn start_popup() {
        install_panic_hook();
    }
}
