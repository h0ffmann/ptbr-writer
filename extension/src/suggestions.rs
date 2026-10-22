//! The model's answer: the JSON array in it, where each item sits in the text (in UTF-16 units,
//! matched in NFC), and the edits.

use crate::types::{Located, Severity, Suggestion};
use crate::utf16::{byte_at, slice16};
use serde_json::Value;
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

/// The suggestions in a reply: the first `[` to the last `]`, the items with every field valid.
pub fn parse_suggestions(raw: &str) -> Vec<Suggestion> {
    let (Some(from), Some(to)) = (raw.find('['), raw.rfind(']')) else {
        return vec![];
    };
    if to < from {
        return vec![];
    }
    let Ok(Value::Array(items)) = serde_json::from_str::<Value>(&raw[from..=to]) else {
        return vec![];
    };
    items
        .iter()
        .filter_map(|o| {
            let original = o.get("original")?.as_str()?;
            if original.is_empty() {
                return None;
            }
            let replacement = o.get("replacement")?.as_str()?;
            let rule = o.get("rule")?.as_str()?;
            let severity: Severity = serde_json::from_value(o.get("severity")?.clone()).ok()?;
            // An integer from 1; anything else is the first occurrence.
            let occurrence = o
                .get("occurrence")
                .and_then(Value::as_f64)
                .filter(|n| n.fract() == 0.0 && (1.0..=u32::MAX as f64).contains(n))
                .map_or(1, |n| n as u32);
            Some(Suggestion {
                original: original.into(),
                replacement: replacement.into(),
                detect_only: false,
                rule: rule.into(),
                severity,
                occurrence,
            })
        })
        .collect()
}

/// NFC of `text` in UTF-16 units plus, for each unit, the span it came from in `text`.
struct Normalized {
    units: Vec<u16>,
    starts: Vec<u32>,
    ends: Vec<u32>,
}

fn normalize(text: &str) -> Normalized {
    let mut n = Normalized { units: vec![], starts: vec![], ends: vec![] };
    let mut chars = text.chars().peekable();
    let mut pos = 0u32;
    // Runs of one character and the marks after it (`\P{M}\p{M}*|\p{M}+`), normalized one by one.
    while let Some(c) = chars.next() {
        let start = pos;
        let mut run = String::from(c);
        pos += c.len_utf16() as u32;
        while let Some(&m) = chars.peek().filter(|m| is_combining_mark(**m)) {
            run.push(m);
            pos += m.len_utf16() as u32;
            chars.next();
        }
        for u in run.nfc().collect::<String>().encode_utf16() {
            n.units.push(u);
            n.starts.push(start);
            n.ends.push(pos);
        }
    }
    n
}

fn find_u16(hay: &[u16], needle: &[u16], from: usize) -> Option<usize> {
    if from > hay.len() || needle.is_empty() {
        return None;
    }
    hay[from..].windows(needle.len()).position(|w| w == needle).map(|i| i + from)
}

/// Where each suggestion's occurrence of `original` is in `text`, sorted by start. One not found,
/// or equal to its replacement, is dropped.
pub fn locate(text: &str, suggestions: &[Suggestion]) -> Vec<Located> {
    let n = normalize(text);
    let mut out: Vec<Located> = suggestions
        .iter()
        .filter(|s| s.original != s.replacement)
        .filter_map(|s| {
            let needle: Vec<u16> = s.original.nfc().collect::<String>().encode_utf16().collect();
            let mut at = None;
            let mut from = 0;
            for _ in 0..s.occurrence {
                at = find_u16(&n.units, &needle, from);
                from = at? + needle.len();
            }
            let at = at?;
            Some(Located { suggestion: s.clone(), start: n.starts[at], end: n.ends[at + needle.len() - 1] })
        })
        .collect();
    out.sort_by_key(|l| l.start);
    out
}

/// Of overlapping spans, the first; `l` sorted by start.
pub fn drop_overlaps(l: Vec<Located>) -> Vec<Located> {
    let mut out: Vec<Located> = vec![];
    let mut last = 0;
    for x in l {
        if x.start < last {
            continue;
        }
        last = x.end;
        out.push(x);
    }
    out
}

/// `text` with the span of `l` replaced; `text` unchanged when the span is not in it.
pub fn apply_to_string(text: &str, l: &Located) -> String {
    match (byte_at(text, l.start), byte_at(text, l.end)) {
        (Some(a), Some(b)) if a <= b => {
            format!("{}{}{}", &text[..a], l.suggestion.replacement, &text[b..])
        }
        _ => text.to_string(),
    }
}

