//! Ollaya answers typed questions with probabilities and never writes text, so a rule broken is a
//! decision: for each sentence, "which rule does it break?", with `nenhuma` as one of the answers.
//! An answer other than `nenhuma` becomes an underline with no replacement (the card has no Aceitar).
//! The sentence is the state and each option is only `PTW-n: title`: a decision model reads a short
//! question and short options (docs/SURVEY_DECISION_MODEL.md).

use crate::rules::{SectionError, section, style_description};
use crate::types::{Level, Severity, Style, Suggestion};
use serde_json::{Map, Value, json};
use std::collections::HashMap;

const NONE: &str = "nenhuma";
// shortcut: one threshold for every model, not tuned on pt-BR (⚠ não medido); tune per model with evals/.
pub const THRESHOLD: f64 = 0.5;
/// One request per sentence: a field longer than this is checked in its first sentences.
const MAX_SENTENCES: usize = 32;

/// `### PTW-1: Quem faz, aparece` → `("PTW-1", "Quem faz, aparece")`, in order, from the Regras
/// section of a skill.
pub fn rules_of(skill: &str, heading: &str, file: &str) -> Result<Vec<(String, String)>, SectionError> {
    Ok(section(skill, heading, "Regras", file)?
        .lines()
        .filter_map(|l| {
            let rest = l.strip_prefix("### ")?;
            let (code, title) = rest.split_once(": ")?;
            let (letters, number) = code.split_once('-')?;
            let is_code = !letters.is_empty()
                && letters.chars().all(|c| c.is_ascii_uppercase())
                && !number.is_empty()
                && number.chars().all(|c| c.is_ascii_digit());
            (is_code && !title.is_empty()).then(|| (code.to_string(), title.to_string()))
        })
        .collect())
}

/// The sentences of `text`, as they appear in it: split after . ! ? … and at line breaks; a piece
/// under 3 words is not a sentence.
pub fn sentences(text: &str) -> Vec<String> {
    let mut out = vec![];
    let mut start = 0;
    let mut prev = ' ';
    let mut chars = text.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c.is_whitespace() {
            let mut end = i + c.len_utf8();
            let mut has_newline = c == '\n';
            while let Some(&(j, w)) = chars.peek().filter(|(_, w)| w.is_whitespace()) {
                has_newline |= w == '\n';
                end = j + w.len_utf8();
                chars.next();
            }
            if matches!(prev, '.' | '!' | '?' | '…') || has_newline {
                out.push(&text[start..i]);
                start = end;
            }
        }
        prev = c;
    }
    out.push(&text[start..]);
    out.into_iter()
        .map(str::trim)
        .filter(|s| s.split_whitespace().count() >= 3)
        .take(MAX_SENTENCES)
        .map(String::from)
        .collect()
}

#[derive(Clone, Debug, PartialEq)]
pub struct Ask {
    pub sentence: String,
    /// 1-based, among the identical sentences of the text.
    pub occurrence: u32,
    pub body: Value,
}

/// One /api/decide body per sentence of `text`.
pub fn decide_asks(
    model: &str,
    level: Level,
    text: &str,
    skill: &str,
    cientifico: &str,
) -> Result<Vec<Ask>, SectionError> {
    let mut rules = rules_of(skill, "## Regras (em desenvolvimento)", "SKILL.md")?;
    if level == Level::Cientifico {
        rules.extend(rules_of(cientifico, "## Regras", "cientifico/SKILL.md")?);
    }
    let mut criteria = Map::new();
    criteria.insert(NONE.into(), json!("nenhuma regra quebrada"));
    criteria.extend(rules.into_iter().map(|(code, title)| (code, Value::String(title))));
    let mut seen: HashMap<String, u32> = HashMap::new();
    Ok(sentences(text)
        .into_iter()
        .map(|sentence| {
            let occurrence = seen.entry(sentence.clone()).or_insert(0);
            *occurrence += 1;
            let occurrence = *occurrence;
            let question = json!({ "type": "choice", "instructions": format!("Qual regra de escrita técnica (nível {level}) esta frase quebra?"), "criteria": criteria });
            let body = json!({ "model": model, "state": sentence, "questions": { "regra": question }, "keep_alive": "10m" });
            Ask { sentence, occurrence, body }
        })
        .collect())
}

/// The underline for one sentence from its /api/decide response: its most likely rule, when that
/// is not `nenhuma` and passes THRESHOLD.
pub fn read_decision(json: &Value, ask: &Ask) -> Option<Suggestion> {
    let a = json.get("answers")?.get("regra")?;
    let choice = a.get("choice")?.as_str()?;
    if choice == NONE {
        return None;
    }
    let p = a.get("probabilities")?.get(choice)?.as_f64()?;
    if p < THRESHOLD {
        return None;
    }
    Some(Suggestion {
        original: ask.sentence.clone(),
        replacement: String::new(),
        detect_only: true,
        rule: format!("{choice} ({}%)", (p * 100.0).round()),
        severity: Severity::Aviso,
        occurrence: ask.occurrence,
    })
}

