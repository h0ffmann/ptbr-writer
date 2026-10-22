//! Replaces one suggestion in the editor.

use crate::content::text::{field_value, range_for, text_model};
use crate::types::Located;
use crate::utf16::slice16;
use wasm_bindgen::JsCast;
use web_sys::{HtmlDocument, HtmlElement, HtmlInputElement, HtmlTextAreaElement, InputEvent, InputEventInit};

/// `l` applied at `offset` into the field. False, changing nothing, when the span is no longer
/// `original`.
pub fn accept_suggestion(el: &HtmlElement, l: &Located, offset: u32) -> bool {
    let (start, end) = (l.start + offset, l.end + offset);
    let (original, replacement) = (l.suggestion.original.as_str(), l.suggestion.replacement.as_str());
    if let Some(value) = field_value(el) {
        if slice16(&value, start, end) != Some(original) {
            return false;
        }
        let set = match el.dyn_ref::<HtmlTextAreaElement>() {
            Some(t) => t.set_range_text_with_start_and_end(replacement, start, end),
            None => el.unchecked_ref::<HtmlInputElement>().set_range_text_with_start_and_end(replacement, start, end),
        };
        if set.is_err() {
            return false;
        }
        let init = InputEventInit::new();
        init.set_bubbles(true);
        if let Ok(ev) = InputEvent::new_with_event_init_dict("input", &init) {
            let _ = el.dispatch_event(&ev);
        }
        return true;
    }
    let m = text_model(el);
    if slice16(&m.text, start, end) != Some(original) {
        return false;
    }
    let Some(range) = range_for(&m, start, end) else { return false };
    let Some(doc) = el.owner_document() else { return false };
    let Some(sel) = doc.get_selection().ok().flatten() else { return false };
    let _ = el.focus();
    let _ = sel.remove_all_ranges();
    let _ = sel.add_range(&range);
    // execCommand goes through the editor's own input pipeline (undo stack, React/Lexical state).
    doc.unchecked_into::<HtmlDocument>()
        .exec_command_with_show_ui_and_value("insertText", false, replacement)
        .unwrap_or(false)
}
