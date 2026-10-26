//! The "never fix" list: phrases of the person's that a correction must keep as written.

pub const NEVER_FIX_MAX: usize = 200;
pub const NEVER_FIX_MAX_WORDS: usize = 3;

/// The words of a phrase, as `phrase.trim().split(/\s+/)` counts them.
pub fn word_count(phrase: &str) -> usize {
    phrase.split_whitespace().count()
}

/// Adds a phrase to the list: trimmed, 1 to 3 words, no repeats (case-insensitive), and only the
/// NEVER_FIX_MAX most recent entries stay.
pub fn add_never_fix(list: &[String], phrase: &str) -> Vec<String> {
    let p = phrase.trim();
    let mut out = list.to_vec();
    if p.is_empty() || word_count(p) > NEVER_FIX_MAX_WORDS {
        return out;
    }
    let low = p.to_lowercase();
    if list.iter().any(|e| e.to_lowercase() == low) {
        return out;
    }
    out.push(p.to_string());
    let extra = out.len().saturating_sub(NEVER_FIX_MAX);
    out.drain(..extra);
    out
}

/// Letters and digits (with accents) are word characters.
fn is_word(c: char) -> bool {
    c.is_alphanumeric()
}

fn same_char(a: char, b: char, ignore_case: bool) -> bool {
    a == b || (ignore_case && a.to_lowercase().eq(b.to_lowercase()))
}

/// The whole-word matches of `entry` in `text`, left to right and apart, as the text writes them.
fn whole_words<'a>(text: &'a str, entry: &str, ignore_case: bool) -> Vec<&'a str> {
    let needle: Vec<char> = entry.chars().collect();
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut found = vec![];
    let mut i = 0;
    while i + needle.len() <= chars.len() {
        let end = i + needle.len();
        let fits = (i == 0 || !is_word(chars[i - 1].1))
            && (end == chars.len() || !is_word(chars[end].1))
            && needle.iter().zip(&chars[i..end]).all(|(n, (_, c))| same_char(*n, *c, ignore_case));
        if fits {
            let to = chars.get(end).map_or(text.len(), |(b, _)| *b);
            found.push(&text[chars[i].0..to]);
            i = end;
        } else {
            i += 1;
        }
    }
    found
}

/// Whole-word match, ignoring case but not accents.
pub fn contains_entry(text: &str, entry: &str) -> bool {
    let e = entry.trim();
    !e.is_empty() && !whole_words(text, e, true).is_empty()
}

/// True when `original` holds the entry (any case) and `replacement` does not keep each matched
/// text exactly as written, case included, as a whole word: `maria` -> `Maria` breaks it.
pub fn breaks_entry(original: &str, replacement: &str, entry: &str) -> bool {
    let e = entry.trim();
    !e.is_empty() && whole_words(original, e, true).into_iter().any(|f| whole_words(replacement, f, false).is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn add_trims_the_ends() {
        assert_eq!(add_never_fix(&[], "  Kafka "), ["Kafka"]);
    }

    #[test]
    fn add_does_not_repeat_ignoring_case() {
        assert_eq!(add_never_fix(&list(&["Kafka"]), "kafka"), ["Kafka"]);
    }

    #[test]
    fn add_ignores_empty_input() {
        assert!(add_never_fix(&[], "   ").is_empty());
    }

    #[test]
    fn add_ignores_input_with_more_than_3_words() {
        assert!(add_never_fix(&[], "um dois três quatro").is_empty());
        assert_eq!(add_never_fix(&[], "um dois três"), ["um dois três"]);
    }

    #[test]
    fn add_keeps_the_200_most_recent() {
        let full: Vec<String> = (0..NEVER_FIX_MAX).map(|i| format!("w{i}")).collect();
        let out = add_never_fix(&full, "novo");
        assert_eq!(out.len(), NEVER_FIX_MAX);
        assert_eq!(out[0], "w1");
        assert_eq!(out[out.len() - 1], "novo");
    }

    #[test]
    fn contains_whole_word_ignoring_case() {
        assert!(contains_entry("use a api nova", "API"));
    }

    #[test]
    fn contains_does_not_match_inside_a_word() {
        assert!(!contains_entry("a capital", "api"));
    }

    #[test]
    fn contains_tells_accents_apart() {
        assert!(!contains_entry("ação rápida", "acao"));
        assert!(contains_entry("ação rápida", "ação"));
    }

    #[test]
    fn a_change_of_case_alone_breaks_the_entry() {
        assert!(breaks_entry("maria", "Maria", "maria"));
        assert!(breaks_entry("use o github", "use o GitHub", "github"));
    }

    #[test]
    fn keeps_the_text_as_written_whatever_the_case_of_the_entry() {
        assert!(!breaks_entry("maria", "maria,", "maria"));
        assert!(!breaks_entry("o GitHub caiu", "o GitHub caiu.", "github"));
    }

    #[test]
    fn needs_a_whole_word_in_the_replacement() {
        assert!(breaks_entry("api", "apis", "api"));
    }

    #[test]
    fn an_original_without_the_entry_breaks_nothing() {
        assert!(!breaks_entry("a capital", "A capital", "api"));
    }
}
