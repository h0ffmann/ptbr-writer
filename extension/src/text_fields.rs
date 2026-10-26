//! Tone, rewrite and wait, shared by the popup and the options page. Each select saves at once.

use crate::types::Tone;

/// A tone as the list shows it: `Informal`, `Neutro`, `Formal`.
pub fn tone_label(t: Tone) -> String {
    let s = t.as_str();
    s[..1].to_uppercase() + &s[1..]
}

/// A wait as the list shows it: `1 s`.
pub fn delay_label(ms: u32) -> String {
    format!("{} s", ms / 1000)
}

#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use crate::dom::{Build, create, el, on, option, spawn};
    use crate::settings::DELAYS;
    use crate::sites::save_text_settings;
    use crate::types::Settings;
    use web_sys::{HtmlElement, HtmlSelectElement};

    fn select(id: &str, options: &[(String, String)], value: &str) -> HtmlSelectElement {
        let sel: HtmlSelectElement = create::<HtmlSelectElement>("select").id(id);
        for (v, text) in options {
            sel.append_child(&option(v, text)).expect("appendChild");
        }
        sel.set_value(value);
        sel
    }

    // On each change, the select's value saved through `set`.
    fn saves(sel: &HtmlSelectElement, set: impl Fn(&mut Settings, &str) + Copy + 'static) {
        on(sel, "change", {
            let sel = sel.clone();
            move || {
                let value = sel.value();
                spawn(async move { save_text_settings(|s| set(s, &value)).await })
            }
        });
    }

    pub fn mount_text_fields(parent: &HtmlElement, s: &Settings) {
        let tones: Vec<(String, String)> = Tone::ALL.iter().map(|t| (t.as_str().into(), tone_label(*t))).collect();
        let tone = select("tone", &tones, s.tone.as_str());
        let rewrites = [("false".into(), "Só erros".into()), ("true".into(), "Erros e reescrita".into())];
        let rewrite = select("rewrite", &rewrites, if s.rewrite { "true" } else { "false" });
        let delays: Vec<(String, String)> = DELAYS.iter().map(|d| (d.to_string(), delay_label(*d))).collect();
        let delay = select("delay", &delays, &s.delay_ms.to_string());
        saves(&tone, |s, v| s.tone = Tone::parse(v).unwrap_or(s.tone));
        saves(&rewrite, |s, v| s.rewrite = v == "true");
        saves(&delay, |s, v| s.delay_ms = v.parse().ok().filter(|d| DELAYS.contains(d)).unwrap_or(s.delay_ms));
        parent.child(&el("label").text("Tom ").child(&tone));
        parent.child(&el("label").text("O quanto corrigir ").child(&rewrite));
        parent.child(&el("label").text("Espera ").child(&delay));
    }
}

#[cfg(target_arch = "wasm32")]
pub use browser::mount_text_fields;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels() {
        assert_eq!(Tone::ALL.map(tone_label), ["Informal", "Neutro", "Formal"]);
        assert_eq!(crate::settings::DELAYS.map(delay_label), ["1 s", "2 s", "3 s"]);
    }
}
