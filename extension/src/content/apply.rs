//! Replaces one suggestion in the editor.

use crate::content::text::{field_value, range_for, text_model};
use crate::types::Located;
use crate::utf16::slice16;
use wasm_bindgen::JsCast;
use web_sys::{
    ClipboardEvent, ClipboardEventInit, DataTransfer, Event, HtmlDocument, HtmlElement, HtmlInputElement,
    HtmlTextAreaElement, InputEvent, InputEventInit,
};

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
    // Editors that keep their own text state (WhatsApp Web) learn the selection from selectionchange,
    // which the browser fires later; fire it now so the paste below lands on the selected span.
    if let Ok(ev) = Event::new("selectionchange") {
        let _ = doc.dispatch_event(&ev);
    }
    if paste_into(el, replacement) {
        return true;
    }
    // An editor that does not take over paste: execCommand goes through its input pipeline (undo stack).
    doc.unchecked_into::<HtmlDocument>()
        .exec_command_with_show_ui_and_value("insertText", false, replacement)
        .unwrap_or(false)
}

// Offers the replacement as a plain-text paste. True when the editor handled it (prevented the
// default), as WhatsApp's does: it then writes the text through its own state, which a DOM insert
// would bypass and the editor would undo.
fn paste_into(el: &HtmlElement, text: &str) -> bool {
    let Ok(data) = DataTransfer::new() else { return false };
    if data.set_data("text/plain", text).is_err() {
        return false;
    }
    let init = ClipboardEventInit::new();
    init.set_clipboard_data(Some(&data));
    init.set_bubbles(true);
    init.set_cancelable(true);
    init.set_composed(true);
    let Ok(paste) = ClipboardEvent::new_with_event_init_dict("paste", &init) else { return false };
    let _ = el.dispatch_event(&paste);
    paste.default_prevented()
}
