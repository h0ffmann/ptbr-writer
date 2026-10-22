//! The content script: what runs on a page. `text` holds the math over the field's text and
//! `watcher` the decisions of a check; their DOM parts, `overlay`, `apply` and `controller` come
//! with the content entry point, wasm only.

pub mod text;
pub mod watcher;

#[cfg(target_arch = "wasm32")]
pub mod apply;
#[cfg(target_arch = "wasm32")]
pub mod controller;
#[cfg(target_arch = "wasm32")]
pub mod overlay;

#[cfg(target_arch = "wasm32")]
mod browser {
    use wasm_bindgen::prelude::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::HtmlElement;

    /// `a === b`.
    pub fn same(a: &impl AsRef<JsValue>, b: &impl AsRef<JsValue>) -> bool {
        a.as_ref() == b.as_ref()
    }

    /// A map keyed by an element, by identity. A page has a few fields: a list is enough, and
    /// a lookup allocates nothing.
    #[derive(Default)]
    pub struct ElMap<V>(Vec<(HtmlElement, V)>);

    impl<V> ElMap<V> {
        pub const fn new() -> Self {
            Self(Vec::new())
        }

        pub fn position(&self, el: &HtmlElement) -> Option<usize> {
            self.0.iter().position(|(e, _)| same(e, el))
        }

        pub fn contains(&self, el: &HtmlElement) -> bool {
            self.position(el).is_some()
        }

        pub fn get(&self, el: &HtmlElement) -> Option<&V> {
            self.position(el).map(|i| &self.0[i].1)
        }

        pub fn get_mut(&mut self, el: &HtmlElement) -> Option<&mut V> {
            self.position(el).map(|i| &mut self.0[i].1)
        }

        /// Sets the value of `el`; its index.
        pub fn insert(&mut self, el: HtmlElement, v: V) -> usize {
            match self.position(&el) {
                Some(i) => {
                    self.0[i].1 = v;
                    i
                }
                None => {
                    self.0.push((el, v));
                    self.0.len() - 1
                }
            }
        }

        pub fn remove(&mut self, el: &HtmlElement) -> Option<V> {
            self.position(el).map(|i| self.0.remove(i).1)
        }

        pub fn at(&self, i: usize) -> &(HtmlElement, V) {
            &self.0[i]
        }

        pub fn at_mut(&mut self, i: usize) -> &mut (HtmlElement, V) {
            &mut self.0[i]
        }

        pub fn len(&self) -> usize {
            self.0.len()
        }

        pub fn is_empty(&self) -> bool {
            self.0.is_empty()
        }

        pub fn clear(&mut self) {
            self.0.clear();
        }

        pub fn iter(&self) -> impl Iterator<Item = &(HtmlElement, V)> {
            self.0.iter()
        }

        pub fn retain(&mut self, mut keep: impl FnMut(&HtmlElement, &mut V) -> bool) {
            self.0.retain_mut(|(el, v)| keep(el, v));
        }

        pub fn drain(&mut self) -> impl Iterator<Item = (HtmlElement, V)> + '_ {
            self.0.drain(..)
        }
    }

    /// A `setTimeout`, cleared when dropped; its callback lives with it.
    pub struct Timer {
        handle: i32,
        _cb: Closure<dyn FnMut()>,
    }

    impl Timer {
        pub fn new(ms: i32, f: impl FnMut() + 'static) -> Timer {
            let cb = Closure::new(f);
            let handle = web_sys::window()
                .expect("a window")
                .set_timeout_with_callback_and_timeout_and_arguments_0(cb.as_ref().unchecked_ref(), ms)
                .expect("setTimeout");
            Timer { handle, _cb: cb }
        }
    }

    impl Drop for Timer {
        fn drop(&mut self) {
            if let Some(w) = web_sys::window() {
                w.clear_timeout_with_handle(self.handle);
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use browser::{ElMap, Timer, same};
