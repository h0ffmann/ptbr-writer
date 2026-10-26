//! The wire types. Their JSON is the TypeScript version's: chrome.storage.local already holds a
//! Settings in that shape, and the messages between the content script and the background too.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Level {
    #[serde(rename = "estrito")]
    Estrito,
    #[serde(rename = "descritivo")]
    Descritivo,
    #[serde(rename = "leve")]
    Leve,
    #[serde(rename = "científico")]
    Cientifico,
}

impl Level {
    pub const ALL: [Level; 4] = [Level::Estrito, Level::Descritivo, Level::Leve, Level::Cientifico];

    /// The value of a `<select>` back as a level.
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|l| l.as_str() == s)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Level::Estrito => "estrito",
            Level::Descritivo => "descritivo",
            Level::Leve => "leve",
            Level::Cientifico => "científico",
        }
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderId {
    Claude,
    Deepseek,
    Ollama,
    Ollaya,
}

impl ProviderId {
    pub const ALL: [ProviderId; 4] = [ProviderId::Claude, ProviderId::Deepseek, ProviderId::Ollama, ProviderId::Ollaya];

    pub fn as_str(self) -> &'static str {
        match self {
            ProviderId::Claude => "claude",
            ProviderId::Deepseek => "deepseek",
            ProviderId::Ollama => "ollama",
            ProviderId::Ollaya => "ollaya",
        }
    }

    /// The value of a `<select>` back as a provider.
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.as_str() == s)
    }

    /// Ollama and Ollaya run on this machine: no key, any model they have installed.
    pub fn is_local(self) -> bool {
        matches!(self, ProviderId::Ollama | ProviderId::Ollaya)
    }
}

impl fmt::Display for ProviderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The register a correction keeps (Settings.tone).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tone {
    Informal,
    Neutro,
    Formal,
}

impl Tone {
    pub const ALL: [Tone; 3] = [Tone::Informal, Tone::Neutro, Tone::Formal];

    pub fn as_str(self) -> &'static str {
        match self {
            Tone::Informal => "informal",
            Tone::Neutro => "neutro",
            Tone::Formal => "formal",
        }
    }

    /// The value of a `<select>` back as a tone.
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|t| t.as_str() == s)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Erro,
    Aviso,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Erro => "erro",
            Severity::Aviso => "aviso",
        }
    }
}

/// The conversation style of a rewrite, which Ollaya picks for Ollama (Settings.ollaya_style).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Style {
    #[serde(rename = "conciso")]
    Conciso,
    #[serde(rename = "socrático")]
    Socratico,
    #[serde(rename = "amigável")]
    Amigavel,
}

impl Style {
    pub const ALL: [Style; 3] = [Style::Conciso, Style::Socratico, Style::Amigavel];

    pub fn as_str(self) -> &'static str {
        match self {
            Style::Conciso => "conciso",
            Style::Socratico => "socrático",
            Style::Amigavel => "amigável",
        }
    }
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Suggestion {
    pub original: String,
    pub replacement: String,
    /// A decision model (Ollaya) names the rule but writes no replacement: no Aceitar.
    #[serde(rename = "detectOnly", default, skip_serializing_if = "std::ops::Not::not")]
    pub detect_only: bool,
    pub rule: String,
    pub severity: Severity,
    /// 1-based.
    pub occurrence: u32,
}

/// A suggestion found in the text sent: `start`/`end` in UTF-16 code units (utf16.rs).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Located {
    #[serde(flatten)]
    pub suggestion: Suggestion,
    pub start: u32,
    pub end: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SiteConfig {
    pub enabled: bool,
    pub level: Level,
}

/// One value per provider: the keys and the models of Settings.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PerProvider<T> {
    pub claude: T,
    pub deepseek: T,
    pub ollama: T,
    pub ollaya: T,
}

impl<T> PerProvider<T> {
    pub fn get(&self, p: ProviderId) -> &T {
        match p {
            ProviderId::Claude => &self.claude,
            ProviderId::Deepseek => &self.deepseek,
            ProviderId::Ollama => &self.ollama,
            ProviderId::Ollaya => &self.ollaya,
        }
    }

    pub fn get_mut(&mut self, p: ProviderId) -> &mut T {
        match p {
            ProviderId::Claude => &mut self.claude,
            ProviderId::Deepseek => &mut self.deepseek,
            ProviderId::Ollama => &mut self.ollama,
            ProviderId::Ollaya => &mut self.ollaya,
        }
    }
}

impl<T> std::ops::Index<ProviderId> for PerProvider<T> {
    type Output = T;
    fn index(&self, p: ProviderId) -> &T {
        self.get(p)
    }
}