/// Ollaya + Ollama: before Ollama writes, Ollaya reads the whole field and picks the conversation
/// style of the rewrite, one of Style; their descriptions are the options the model weighs.
pub fn style_body(model: &str, text: &str) -> Value {
    let criteria: Map<String, Value> =
        Style::ALL.iter().map(|s| (s.to_string(), json!(style_description(*s)))).collect();
    let question = json!({ "type": "choice", "instructions": "Qual estilo de conversa a reescrita deste texto deve seguir?", "criteria": criteria });
    json!({ "model": model, "state": text, "questions": { "estilo": question }, "keep_alive": "10m" })
}

/// The style Ollaya chose; `None` when the reply is not one of Style.
pub fn read_style(json: &Value) -> Option<Style> {
    serde_json::from_value(json.get("answers")?.get("estilo")?.get("choice")?.clone()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::{CIENTIFICO, SKILL};

    const TEXT: &str =
        "Foi decidido que o deploy será feito. A equipe valida o build hoje.\nFoi decidido que o deploy será feito.";

    #[test]
    fn reads_ptw_1_to_13_with_their_titles() {
        let r = rules_of(SKILL, "## Regras (em desenvolvimento)", "SKILL.md").unwrap();
        let codes: Vec<&str> = r.iter().map(|(c, _)| c.as_str()).collect();
        assert_eq!(codes, (1..=13).map(|i| format!("PTW-{i}")).collect::<Vec<_>>());
        assert_eq!(r[0].1, "Quem faz, aparece");
    }

    #[test]
    fn splits_after_punctuation_and_at_line_breaks_and_drops_fragments_under_3_words() {
        assert_eq!(sentences("Oi. Isto é uma frase! E esta outra?\nok sim"), ["Isto é uma frase!", "E esta outra?"]);
        assert_eq!(
            sentences("um dois três \n \n quatro cinco seis…  sete oito nove"),
            ["um dois três", "quatro cinco seis…", "sete oito nove"]
        );
        assert_eq!(sentences(&"a b c. ".repeat(40)).len(), MAX_SENTENCES);
    }

    #[test]
    fn one_body_per_sentence_the_sentence_is_the_state_one_choice_question_nenhuma_first() {
        let asks = decide_asks("decima", Level::Estrito, TEXT, SKILL, CIENTIFICO).unwrap();
        let seen: Vec<(&str, u32)> = asks.iter().map(|a| (a.sentence.as_str(), a.occurrence)).collect();
        assert_eq!(
            seen,
            [
                ("Foi decidido que o deploy será feito.", 1),
                ("A equipe valida o build hoje.", 1),
                ("Foi decidido que o deploy será feito.", 2)
            ]
        );
        let body = &asks[1].body;
        assert_eq!(body["model"], "decima");
        assert_eq!(body["state"], "A equipe valida o build hoje.");
        assert_eq!(body["questions"]["regra"]["type"], "choice");
        let criteria = body["questions"]["regra"]["criteria"].as_object().unwrap();
        assert_eq!(criteria.keys().next().unwrap(), "nenhuma");
        assert_eq!(criteria.len(), 14);
    }

    #[test]
    fn cientifico_adds_the_cie_rules() {
        let asks = decide_asks("decima", Level::Cientifico, TEXT, SKILL, CIENTIFICO).unwrap();
        assert!(asks[0].body["questions"]["regra"]["criteria"]["CIE-1"].is_string());
    }

    fn reply(choice: &str, p: f64) -> Value {
        json!({ "answers": { "regra": { "type": "choice", "choice": choice, "probabilities": { choice: p } } } })
    }

    #[test]
    fn a_rule_above_the_threshold_becomes_a_detection_at_its_occurrence() {
        let ask = &decide_asks("decima", Level::Estrito, TEXT, SKILL, CIENTIFICO).unwrap()[2];
        assert_eq!(
            read_decision(&reply("PTW-1", 0.82), ask),
            Some(Suggestion {
                original: "Foi decidido que o deploy será feito.".into(),
                replacement: String::new(),
                detect_only: true,
                rule: "PTW-1 (82%)".into(),
                severity: Severity::Aviso,
                occurrence: 2,
            })
        );
    }

    #[test]
    fn nenhuma_a_weak_answer_or_an_unexpected_body_give_nothing() {
        let ask = &decide_asks("decima", Level::Estrito, TEXT, SKILL, CIENTIFICO).unwrap()[2];
        assert_eq!(read_decision(&reply("nenhuma", 0.9), ask), None);
        assert_eq!(read_decision(&reply("PTW-1", THRESHOLD - 0.01), ask), None);
        assert_eq!(read_decision(&Value::Null, ask), None);
        assert_eq!(read_decision(&json!({ "answers": { "regra": { "choice": 3 } } }), ask), None);
    }

    #[test]
    fn asks_one_choice_question_over_the_whole_field_the_styles_as_options() {
        let body = style_body("decima", TEXT);
        assert_eq!(body["state"], TEXT);
        let keys: Vec<&str> =
            body["questions"]["estilo"]["criteria"].as_object().unwrap().keys().map(String::as_str).collect();
        assert_eq!(keys, ["conciso", "socrático", "amigável"]);
    }

    #[test]
    fn reads_the_chosen_style_anything_else_is_none() {
        assert_eq!(read_style(&json!({ "answers": { "estilo": { "choice": "socrático" } } })), Some(Style::Socratico));
        assert_eq!(read_style(&json!({ "answers": { "estilo": { "choice": "toString" } } })), None);
        assert_eq!(read_style(&Value::Null), None);
    }
}
