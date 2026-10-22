//! The text of a field as one string, and which part of it goes to the provider. Offsets are
//! UTF-16 units (utf16.rs): the caret and the DOM ranges count that way. A contenteditable's text
//! is its text nodes in order, with a synthetic "\n" before each block that follows some text; that
//! "\n" belongs to no node, so a span over it has no Range (`span_ends`).

pub const MAX_CHARS: usize = 4000;
const NL: u16 = b'\n' as u16;

/// What goes to the provider and where it starts in the field's text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sent {
    pub text: String,
    pub offset: u32,
}

/// A text node of a contenteditable in the model: where it starts in the text and how long it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub start: u32,
    pub len: u32,
}

/// Where `[start, end)` of a text of `len` units begins and ends among its nodes: (node, offset in
/// it) for each end. `None` out of bounds, or when some position belongs to no node.
pub fn span_ends(spans: &[Span], len: u32, start: u32, end: u32) -> Option<((usize, u32), (usize, u32))> {
    if end <= start || end > len {
        return None;
    }
    let find = |pos: u32, at_end: bool| {
        spans.iter().position(|s| {
            let after = s.start + s.len;
            (if at_end { pos > s.start } else { pos >= s.start }) && pos <= after && (at_end || pos < after)
        })
    };
    let a = find(start, false)?;
    let b = find(end, true)?;
    let covered: u32 = spans.iter().map(|s| end.min(s.start + s.len).saturating_sub(start.max(s.start))).sum();
    (covered == end - start).then_some(((a, start - spans[a].start), (b, end - spans[b].start)))
}

/// The whole text when it fits in MAX_CHARS; else the paragraph (line) with the caret, windowed to
/// MAX_CHARS around the caret when the paragraph itself is longer.
pub fn select_for_sending(text: &str, caret: u32) -> Sent {
    let units: Vec<u16> = text.encode_utf16().collect();
    let len = units.len();
    if len <= MAX_CHARS {
        return Sent { text: text.to_string(), offset: 0 };
    }
    let c = (caret as usize).min(len);
    let start = units[..c].iter().rposition(|&u| u == NL).map_or(0, |i| i + 1);
    let end = units[c..].iter().position(|&u| u == NL).map_or(len, |i| i + c);
    // shortcut: a window cut inside a surrogate pair sends U+FFFD for its half; JS sent the lone
    // surrogate. Snap the window to a character boundary if a model ever trips on it.
    let slice = |a: usize, b: usize| Sent { text: String::from_utf16_lossy(&units[a..b]), offset: a as u32 };
    if end - start <= MAX_CHARS {
        return slice(start, end);
    }
    let from = (c as i64 - MAX_CHARS as i64 / 2).min((end - MAX_CHARS) as i64).max(start as i64) as usize;
    slice(from, from + MAX_CHARS)
}

/// Whitespace-separated words.
pub fn word_count(text: &str) -> usize {
    text.split_whitespace().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sends_the_whole_text_when_it_fits() {
        assert_eq!(select_for_sending(&"a".repeat(4000), 10), Sent { text: "a".repeat(4000), offset: 0 });
    }

    #[test]
    fn sends_the_paragraph_containing_the_caret() {
        let p1 = "a".repeat(1500);
        let p2 = "b".repeat(2000);
        let p3 = "c".repeat(1500);
        let text = [p1.as_str(), p2.as_str(), p3.as_str()].join("\n");
        assert!(text.len() > 4000);
        let start = p1.len() as u32 + 1;
        assert_eq!(select_for_sending(&text, start + 10), Sent { text: p2, offset: start });
    }

    #[test]
    fn windows_a_paragraph_over_4000_around_the_caret() {
        let text = "x".repeat(10000);
        assert_eq!(select_for_sending(&text, 5000), Sent { text: text[3000..7000].into(), offset: 3000 });
        assert_eq!(select_for_sending(&text, 100), Sent { text: text[0..4000].into(), offset: 0 });
        assert_eq!(select_for_sending(&text, 9900), Sent { text: text[6000..10000].into(), offset: 6000 });
    }

    #[test]
    fn with_the_caret_at_0_sends_the_first_paragraph_even_when_it_is_empty() {
        let text = format!("\n{}", "a ".repeat(2500));
        assert_eq!(select_for_sending(&text, 0), Sent { text: String::new(), offset: 0 });
    }

    #[test]
    fn sends_a_short_paragraph_between_long_ones() {
        let text = format!("{}\nc d\n{}", "a ".repeat(1500), "e ".repeat(1500));
        let at = text.find("c d").unwrap() as u32;
        assert_eq!(select_for_sending(&text, at + 1), Sent { text: "c d".into(), offset: at });
    }

    #[test]
    fn windows_inside_a_later_paragraph() {
        let text = format!("a\n{}", "y".repeat(6000));
        let r = select_for_sending(&text, 2 + 100);
        assert_eq!(r.offset, 2);
        assert_eq!(r.text, text[2..4002]);
    }

    #[test]
    fn offsets_are_utf16_units() {
        // 2001 emoji = 4002 units: over the cap, and the caret counts the pairs.
        let text = format!("{}\nfim", "😀".repeat(2001));
        let r = select_for_sending(&text, 4003);
        assert_eq!(r, Sent { text: "fim".into(), offset: 4003 });
        let r = select_for_sending(&text, 10);
        assert_eq!(r.offset, 0);
        assert_eq!(r.text.encode_utf16().count(), 4000);
    }

    #[test]
    fn counts_whitespace_separated_words() {
        assert_eq!(word_count(""), 0);
        assert_eq!(word_count("  um  dois\ntrês "), 3);
    }

    fn spans(s: &[(u32, u32)]) -> Vec<Span> {
        s.iter().map(|&(start, len)| Span { start, len }).collect()
    }

    #[test]
    fn span_ends_spans_several_text_nodes() {
        // <p>um <b>dois</b> três</p>
        let s = spans(&[(0, 3), (3, 4), (7, 5)]);
        assert_eq!(span_ends(&s, 12, 1, 9), Some(((0, 1), (2, 2))));
        // <p>😀 olá mundo</p>: one node, UTF-16 offsets
        assert_eq!(span_ends(&spans(&[(0, 12)]), 12, 7, 12), Some(((0, 7), (0, 12))));
    }

    #[test]
    fn span_ends_is_none_across_a_block_separator() {
        // <div>a b</div><div>c d</div>: "a b\nc d", the "\n" in no node
        let s = spans(&[(0, 3), (4, 3)]);
        assert_eq!(span_ends(&s, 7, 2, 5), None);
        assert_eq!(span_ends(&s, 7, 3, 4), None);
        assert_eq!(span_ends(&s, 7, 4, 7), Some(((1, 0), (1, 3))));
    }

    #[test]
    fn span_ends_is_none_out_of_bounds() {
        let s = spans(&[(0, 2)]);
        assert_eq!(span_ends(&s, 2, 5, 9), None);
        assert_eq!(span_ends(&s, 2, 1, 1), None);
        assert_eq!(span_ends(&s, 2, 0, 3), None);
    }
}