/// The text a located suggestion covers, for a check against `original`.
pub fn span<'a>(text: &'a str, l: &Located) -> Option<&'a str> {
    slice16(text, l.start, l.end)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sug(original: &str, occurrence: u32, replacement: &str) -> Suggestion {
        Suggestion {
            original: original.into(),
            replacement: replacement.into(),
            detect_only: false,
            rule: "PTW-1".into(),
            severity: Severity::Erro,
            occurrence,
        }
    }

    #[test]
    fn tira_cercas_e_preenche_occurrence() {
        let raw = "```json\n[{\"original\":\"Foi decidido\",\"replacement\":\"A equipe decidiu\",\"rule\":\"PTW-1\",\"severity\":\"erro\"}]\n```";
        assert_eq!(parse_suggestions(raw), vec![sug("Foi decidido", 1, "A equipe decidiu")]);
    }

    #[test]
    fn prosa_com_lista_vazia() {
        assert_eq!(parse_suggestions("Aqui está: [] ok"), vec![]);
    }

    #[test]
    fn json_invalido() {
        assert_eq!(parse_suggestions("não é json"), vec![]);
        assert_eq!(parse_suggestions("[{oops]"), vec![]);
    }

    #[test]
    fn descarta_severity_desconhecida_e_campos_faltando() {
        let raw = json!([
            { "original": "a", "replacement": "b", "rule": "R", "severity": "info" },
            { "original": "", "replacement": "b", "rule": "R", "severity": "erro" },
            { "original": "a", "replacement": 1, "rule": "R", "severity": "erro" },
            { "original": "a", "replacement": "b", "rule": "R", "severity": "aviso" },
        ]);
        assert_eq!(parse_suggestions(&raw.to_string()).len(), 1);
    }

    #[test]
    fn occurrence_invalida_vira_1() {
        let mk = |o: Value| {
            parse_suggestions(
                &json!([{ "original": "a", "replacement": "b", "rule": "R", "severity": "erro", "occurrence": o }])
                    .to_string(),
            )[0]
            .occurrence
        };
        assert_eq!(mk(json!(0)), 1);
        assert_eq!(mk(json!(1.5)), 1);
        assert_eq!(mk(json!("2")), 1);
        assert_eq!(mk(json!(3)), 3);
    }

    #[test]
    fn acha_a_ocorrencia_pedida() {
        let r = locate("a nível de x e a nível de y", &[sug("a nível de", 2, "x")]);
        assert_eq!(r[0].start, 15);
        assert_eq!(r[0].end, 25);
    }

    #[test]
    fn descarta_ausente_igual_e_ocorrencia_inexistente() {
        assert_eq!(locate("foo bar", &[sug("baz", 1, "x")]), vec![]);
        assert_eq!(locate("foo bar", &[sug("foo", 1, "foo")]), vec![]);
        assert_eq!(locate("foo bar", &[sug("foo", 2, "x")]), vec![]);
    }

    #[test]
    fn ordena_por_start() {
        let r = locate("foo bar", &[sug("bar", 1, "x"), sug("foo", 1, "x")]);
        assert_eq!(r.iter().map(|l| l.start).collect::<Vec<_>>(), [0, 4]);
        assert_eq!(r.iter().map(|l| l.suggestion.original.as_str()).collect::<Vec<_>>(), ["foo", "bar"]);
    }

    #[test]
    fn conta_em_utf16_com_emoji() {
        let r = locate("Oi 👋 foi decidido", &[sug("foi decidido", 1, "x")]);
        assert_eq!(r[0].start, 6);
    }

    #[test]
    fn casa_em_nfc_e_devolve_posicoes_do_texto_original_nfd() {
        let nfd = "ola e\u{301} muito bom";
        let r = locate(nfd, &[sug("é muito", 1, "x")]);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].start, 4);
        assert_eq!(span(nfd, &r[0]), Some("e\u{301} muito"));
        let nfc = locate("ola é muito bom", &[sug("é muito", 1, "x")]);
        assert_eq!(nfc[0].start, 4);
        assert_eq!(nfc[0].end, 11);
    }

    #[test]
    fn mantem_o_primeiro_de_cada_sobreposicao() {
        let l = |start, end| Located { suggestion: sug("x", 1, "x"), start, end };
        let r = drop_overlaps(vec![l(0, 12), l(4, 20), l(20, 25)]);
        assert_eq!(r.iter().map(|x| (x.start, x.end)).collect::<Vec<_>>(), [(0, 12), (20, 25)]);
    }

    #[test]
    fn aplica_a_troca() {
        let l = Located { suggestion: sug("Foi decidido", 1, "A equipe decidiu"), start: 0, end: 12 };
        assert_eq!(apply_to_string("Foi decidido hoje", &l), "A equipe decidiu hoje");
        let l = Located { suggestion: sug("ção", 1, "cao"), start: 4, end: 7 };
        assert_eq!(apply_to_string("👋 ação!", &l), "👋 acao!");
    }
}
