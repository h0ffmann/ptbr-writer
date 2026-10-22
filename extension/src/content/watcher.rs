//! A check of a field: what it sends, and what its reply means once it arrives. The decisions are
//! plain Rust (`plan`, `outcome`, `own_edit`); the browser part, `Watcher`, puts them behind the
//! field's input events, a debounce timer and the message to the background.

use crate::content::text::{Sent, select_for_sending, word_count};
use crate::suggestions::{drop_overlaps, locate, span};
use crate::types::{CheckError, CheckResult, Located};

pub const MIN_WORDS: usize = 3;
pub const DELAY_MS: i32 = 1000;

/// What a field's badge shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Status {
    #[default]
    Idle,
    Checking,
    Failed(CheckError),
}

/// What a check sends for a field with `text` and the caret at `caret`; `None` skips it: a
/// password field, or fewer than MIN_WORDS words in the field or in the part that would go.
pub fn plan(text: &str, caret: u32, password: bool) -> Option<Sent> {
    if password || word_count(text) < MIN_WORDS {
        return None;
    }
    let sent = select_for_sending(text, caret);
    (word_count(&sent.text) >= MIN_WORDS).then_some(sent)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The field changed while the check was out.
    Stale,
    Failed(CheckError),
    Found {
        located: Vec<Located>,
        offset: u32,
    },
}

/// The reply to `sent`, for a field whose text was `before` when it went out and is `now`.
pub fn outcome(sent: &Sent, before: &str, now: &str, result: CheckResult) -> Outcome {
    if now != before {
        return Outcome::Stale;
    }
    match result {
        CheckResult::Err(e) => Outcome::Failed(e),
        CheckResult::Ok(suggestions) => {
            // A span across a line break cannot be shown or applied in a contenteditable, whose
            // breaks are synthetic.
            let located = locate(&sent.text, &suggestions)
                .into_iter()
                .filter(|l| !span(&sent.text, l).is_some_and(|s| s.contains('\n')))
                .collect();
            Outcome::Found { located: drop_overlaps(located), offset: sent.offset }
        }
    }
}

