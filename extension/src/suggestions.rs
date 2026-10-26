//! The model's answer: the JSON array in it, where each item sits in the text (in UTF-16 units,
//! matched in NFC), and the edits.

use crate::never_fix::breaks_entry;
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

/// The service worker's filters, on positions `locate` finds in the sent text, in this order:
/// (1) without `rewrite` every aviso goes; (2) an aviso across a line break goes, as in the watcher,
/// so its erros stay; (3) a suggestion that would change a never-fix entry goes; (4) an erro inside a
/// remaining aviso goes (one sentence, one color); (5) `drop_overlaps` settles the rest.
pub fn filter_suggestions(text: &str, s: &[Suggestion], rewrite: bool, never_fix: &[String]) -> Vec<Suggestion> {
    let aviso = |x: &Located| x.suggestion.severity == Severity::Aviso;
    let mut l = locate(text, s);
    // "Só erros" drops rewrites; a detection (Ollaya, no replacement) is not a rewrite and stays.
    if !rewrite {
        l.retain(|x| !aviso(x) || x.suggestion.detect_only);
    }
    l.retain(|x| !aviso(x) || !span(text, x).is_some_and(|t| t.contains('\n')));
    // A detect-only card (Ollaya) changes nothing, so it cannot break a word the person keeps.
    l.retain(|x| {
        let s = &x.suggestion;
        s.detect_only || !never_fix.iter().any(|e| breaks_entry(&s.original, &s.replacement, e))
    });
    let avisos: Vec<(u32, u32)> = l.iter().filter(|x| aviso(x)).map(|x| (x.start, x.end)).collect();
    l.retain(|x| aviso(x) || !avisos.iter().any(|&(a, b)| x.start >= a && x.end <= b));
    drop_overlaps(l).into_iter().map(|x| x.suggestion).collect()
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

    mod filter {
        use super::*;

        const TEXT: &str = "bomd ia tudo bem com vc. O relatório está pronto";

        fn mk(original: &str, replacement: &str, severity: Severity) -> Suggestion {
            Suggestion { severity, ..sug(original, 1, replacement) }
        }

        fn aviso() -> Suggestion {
            mk("bomd ia tudo bem com vc.", "Bom dia, tudo bem com você.", Severity::Aviso)
        }

        fn erro() -> Suggestion {
            mk("vc", "você", Severity::Erro)
        }

        fn keep(list: &[&str]) -> Vec<String> {
            list.iter().map(|s| s.to_string()).collect()
        }

        #[test]
        fn erro_dentro_de_aviso_fica_so_o_aviso() {
            assert_eq!(filter_suggestions(TEXT, &[aviso(), erro()], true, &[]), [aviso()]);
        }

        #[test]
        fn so_erros_fica_so_o_erro() {
            assert_eq!(filter_suggestions(TEXT, &[aviso(), erro()], false, &[]), [erro()]);
        }

        #[test]
        fn aviso_nao_achado_o_erro_fica() {
            let lost = mk("bomd  ia", "Bom dia", Severity::Aviso);
            assert_eq!(filter_suggestions(TEXT, &[lost, erro()], true, &[]), [erro()]);
        }

        #[test]
        fn lista_some_o_erro_e_o_aviso_que_trocam_a_entrada() {
            let kept = mk("O relatório está pronto", "O relatório está pronto, vc", Severity::Aviso);
            let drop = mk("bomd ia tudo bem com vc.", "Bom dia, tudo bem.", Severity::Aviso);
            let vc = keep(&["vc"]);
            assert_eq!(filter_suggestions(TEXT, &[erro()], true, &vc), []);
            assert_eq!(filter_suggestions(TEXT, &[drop], true, &vc), []);
            assert_eq!(filter_suggestions(TEXT, std::slice::from_ref(&kept), true, &vc), [kept]);
        }

        #[test]
        fn lista_troca_so_de_maiuscula_tambem_some_com_a_entrada_intacta_fica() {
            let t = "falei com maria ontem";
            let up = mk("maria", "Maria", Severity::Erro);
            let comma = mk("maria", "maria,", Severity::Erro);
            let maria = keep(&["maria"]);
            assert_eq!(filter_suggestions(t, &[up], true, &maria), []);
            assert_eq!(filter_suggestions(t, std::slice::from_ref(&comma), true, &maria), [comma]);
        }

        #[test]
        fn aviso_descartado_pela_lista_devolve_os_seus_erros() {
            let bomd = mk("bomd ia", "bom dia", Severity::Erro);
            assert_eq!(filter_suggestions(TEXT, &[aviso(), bomd.clone()], true, &keep(&["vc"])), [bomd]);
        }

        #[test]
        fn aviso_que_atravessa_uma_quebra_de_linha_some_e_devolve_os_seus_erros() {
            let t = "bomd ia tudo\nbem com vc.";
            let across = mk("bomd ia tudo\nbem com vc.", "Bom dia, tudo bem com você?", Severity::Aviso);
            let bomd = mk("bomd ia", "bom dia", Severity::Erro);
            assert_eq!(filter_suggestions(t, &[across, bomd.clone()], true, &[]), [bomd]);
        }

        #[test]
        fn sobreposicao_parcial_fica_so_o_primeiro_sem_erro() {
            let a = mk(&TEXT[..10], "A", Severity::Aviso);
            let e = mk(&TEXT[5..15], "E", Severity::Erro);
            assert_eq!(filter_suggestions(TEXT, &[a.clone(), e], true, &[]), [a]);
        }
    }
}
