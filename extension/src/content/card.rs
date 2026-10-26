//! What a suggestion's card says and offers; the overlay draws it.

use crate::never_fix::{NEVER_FIX_MAX_WORDS, word_count};
use crate::types::{Located, Severity};
use std::collections::BTreeMap;

/// The first line of a card: `PTW-15 · Pontuação`, or `Reescrita` for a sentence rewritten
/// (PTW-17). A local provider adds to the code (`PTW-3 (83%)`, `PTW-3 · estilo conciso`): the
/// title is the code's.
pub fn card_heading(l: &Located, titles: &BTreeMap<String, String>) -> String {
    let s = &l.suggestion;
    if s.severity == Severity::Aviso && s.rule == "PTW-17" && !s.detect_only {
        return "Reescrita".into();
    }
    let letters = s.rule.find(|c: char| !c.is_ascii_uppercase()).unwrap_or(s.rule.len());
    let digits =
        s.rule[letters..].strip_prefix('-').map_or(0, |r| r.find(|c: char| !c.is_ascii_digit()).unwrap_or(r.len()));
    let code = if letters > 0 && digits > 0 { &s.rule[..letters + 1 + digits] } else { s.rule.as_str() };
    match titles.get(code) {
        Some(title) => format!("{} · {title}", s.rule),
        None => s.rule.clone(),
    }
}

/// "Sempre ignorar" keeps the original as written; a detection changes nothing, so the list has
/// nothing to keep there.
pub fn offers_never_fix(l: &Located) -> bool {
    !l.suggestion.detect_only && word_count(&l.suggestion.original) <= NEVER_FIX_MAX_WORDS
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Suggestion;

    fn located(rule: &str, severity: Severity, original: &str) -> Located {
        let suggestion = Suggestion {
            original: original.into(),
            replacement: "x".into(),
            detect_only: false,
            rule: rule.into(),
            severity,
            occurrence: 1,
        };
        Located { suggestion, start: 0, end: 1 }
    }

    fn titles(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs.iter().map(|(c, t)| (c.to_string(), t.to_string())).collect()
    }

    #[test]
    fn shows_the_code_and_the_title_of_an_error() {
        let l = located("PTW-15", Severity::Erro, "a");
        assert_eq!(card_heading(&l, &titles(&[("PTW-15", "Pontuação")])), "PTW-15 · Pontuação");
    }

    #[test]
    fn shows_only_the_code_when_there_is_no_title() {
        assert_eq!(card_heading(&located("PTW-15", Severity::Erro, "a"), &titles(&[])), "PTW-15");
    }

    #[test]
    fn shows_reescrita_on_a_ptw_17_warning() {
        let l = located("PTW-17", Severity::Aviso, "a");
        assert_eq!(card_heading(&l, &titles(&[("PTW-17", "Frase reorganizada")])), "Reescrita");
    }

    #[test]
    fn shows_the_code_and_the_title_of_a_warning_from_another_rule() {
        assert_eq!(
            card_heading(&located("PTW-2", Severity::Aviso, "a"), &titles(&[("PTW-2", "Algo")])),
            "PTW-2 · Algo"
        );
    }

    #[test]
    fn a_local_providers_suffix_keeps_the_codes_title() {
        let t = titles(&[("PTW-3", "Três")]);
        assert_eq!(card_heading(&located("PTW-3 (83%)", Severity::Aviso, "a"), &t), "PTW-3 (83%) · Três");
        assert_eq!(
            card_heading(&located("PTW-3 · estilo conciso", Severity::Erro, "a"), &t),
            "PTW-3 · estilo conciso · Três"
        );
    }

    #[test]
    fn sempre_ignorar_up_to_3_words_and_not_on_a_detection() {
        assert!(offers_never_fix(&located("PTW-1", Severity::Erro, "um dois três")));
        assert!(!offers_never_fix(&located("PTW-1", Severity::Erro, "um dois três quatro")));
        let mut detect = located("PTW-1", Severity::Aviso, "um");
        detect.suggestion.detect_only = true;
        assert!(!offers_never_fix(&detect));
    }
}