/// Whether an input event is the extension's own edit: the one `expected` leaves exactly the text
/// `now` gives. Either way the expectation is spent.
pub fn own_edit(expected: &mut Option<String>, now: impl FnOnce() -> String) -> bool {
    expected.take().is_some_and(|e| e == now())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Severity, Suggestion};

    fn suggestion(original: &str, replacement: &str) -> Suggestion {
        Suggestion {
            original: original.into(),
            replacement: replacement.into(),
            detect_only: false,
            rule: "R".into(),
            severity: Severity::Aviso,
            occurrence: 1,
        }
    }

    fn sent(text: &str) -> Sent {
        Sent { text: text.into(), offset: 0 }
    }

    #[test]
    fn ignores_passwords_and_texts_under_3_words() {
        assert_eq!(plan("uma senha bem longa x", 0, true), None);
        assert_eq!(plan("dois termos", 0, false), None);
        assert_eq!(plan("um dois três abc", 16, false), Some(sent("um dois três abc")));
    }

    #[test]
    fn does_not_send_an_empty_paragraph_of_a_long_text() {
        let p1 = "a ".repeat(2100);
        let text = format!("{p1}\n\n{}", "b ".repeat(2100));
        assert_eq!(plan(&text, p1.len() as u32 + 1, false), None);
    }

    #[test]
    fn discards_a_response_when_the_field_changed_meanwhile() {
        let s = sent("a casa é bonito");
        let ok = CheckResult::Ok(vec![suggestion("casa", "casas")]);
        assert_eq!(outcome(&s, &s.text, "outro texto bem diferente", ok), Outcome::Stale);
    }

    #[test]
    fn applies_a_fresh_response_as_located_suggestions_with_the_offset() {
        let s = Sent { text: "a casa é bonita".into(), offset: 7 };
        let ok = CheckResult::Ok(vec![suggestion("casa", "lar")]);
        let Outcome::Found { located, offset } = outcome(&s, &s.text, &s.text, ok) else { panic!() };
        assert_eq!(offset, 7);
        assert_eq!(located.len(), 1);
        assert_eq!((located[0].start, located[0].end), (2, 6));
        assert_eq!(located[0].suggestion.original, "casa");
    }

    #[test]
    fn drops_a_suggestion_that_spans_two_lines() {
        let s = sent("um\ndois três");
        let ok = CheckResult::Ok(vec![suggestion("um\ndois", "um dois"), suggestion("dois", "2")]);
        let Outcome::Found { located, .. } = outcome(&s, &s.text, &s.text, ok) else { panic!() };
        assert_eq!(located.iter().map(|l| l.suggestion.original.as_str()).collect::<Vec<_>>(), ["dois"]);
    }

    #[test]
    fn reports_an_error_state() {
        let s = sent("a casa é bonita");
        assert_eq!(
            outcome(&s, &s.text, &s.text, CheckResult::Err(CheckError::Auth)),
            Outcome::Failed(CheckError::Auth)
        );
    }

    #[test]
    fn an_expected_edit_is_the_extensions_own_and_is_spent() {
        let mut expected = Some("A equipe decidiu que o deploy vai ser na sexta.".to_string());
        assert!(own_edit(&mut expected, || "A equipe decidiu que o deploy vai ser na sexta.".into()));
        assert_eq!(expected, None);
        // typing after it: no expectation left
        assert!(!own_edit(&mut expected, || "A equipe decidiu que o deploy vai ser na sexta. Mais".into()));
    }

    #[test]
    fn an_edit_different_from_the_expected_one_is_typing() {
        let mut expected = Some("um dois três quatro".to_string());
        assert!(!own_edit(&mut expected, || "um dois três outra coisa".into()));
        assert_eq!(expected, None);
        assert!(!own_edit(&mut None, || unreachable!("no expectation: the text is not read")));
    }
}

#[cfg(target_arch = "wasm32")]
mod browser {
    use super::{DELAY_MS, Outcome, Status, outcome, own_edit, plan};
    use crate::chrome;
    use crate::content::text::{caret_offset, text_model};
    use crate::content::{ElMap, Timer};
    use crate::types::{CheckError, CheckRequest, CheckResult, Level, Located, Message};
    use std::cell::RefCell;
    use std::rc::{Rc, Weak};
    use wasm_bindgen::JsCast;
    use wasm_bindgen::prelude::Closure;
    use wasm_bindgen_futures::spawn_local;
    use web_sys::{HtmlElement, HtmlInputElement};

    /// What the watcher tells its owner. Called with no borrow of the watcher held: a hook may
    /// call back into it (`expect_edit`).
    pub struct Hooks {
        pub on_result: OnResult,
        pub on_state: OnState,
    }

    /// The located suggestions of a field and the offset of the text they are in.
    pub type OnResult = Box<dyn Fn(&HtmlElement, Vec<Located>, u32)>;
    pub type OnState = Box<dyn Fn(&HtmlElement, Status)>;

    /// Checks the attached fields: an input event clears their suggestions and, DELAY_MS after
    /// the last one, sends the text. Dropping it detaches every field.
    pub struct Watcher(Rc<RefCell<Inner>>);

    struct Inner {
        level: Level,
        seq: u32,
        fields: ElMap<Field>,
        hooks: Rc<Hooks>,
    }

    struct Field {
        key: String,
        on_input: Closure<dyn FnMut()>,
        timer: Option<Timer>,
        // The text an accepted suggestion will leave in the field: that edit is the extension's
        // own, so it neither clears the other suggestions nor calls the API again.
        expected: Option<String>,
    }

    impl Watcher {
        pub fn new(level: Level, hooks: Hooks) -> Watcher {
            Watcher(Rc::new(RefCell::new(Inner { level, seq: 0, fields: ElMap::new(), hooks: Rc::new(hooks) })))
        }

