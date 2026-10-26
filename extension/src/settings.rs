//! The settings: defaults, the model lists, the level of a site, and what chrome.storage.local
//! holds merged over the defaults.

use crate::types::{Level, PerProvider, ProviderId, Settings, SiteConfig, Tone};
use serde_json::Value;
use std::collections::BTreeMap;

pub const STORAGE_KEY: &str = "settings";

impl Default for Settings {
    fn default() -> Self {
        Settings {
            enabled: true,
            provider: ProviderId::Claude,
            // The local providers take no key.
            keys: PerProvider {
                claude: String::new(),
                deepseek: String::new(),
                ollama: String::new(),
                ollaya: String::new(),
            },
            models: PerProvider {
                claude: "claude-haiku-5-5".into(),
                // Checked 2026-10-21 at https://api-docs.deepseek.com (and /quick_start/pricing): the docs list
                // `deepseek-flash` and `deepseek-v4-pro` and no longer mention `deepseek-chat`.
                // ⚠ Read through a page summarizer, not by hand; confirm in the browser before a release.
                deepseek: "deepseek-flash".into(),
                // The model of just opencode-qwen (Qwen3.6-27B), from Ollama's library; just ext-ollama pulls it.
                ollama: "qwen3.6:27b".into(),
                // A decision model whose card lists Portuguese, runs on a CPU and is calibrated: the pick of
                // docs/SURVEY_DECISION_MODEL.md (`decima` is `decima:base`). ⚠ Not measured on pt-BR rules.
                ollaya: "decima".into(),
            },
            ollaya_style: false,
            sites: BTreeMap::from([
                ("web.whatsapp.com".to_string(), SiteConfig { enabled: true, level: Level::Leve }),
                ("mail.google.com".to_string(), SiteConfig { enabled: true, level: Level::Descritivo }),
                ("github.com".to_string(), SiteConfig { enabled: true, level: Level::Descritivo }),
            ]),
            tone: Tone::Neutro,
            rewrite: true,
            never_fix: Vec::new(),
            delay_ms: 1000,
            api_base: BTreeMap::new(),
        }
    }
}

/// The waits the settings offer, in ms.
pub const DELAYS: [u32; 3] = [1000, 2000, 3000];

/// The models the options page offers; the first of each list is the default.
pub fn models(p: ProviderId) -> &'static [&'static str] {
    match p {
        ProviderId::Claude => &["claude-haiku-5-5", "claude-sonnet-5-5", "claude-opus-5-5"],
        ProviderId::Deepseek => &["deepseek-flash", "deepseek-v4-pro"],
        // Suggestions; the model list also shows what Ollama has installed.
        ProviderId::Ollama => &["qwen3.6:27b", "qwen3:14b", "qwen3:8b", "qwen3:4b"],
        // The same, for Ollaya; the last two want a GPU.
        ProviderId::Ollaya => &["decima", "laya:multilingual", "decima:small", "pplx-decider", "winnow:e4b"],
    }
}

/// A model the options page could have saved: any for a local provider, one of `models` otherwise.
pub fn known_model(p: ProviderId, m: Option<&str>) -> bool {
    m.is_some_and(|m| !m.is_empty() && (p.is_local() || models(p).contains(&m)))
}

pub fn level_for(host: &str, s: &Settings) -> Option<Level> {
    if !s.enabled {
        return None;
    }
    s.sites.get(host).filter(|site| site.enabled).map(|site| site.level)
}

