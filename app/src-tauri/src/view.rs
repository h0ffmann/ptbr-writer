//! The checked text and its suggestions: the pieces the window underlines, and what Aceitar and
//! Ignorar do to them. Offsets are the extension's, UTF-16 units (ptbr_ext::utf16).

use ptbr_app_wire::{Card, Segment, View};
use ptbr_ext::rules::rule_titles;
use ptbr_ext::skills::{CIENTIFICO, SKILL};
use ptbr_ext::suggestions::{apply_to_string, drop_overlaps, locate};
use ptbr_ext::types::{Located, Suggestion};
use ptbr_ext::utf16::{len16, slice16};
use std::collections::BTreeMap;

/// The last check: the text sent and the suggestions still open.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Session {
    pub text: String,
    pub suggestions: Vec<Suggestion>,
}

impl Session {
    pub fn new(text: String, suggestions: Vec<Suggestion>) -> Self {
        Session { text, suggestions }
    }

    fn located(&self) -> Vec<Located> {
        drop_overlaps(locate(&self.text, &self.suggestions))
    }

    pub fn view(&self) -> View {
        let titles = titles();
        let located = self.located();
        let mut segments = vec![];
        let mut at = 0;
        let mut piece = |from: u32, to: u32, card: Option<usize>| {
            if let Some(text) = slice16(&self.text, from, to).filter(|t| !t.is_empty()) {
                segments.push(Segment { text: text.to_string(), card });
            }
        };
        for (id, l) in located.iter().enumerate() {
            piece(at, l.start, None);
            piece(l.start, l.end, Some(id));
            at = l.end;
        }
        piece(at, len16(&self.text), None);
        let cards = located.iter().enumerate().map(|(id, l)| card(id, &l.suggestion, &titles)).collect();
        View { text: self.text.clone(), segments, cards }
    }

    /// The card's replacement into the text; the others stay open.
    pub fn accept(&mut self, id: usize) {
        let Some(l) = self.located().into_iter().nth(id) else { return };
        if l.suggestion.detect_only {
            return;
        }
        self.text = apply_to_string(&self.text, &l);
        self.close(&l.suggestion);
        // The occurrence it replaced is gone: a later one of the same words is now one earlier.
        if !l.suggestion.replacement.contains(&l.suggestion.original) {
            for s in &mut self.suggestions {
                if s.original == l.suggestion.original && s.occurrence > l.suggestion.occurrence {
                    s.occurrence -= 1;
                }
            }
        }
    }

    pub fn ignore(&mut self, id: usize) {
        if let Some(l) = self.located().into_iter().nth(id) {
            self.close(&l.suggestion);
        }
    }

    fn close(&mut self, s: &Suggestion) {
        if let Some(i) = self.suggestions.iter().position(|x| x == s) {
            self.suggestions.remove(i);
        }
    }
}

/// `PTW-n` → its title, from both skills.
fn titles() -> BTreeMap<String, String> {
    let mut t = rule_titles(SKILL);
    t.extend(rule_titles(CIENTIFICO));
    t
}

fn card(id: usize, s: &Suggestion, titles: &BTreeMap<String, String>) -> Card {
    // Ollaya's style rides on the rule: `PTW-17 · estilo conciso`.
    let rule_id = s.rule.split_whitespace().next().unwrap_or_default();
    Card {
        id,
        rule: s.rule.clone(),
        title: titles.get(rule_id).cloned().unwrap_or_default(),
        severity: s.severity.as_str().into(),
        original: s.original.clone(),
        replacement: s.replacement.clone(),
        detect_only: s.detect_only,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ptbr_ext::types::Severity;

    fn s(original: &str, replacement: &str, occurrence: u32) -> Suggestion {
        Suggestion {
            original: original.into(),
            replacement: replacement.into(),
            detect_only: false,
            rule: "PTW-1".into(),
            severity: Severity::Erro,
            occurrence,
        }
    }

    fn pieces(v: &View) -> Vec<(&str, Option<usize>)> {
        v.segments.iter().map(|x| (x.text.as_str(), x.card)).collect()
    }

    #[test]
    fn segments_cover_the_text_in_order() {
        let session = Session::new(
            "Foi decidido que 👋 vai.".into(),
            vec![s("vai", "irá", 1), s("Foi decidido", "Decidimos", 1)],
        );
        let v = session.view();
        assert_eq!(pieces(&v), vec![("Foi decidido", Some(0)), (" que 👋 ", None), ("vai", Some(1)), (".", None)]);
        assert_eq!(v.cards[0].original, "Foi decidido");
        assert_eq!(v.cards[0].title, rule_titles(SKILL)["PTW-1"]);
        assert_eq!(v.segments.iter().map(|x| x.text.as_str()).collect::<String>(), v.text);
    }

    #[test]
    fn a_suggestion_not_in_the_text_has_no_card() {
        let v = Session::new("abc".into(), vec![s("xyz", "w", 1)]).view();
        assert!(v.cards.is_empty());
        assert_eq!(pieces(&v), vec![("abc", None)]);
    }

    #[test]
    fn accept_replaces_and_shifts_later_occurrences() {
        let mut session = Session::new("é que é que".into(), vec![s("é que", "", 1), s("é que", "", 2)]);
        session.accept(0);
        assert_eq!(session.text, " é que");
        assert_eq!(session.suggestions, vec![s("é que", "", 1)]);
        assert_eq!(session.view().cards.len(), 1);
        session.accept(0);
        assert_eq!(session.text, " ");
    }

    #[test]
    fn ignore_closes_one_card_and_keeps_the_text() {
        let mut session = Session::new("a b".into(), vec![s("a", "x", 1), s("b", "y", 1)]);
        session.ignore(0);
        assert_eq!(session.text, "a b");
        assert_eq!(session.view().cards[0].original, "b");
        session.ignore(9);
        assert_eq!(session.suggestions.len(), 1);
    }

    #[test]
    fn a_detection_cannot_be_accepted() {
        let mut d = s("a", "", 1);
        d.detect_only = true;
        let mut session = Session::new("a b".into(), vec![d]);
        session.accept(0);
        assert_eq!(session.text, "a b");
        assert!(session.view().cards[0].detect_only);
    }
}