        pub fn attach(&self, el: &HtmlElement) {
            let mut inner = self.0.borrow_mut();
            if inner.fields.contains(el) {
                return;
            }
            inner.seq += 1;
            let key = format!("f{}", inner.seq);
            let weak = Rc::downgrade(&self.0);
            let target = el.clone();
            let on_input = Closure::new(move || on_input(&weak, &target));
            el.add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref()).expect("addEventListener");
            inner.fields.insert(el.clone(), Field { key, on_input, timer: None, expected: None });
        }

        /// The next input that leaves exactly `text` in the field is the extension's own edit;
        /// `None` forgets it.
        pub fn expect_edit(&self, el: &HtmlElement, text: Option<String>) {
            if let Some(f) = self.0.borrow_mut().fields.get_mut(el) {
                f.expected = text;
            }
        }

        pub fn detach(&self, el: &HtmlElement) {
            if let Some(f) = self.0.borrow_mut().fields.remove(el) {
                detach(el, &f);
            }
        }
    }

    impl Drop for Watcher {
        fn drop(&mut self) {
            for (el, f) in self.0.borrow_mut().fields.drain() {
                detach(&el, &f);
            }
        }
    }

    // The listener goes before the field (and its timer) is dropped.
    fn detach(el: &HtmlElement, f: &Field) {
        let _ = el.remove_event_listener_with_callback("input", f.on_input.as_ref().unchecked_ref());
    }

    fn on_input(weak: &Weak<RefCell<Inner>>, el: &HtmlElement) {
        let Some(inner) = weak.upgrade() else { return };
        let hooks = {
            let mut i = inner.borrow_mut();
            let Some(f) = i.fields.get_mut(el) else { return };
            if own_edit(&mut f.expected, || text_model(el).text) {
                return;
            }
            i.hooks.clone()
        };
        (hooks.on_result)(el, vec![], 0);
        let mut i = inner.borrow_mut();
        let Some(f) = i.fields.get_mut(el) else { return };
        let (weak, target) = (weak.clone(), el.clone());
        // Replacing the timer clears the pending one.
        f.timer = Some(Timer::new(DELAY_MS, move || spawn_local(check(weak.clone(), target.clone()))));
    }

    async fn check(weak: Weak<RefCell<Inner>>, el: HtmlElement) {
        let Some(inner) = weak.upgrade() else { return };
        let (key, level, hooks) = {
            let mut i = inner.borrow_mut();
            let (level, hooks) = (i.level, i.hooks.clone());
            let Some(f) = i.fields.get_mut(&el) else { return };
            f.timer = None;
            (f.key.clone(), level, hooks)
        };
        drop(inner);
        let m = text_model(&el);
        let password = el.dyn_ref::<HtmlInputElement>().is_some_and(|i| i.type_() == "password");
        let Some(sent) = plan(&m.text, caret_offset(&el, &m), password) else {
            return (hooks.on_state)(&el, Status::Idle);
        };
        (hooks.on_state)(&el, Status::Checking);
        let request = Message::Check(CheckRequest { key, text: sent.text.clone(), level });
        // No answer (the worker gone) is the network; an answer that is not a result is the provider.
        let result = match chrome::send_message_raw(&request).await {
            Ok(reply) => chrome::from_js(reply).unwrap_or(CheckResult::Err(CheckError::Provider)),
            Err(_) => CheckResult::Err(CheckError::Network),
        };
        let Some(inner) = weak.upgrade() else { return };
        if !inner.borrow().fields.contains(&el) {
            return;
        }
        match outcome(&sent, &m.text, &text_model(&el).text, result) {
            Outcome::Stale => (hooks.on_state)(&el, Status::Idle),
            Outcome::Failed(e) => (hooks.on_state)(&el, Status::Failed(e)),
            Outcome::Found { located, offset } => {
                (hooks.on_result)(&el, located, offset);
                (hooks.on_state)(&el, Status::Idle);
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use browser::{Hooks, OnResult, OnState, Watcher};