#[cfg(target_arch = "wasm32")]
mod browser {
    use super::Span;
    use crate::content::same;
    use crate::utf16::len16;
    use wasm_bindgen::JsCast;
    use web_sys::{Element, HtmlElement, HtmlInputElement, HtmlTextAreaElement, Node, Range, Text};

    const QUOTED: &str = "blockquote, .gmail_quote";
    // Elements that start a new line.
    const BREAKS: &str = "div, p, li, h1, h2, h3, h4, h5, h6, pre, tr, br";

    /// A field's text, with the node and span of each piece when it is a contenteditable.
    pub struct TextModel {
        pub text: String,
        /// `text` in UTF-16 units.
        pub len: u32,
        pub spans: Vec<Span>,
        pub nodes: Vec<Text>,
    }

    /// A textarea or an input: a field with a `value`, no text nodes.
    pub fn is_field(el: &HtmlElement) -> bool {
        el.is_instance_of::<HtmlTextAreaElement>() || el.is_instance_of::<HtmlInputElement>()
    }

    /// The value of a textarea or an input; `None` for any other element.
    pub fn field_value(el: &HtmlElement) -> Option<String> {
        if let Some(t) = el.dyn_ref::<HtmlTextAreaElement>() {
            return Some(t.value());
        }
        el.dyn_ref::<HtmlInputElement>().map(HtmlInputElement::value)
    }

    pub fn text_model(root: &HtmlElement) -> TextModel {
        let mut m = TextModel { text: String::new(), len: 0, spans: vec![], nodes: vec![] };
        match field_value(root) {
            Some(v) => {
                m.len = len16(&v);
                m.text = v;
            }
            None => walk(root, &mut m),
        }
        m
    }

    // A quoted subtree is left out, a block after some text adds "\n", every text node is taken.
    fn walk(parent: &Node, m: &mut TextModel) {
        let mut next = parent.first_child();
        while let Some(n) = next {
            next = n.next_sibling();
            if let Some(t) = n.dyn_ref::<Text>() {
                let data = t.text_content().unwrap_or_default();
                let len = len16(&data);
                m.spans.push(Span { start: m.len, len });
                m.nodes.push(t.clone());
                m.text.push_str(&data);
                m.len += len;
            } else if let Some(e) = n.dyn_ref::<Element>() {
                if e.matches(QUOTED).unwrap_or(false) {
                    continue;
                }
                if !m.text.is_empty() && e.matches(BREAKS).unwrap_or(false) {
                    m.text.push('\n');
                    m.len += 1;
                }
                walk(&n, m);
            }
        }
    }

    /// Offset of the selection focus within the model; 0 when outside it.
    pub fn caret_offset(root: &HtmlElement, m: &TextModel) -> u32 {
        if let Some(t) = root.dyn_ref::<HtmlTextAreaElement>() {
            return t.selection_start().ok().flatten().unwrap_or(0);
        }
        if let Some(i) = root.dyn_ref::<HtmlInputElement>() {
            return i.selection_start().ok().flatten().unwrap_or(0);
        }
        let Some(sel) = root.owner_document().and_then(|d| d.get_selection().ok().flatten()) else { return 0 };
        let Some(focus) = sel.focus_node() else { return 0 };
        match m.nodes.iter().position(|n| same(n, &focus)) {
            Some(i) => m.spans[i].start + sel.focus_offset(),
            None => 0,
        }
    }

    /// A Range over `[start, end)` of the model; `None` when it would cross a synthetic "\n".
    pub fn range_for(m: &TextModel, start: u32, end: u32) -> Option<Range> {
        let ((a, at_a), (b, at_b)) = super::span_ends(&m.spans, m.len, start, end)?;
        let r = m.nodes[a].owner_document()?.create_range().ok()?;
        r.set_start(&m.nodes[a], at_a).ok()?;
        r.set_end(&m.nodes[b], at_b).ok()?;
        Some(r)
    }
}

#[cfg(target_arch = "wasm32")]
pub use browser::{TextModel, caret_offset, field_value, is_field, range_for, text_model};