impl Settings {
    /// The settings in storage over the defaults: a field missing or malformed keeps its default,
    /// a key, model or site missing keeps the default one, a stored model outside its provider's
    /// list falls back to the default.
    pub fn from_stored(stored: &Value) -> Settings {
        let mut s = Settings::default();
        let field = |key: &str| stored.get(key);
        if let Some(enabled) = field("enabled").and_then(Value::as_bool) {
            s.enabled = enabled;
        }
        if let Some(provider) = field("provider").and_then(|v| serde_json::from_value(v.clone()).ok()) {
            s.provider = provider;
        }
        if let Some(style) = field("ollayaStyle").and_then(Value::as_bool) {
            s.ollaya_style = style;
        }
        for p in ProviderId::ALL {
            if let Some(key) = field("keys").and_then(|k| k.get(p.as_str())).and_then(Value::as_str) {
                s.keys[p] = key.into();
            }
            let model = field("models").and_then(|m| m.get(p.as_str())).and_then(Value::as_str);
            if known_model(p, model) {
                s.models[p] = model.unwrap_or_default().into();
            }
            if let Some(base) = field("apiBase").and_then(|b| b.get(p.as_str())).and_then(Value::as_str) {
                s.api_base.insert(p, base.into());
            }
        }
        if let Some(tone) = field("tone").and_then(|v| serde_json::from_value(v.clone()).ok()) {
            s.tone = tone;
        }
        if let Some(rewrite) = field("rewrite").and_then(Value::as_bool) {
            s.rewrite = rewrite;
        }
        if let Some(list) = field("neverFix").and_then(Value::as_array) {
            s.never_fix = list.iter().filter_map(Value::as_str).map(String::from).collect();
        }
        if let Some(delay) = field("delayMs").and_then(Value::as_u64).and_then(|d| u32::try_from(d).ok())
            && DELAYS.contains(&delay)
        {
            s.delay_ms = delay;
        }
        for (host, cfg) in field("sites").and_then(Value::as_object).into_iter().flatten() {
            if let Ok(cfg) = serde_json::from_value::<SiteConfig>(cfg.clone()) {
                s.sites.insert(host.clone(), cfg);
            }
        }
        s
    }
}

#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use crate::chrome;
    use wasm_bindgen::JsValue;

    pub async fn load_settings() -> Result<Settings, JsValue> {
        let stored = chrome::storage_get(STORAGE_KEY).await?.unwrap_or(Value::Null);
        Ok(Settings::from_stored(&stored))
    }

    pub async fn save_settings(s: &Settings) -> Result<(), JsValue> {
        chrome::storage_set(STORAGE_KEY, s).await
    }
}

