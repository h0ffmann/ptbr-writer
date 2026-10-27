//! The settings file: the extension's Settings (its JSON, checked by `Settings::from_stored`) plus
//! the app's level. Keys are stored in it as the extension stores them in chrome.storage.local.

use ptbr_app_wire::Prefs;
use ptbr_ext::types::{Level, ProviderId, Settings};
use serde_json::{Value, json};
use std::path::Path;

pub const FILE: &str = "settings.json";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stored {
    pub settings: Settings,
    pub level: Level,
}

impl Default for Stored {
    fn default() -> Self {
        Stored { settings: Settings::default(), level: Level::Descritivo }
    }
}

impl Stored {
    /// A field missing or malformed keeps its default.
    pub fn from_value(v: &Value) -> Stored {
        let level = v.get("level").and_then(Value::as_str).and_then(Level::parse).unwrap_or(Level::Descritivo);
        Stored { settings: Settings::from_stored(v), level }
    }

    pub fn to_value(&self) -> Value {
        let mut v = serde_json::to_value(&self.settings).expect("Settings is JSON");
        v["level"] = json!(self.level.as_str());
        v
    }

    pub fn prefs(&self) -> Prefs {
        let s = &self.settings;
        let per =
            |f: &dyn Fn(ProviderId) -> String| ProviderId::ALL.iter().map(|&p| (p.as_str().into(), f(p))).collect();
        Prefs {
            provider: s.provider.as_str().into(),
            keys: per(&|p| s.keys[p].clone()),
            models: per(&|p| s.models[p].clone()),
            tone: s.tone.as_str().into(),
            rewrite: s.rewrite,
            level: self.level.as_str().into(),
        }
    }

    /// The window's prefs over these settings, checked as a stored file is.
    pub fn with_prefs(&self, p: &Prefs) -> Stored {
        let mut v = self.to_value();
        v["provider"] = json!(p.provider);
        v["keys"] = json!(p.keys);
        v["models"] = json!(p.models);
        v["tone"] = json!(p.tone);
        v["rewrite"] = json!(p.rewrite);
        v["level"] = json!(p.level);
        Stored::from_value(&v)
    }

    /// The defaults when the file is missing or is not JSON.
    pub fn load(dir: &Path) -> Stored {
        std::fs::read_to_string(dir.join(FILE))
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .map_or_else(Stored::default, |v| Stored::from_value(&v))
    }

    pub fn save(&self, dir: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(dir)?;
        let body = serde_json::to_string_pretty(&self.to_value()).expect("JSON");
        std::fs::write(dir.join(FILE), body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefs_round_trip() {
        let mut p = Stored::default().prefs();
        p.provider = "deepseek".into();
        p.keys.insert("deepseek".into(), "sk-placeholder".into());
        p.models.insert("claude".into(), "claude-opus-5-5".into());
        p.tone = "formal".into();
        p.rewrite = false;
        p.level = "científico".into();
        let s = Stored::default().with_prefs(&p);
        assert_eq!(s.settings.provider, ProviderId::Deepseek);
        assert_eq!(s.settings.keys.deepseek, "sk-placeholder");
        assert_eq!(s.level, Level::Cientifico);
        assert_eq!(s.prefs(), p);
        assert_eq!(Stored::from_value(&s.to_value()), s);
    }

    #[test]
    fn bad_values_keep_the_defaults() {
        let mut p = Stored::default().prefs();
        p.provider = "nenhum".into();
        p.models.insert("claude".into(), "gpt".into());
        p.level = "raso".into();
        let s = Stored::default().with_prefs(&p);
        assert_eq!(s, Stored::default());
    }

    #[test]
    fn a_missing_or_broken_file_is_the_defaults() {
        let dir = std::env::temp_dir().join(format!("ptbr-app-store-{}", std::process::id()));
        assert_eq!(Stored::load(&dir), Stored::default());
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(FILE), "{").unwrap();
        assert_eq!(Stored::load(&dir), Stored::default());
        let s = Stored { level: Level::Leve, ..Stored::default() };
        s.save(&dir).unwrap();
        assert_eq!(Stored::load(&dir), s);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