impl<T> std::ops::IndexMut<ProviderId> for PerProvider<T> {
    fn index_mut(&mut self, p: ProviderId) -> &mut T {
        self.get_mut(p)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    pub enabled: bool,
    pub provider: ProviderId,
    pub keys: PerProvider<String>,
    pub models: PerProvider<String>,
    /// A feature flag: Ollaya picks the style before Ollama writes. Off, or Ollaya down, Ollama writes alone.
    #[serde(rename = "ollayaStyle")]
    pub ollaya_style: bool,
    /// By hostname.
    pub sites: BTreeMap<String, SiteConfig>,
    pub tone: Tone,
    /// true: errors and rewrites; false: errors only.
    pub rewrite: bool,
    /// Phrases a correction must keep as written: 1 to 3 words each, at most 200 (never_fix.rs).
    #[serde(rename = "neverFix")]
    pub never_fix: Vec<String>,
    /// The wait after the last keystroke, in ms: one of settings::DELAYS.
    #[serde(rename = "delayMs")]
    pub delay_ms: u32,
    /// Honored only in a test build (the `test-build` feature).
    #[serde(rename = "apiBase", default, skip_serializing_if = "BTreeMap::is_empty")]
    pub api_base: BTreeMap<ProviderId, String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckError {
    #[serde(rename = "no-key")]
    NoKey,
    #[serde(rename = "local")]
    Local,
    #[serde(rename = "local-ollaya")]
    LocalOllaya,
    #[serde(rename = "auth")]
    Auth,
    #[serde(rename = "rate-limit")]
    RateLimit,
    #[serde(rename = "provider")]
    Provider,
    #[serde(rename = "network")]
    Network,
    /// The extension reloaded under an open tab: its old content script can no longer reach the background.
    #[serde(rename = "reload")]
    Reload,
}

/// What a content script asks the background: `{type: "check", key, text, level}`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckRequest {
    pub key: String,
    pub text: String,
    pub level: Level,
}

/// Every message sent through chrome.runtime.sendMessage, told apart by `type`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Message {
    #[serde(rename = "check")]
    Check(CheckRequest),
    #[serde(rename = "enable-site")]
    EnableSite { host: String },
    #[serde(rename = "disable-site")]
    DisableSite { host: String },
}

/// `{ok: true, suggestions}` or `{ok: false, error}`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "CheckResultWire", into = "CheckResultWire")]
pub enum CheckResult {
    Ok(Vec<Suggestion>),
    Err(CheckError),
}

#[derive(Serialize, Deserialize)]
struct CheckResultWire {
    ok: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    suggestions: Vec<Suggestion>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    error: Option<CheckError>,
}

impl From<CheckResultWire> for CheckResult {
    fn from(w: CheckResultWire) -> Self {
        match (w.ok, w.error) {
            (false, Some(e)) => CheckResult::Err(e),
            (false, None) => CheckResult::Err(CheckError::Provider),
            (true, _) => CheckResult::Ok(w.suggestions),
        }
    }
}

impl From<CheckResult> for CheckResultWire {
    fn from(r: CheckResult) -> Self {
        match r {
            CheckResult::Ok(suggestions) => CheckResultWire { ok: true, suggestions, error: None },
            CheckResult::Err(e) => CheckResultWire { ok: false, suggestions: vec![], error: Some(e) },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn wire_strings_are_the_typescript_ones() {
        assert_eq!(serde_json::to_value(Level::Cientifico).unwrap(), json!("científico"));
        assert_eq!(serde_json::to_value(Style::Socratico).unwrap(), json!("socrático"));
        assert_eq!(serde_json::to_value(Style::Amigavel).unwrap(), json!("amigável"));
        assert_eq!(serde_json::to_value(CheckError::RateLimit).unwrap(), json!("rate-limit"));
        assert_eq!(serde_json::to_value(CheckError::LocalOllaya).unwrap(), json!("local-ollaya"));
        assert_eq!(serde_json::to_value(CheckError::NoKey).unwrap(), json!("no-key"));
        assert_eq!(serde_json::to_value(CheckError::Reload).unwrap(), json!("reload"));
        assert_eq!(serde_json::to_value(Tone::Neutro).unwrap(), json!("neutro"));
        assert_eq!(serde_json::from_value::<ProviderId>(json!("deepseek")).unwrap(), ProviderId::Deepseek);
    }

    #[test]
    fn messages_and_results_keep_their_shape() {
        let m = Message::Check(CheckRequest { key: "f1".into(), text: "oi".into(), level: Level::Leve });
        assert_eq!(
            serde_json::to_value(&m).unwrap(),
            json!({"type": "check", "key": "f1", "text": "oi", "level": "leve"})
        );
        assert_eq!(
            serde_json::from_value::<Message>(json!({"type": "enable-site", "host": "a.com"})).unwrap(),
            Message::EnableSite { host: "a.com".into() }
        );
        assert_eq!(serde_json::to_value(CheckResult::Ok(vec![])).unwrap(), json!({"ok": true}));
        assert_eq!(
            serde_json::to_value(CheckResult::Err(CheckError::Auth)).unwrap(),
            json!({"ok": false, "error": "auth"})
        );
        let ok = json!({"ok": true, "suggestions": [{"original": "a", "replacement": "b", "rule": "R", "severity": "erro", "occurrence": 2}]});
        let CheckResult::Ok(s) = serde_json::from_value(ok).unwrap() else { panic!() };
        assert_eq!(s[0].occurrence, 2);
        assert!(!s[0].detect_only);
    }

    #[test]
    fn detect_only_is_present_only_when_true() {
        let mut s = Suggestion {
            original: "a".into(),
            replacement: String::new(),
            detect_only: true,
            rule: "R".into(),
            severity: Severity::Aviso,
            occurrence: 1,
        };
        assert_eq!(serde_json::to_value(&s).unwrap()["detectOnly"], json!(true));
        s.detect_only = false;
        assert!(serde_json::to_value(&s).unwrap().get("detectOnly").is_none());
    }

    #[test]
    fn located_flattens_its_suggestion() {
        let l = Located {
            suggestion: Suggestion {
                original: "a".into(),
                replacement: "b".into(),
                detect_only: false,
                rule: "R".into(),
                severity: Severity::Erro,
                occurrence: 1,
            },
            start: 3,
            end: 4,
        };
        let v = serde_json::to_value(&l).unwrap();
        assert_eq!(v["original"], json!("a"));
        assert_eq!(v["start"], json!(3));
        assert_eq!(serde_json::from_value::<Located>(v).unwrap(), l);
    }
}
