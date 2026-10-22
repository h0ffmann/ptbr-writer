//! Offsets in UTF-16 code units, as the DOM (Range, selectionStart) and the TypeScript version count
//! them. Every `start`/`end` in this crate is one of these; a byte offset into a `&str` is derived
//! here and nowhere else.

/// The length of `s` in UTF-16 code units.
pub fn len16(s: &str) -> u32 {
    s.chars().map(|c| c.len_utf16() as u32).sum()
}

/// The byte offset of UTF-16 offset `at`; `None` past the end or inside a surrogate pair.
pub fn byte_at(s: &str, at: u32) -> Option<usize> {
    let mut units = 0u32;
    for (byte, c) in s.char_indices() {
        if units == at {
            return Some(byte);
        }
        if units > at {
            return None;
        }
        units += c.len_utf16() as u32;
    }
    (units == at).then_some(s.len())
}

/// `s[start..end]` in UTF-16 offsets.
pub fn slice16(s: &str, start: u32, end: u32) -> Option<&str> {
    if end < start {
        return None;
    }
    Some(&s[byte_at(s, start)?..byte_at(s, end)?])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_code_units() {
        assert_eq!(len16("abc"), 3);
        assert_eq!(len16("á ç"), 3);
        assert_eq!(len16("e\u{301}"), 2); // NFD: two units
        assert_eq!(len16("Oi 👋"), 5); // an emoji is a surrogate pair
    }

    #[test]
    fn maps_offsets_to_bytes() {
        let s = "Oi 👋 çá";
        assert_eq!(byte_at(s, 0), Some(0));
        assert_eq!(byte_at(s, 3), Some(3));
        assert_eq!(byte_at(s, 4), None); // inside the pair
        assert_eq!(byte_at(s, 5), Some(7));
        assert_eq!(byte_at(s, 6), Some(8)); // after the space: ç is two bytes, one unit
        assert_eq!(byte_at(s, 7), Some(10));
        assert_eq!(byte_at(s, 8), Some(12)); // the end
        assert_eq!(byte_at(s, 9), None);
    }

    #[test]
    fn slices() {
        assert_eq!(slice16("Oi 👋 foi", 5, 8), Some(" fo"));
        assert_eq!(slice16("açaí", 1, 3), Some("ça"));
        assert_eq!(slice16("abc", 2, 1), None);
        assert_eq!(slice16("abc", 0, 9), None);
    }
}
