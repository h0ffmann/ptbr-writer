//! The text of a field as one string, and which part of it goes to the provider. Offsets are
//! UTF-16 units (utf16.rs): the caret and the DOM ranges count that way.

pub const MAX_CHARS: usize = 4000;
const NL: u16 = b'\n' as u16;

/// What goes to the provider and where it starts in the field's text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sent {
    pub text: String,
    pub offset: u32,
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
}