#[cfg(target_arch = "wasm32")]
pub use browser::{load_settings, save_settings};

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn with_sites(sites: BTreeMap<String, SiteConfig>) -> Settings {
        Settings { sites, ..Settings::default() }
    }

    #[test]
    fn default_sites() {
        let d = Settings::default();
        assert_eq!(level_for("web.whatsapp.com", &d), Some(Level::Leve));
        assert_eq!(level_for("mail.google.com", &d), Some(Level::Descritivo));
        assert_eq!(level_for("github.com", &d), Some(Level::Descritivo));
        assert_eq!(level_for("bank.example", &d), None);
    }

    #[test]
    fn disabled_extension() {
        assert_eq!(level_for("github.com", &Settings { enabled: false, ..Settings::default() }), None);
    }

    #[test]
    fn disabled_site() {
        let s =
            with_sites(BTreeMap::from([("github.com".to_string(), SiteConfig { enabled: false, level: Level::Leve })]));
        assert_eq!(level_for("github.com", &s), None);
    }

    #[test]
    fn allowed_site() {
        let mut s = Settings::default();
        s.sites.insert("bank.example".into(), SiteConfig { enabled: true, level: Level::Descritivo });
        assert_eq!(level_for("bank.example", &s), Some(Level::Descritivo));
    }

    #[test]
    fn values() {
        let d = Settings::default();
        assert!(d.enabled);
        assert_eq!(d.provider, ProviderId::Claude);
        assert_eq!(
            d.keys,
            PerProvider { claude: "".into(), deepseek: "".into(), ollama: "".into(), ollaya: "".into() }
        );
        assert_eq!(d.models.claude, "claude-haiku-5-5");
    }

    #[test]
    fn empty_storage_gives_defaults() {
        assert_eq!(Settings::from_stored(&Value::Null), Settings::default());
        assert_eq!(Settings::from_stored(&json!({})), Settings::default());
    }

    #[test]
    fn keeps_default_keys_and_models_missing_from_storage() {
        let s = Settings::from_stored(
            &json!({ "keys": { "claude": "sk-test" }, "models": { "deepseek": "deepseek-v4-pro" } }),
        );
        assert_eq!(
            s.keys,
            PerProvider { claude: "sk-test".into(), deepseek: "".into(), ollama: "".into(), ollaya: "".into() }
        );
        assert_eq!(s.models, PerProvider { deepseek: "deepseek-v4-pro".into(), ..Settings::default().models });
    }

    #[test]
    fn a_stored_model_outside_the_list_falls_back_to_the_provider_default() {
        let s = Settings::from_stored(&json!({ "models": { "claude": "claude-typo", "deepseek": "deepseek-chat" } }));
        assert_eq!(s.models, Settings::default().models);
    }

    #[test]
    fn every_default_model_is_in_its_provider_list() {
        let d = Settings::default();
        assert_eq!(models(ProviderId::Claude), ["claude-haiku-5-5", "claude-sonnet-5-5", "claude-opus-5-5"]);
        assert_eq!(models(ProviderId::Deepseek), ["deepseek-flash", "deepseek-v4-pro"]);
        assert_eq!(models(ProviderId::Claude)[0], d.models.claude);
        assert_eq!(models(ProviderId::Deepseek)[0], d.models.deepseek);
        assert_eq!(models(ProviderId::Ollama), ["qwen3.6:27b", "qwen3:14b", "qwen3:8b", "qwen3:4b"]);
        assert_eq!(models(ProviderId::Ollama)[0], d.models.ollama);
        assert_eq!(models(ProviderId::Ollaya)[0], d.models.ollaya);
    }

    #[test]
    fn keeps_the_default_sites_when_storage_has_one_host() {
        let s = Settings::from_stored(&json!({ "sites": { "bank.example": { "enabled": true, "level": "estrito" } } }));
        let hosts: Vec<&str> = s.sites.keys().map(String::as_str).collect();
        assert_eq!(hosts, ["bank.example", "github.com", "mail.google.com", "web.whatsapp.com"]);
        assert_eq!(level_for("bank.example", &s), Some(Level::Estrito));
    }

    #[test]
    fn stored_top_level_values_win() {
        let s = Settings::from_stored(&json!({ "enabled": false, "provider": "deepseek" }));
        assert!(!s.enabled);
        assert_eq!(s.provider, ProviderId::Deepseek);
    }

    #[test]
    fn save_round_trips_through_the_stored_shape() {
        let s = Settings { provider: ProviderId::Deepseek, ollaya_style: true, ..Settings::default() };
        let stored = serde_json::to_value(&s).unwrap();
        assert_eq!(stored["ollayaStyle"], json!(true));
        assert!(stored.get("apiBase").is_none());
        assert_eq!(Settings::from_stored(&stored), s);
        let with_base = Settings::from_stored(&json!({ "apiBase": { "claude": "http://127.0.0.1:9" } }));
        assert_eq!(with_base.api_base.get(&ProviderId::Claude).map(String::as_str), Some("http://127.0.0.1:9"));
        assert_eq!(Settings::from_stored(&serde_json::to_value(&with_base).unwrap()), with_base);
    }

    #[test]
    fn a_malformed_field_keeps_its_default_and_the_rest_loads() {
        let s = Settings::from_stored(
            &json!({ "provider": "gemini", "enabled": false, "sites": { "a.com": { "enabled": true }, "b.com": { "enabled": true, "level": "leve" } } }),
        );
        assert_eq!(s.provider, ProviderId::Claude);
        assert!(!s.enabled);
        assert!(!s.sites.contains_key("a.com"));
        assert_eq!(s.sites["b.com"], SiteConfig { enabled: true, level: Level::Leve });
    }

    #[test]
    fn tone_rewrite_never_fix_and_wait_defaults() {
        let d = Settings::from_stored(&json!({}));
        assert_eq!((d.tone, d.rewrite, d.never_fix.len(), d.delay_ms), (Tone::Neutro, true, 0, 1000));
    }

    #[test]
    fn tone_rewrite_never_fix_and_wait_outside_the_list_fall_back_to_the_defaults() {
        let s =
            Settings::from_stored(&json!({ "tone": "gritado", "delayMs": 5000, "rewrite": "sim", "neverFix": "x" }));
        assert_eq!((s.tone, s.rewrite, s.never_fix.len(), s.delay_ms), (Tone::Neutro, true, 0, 1000));
    }

    #[test]
    fn valid_tone_rewrite_never_fix_and_wait_are_kept() {
        let s = Settings::from_stored(
            &json!({ "tone": "formal", "delayMs": 3000, "rewrite": false, "neverFix": ["Kafka", 3, "Pix"] }),
        );
        assert_eq!((s.tone, s.rewrite, s.delay_ms), (Tone::Formal, false, 3000));
        assert_eq!(s.never_fix, ["Kafka", "Pix"]);
        assert_eq!(Settings::from_stored(&serde_json::to_value(&s).unwrap()), s);
    }
}
