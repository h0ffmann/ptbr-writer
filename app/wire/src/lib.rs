//! What the window (ui, wasm) and the Rust side (src-tauri) send each other through Tauri's
//! `invoke`. The window holds no logic: it shows a `View` and sends back a card's id.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// What the window needs to draw itself, once at start.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Boot {
    pub version: String,
    pub levels: Vec<String>,
    pub tones: Vec<String>,
    pub providers: Vec<ProviderInfo>,
    pub prefs: Prefs,
    /// A text to check at once: the screenshot build only (`demo` feature).
    pub demo: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    /// Ollama and Ollaya run on this machine and take no key.
    pub local: bool,
    /// The models offered; the first is the default.
    pub models: Vec<String>,
}

/// The settings the window edits; the Rust side checks them as the extension's `from_stored` does.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Prefs {
    pub provider: String,
    /// By provider id.
    pub keys: BTreeMap<String, String>,
    pub models: BTreeMap<String, String>,
    pub tone: String,
    /// true: errors and rewrites; false: errors only.
    pub rewrite: bool,
    pub level: String,
}

/// The checked text: its pieces in order, each underlined or not, and one card per underline.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct View {
    pub text: String,
    pub segments: Vec<Segment>,
    pub cards: Vec<Card>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Segment {
    pub text: String,
    /// The card this piece is underlined for.
    pub card: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Card {
    pub id: usize,
    pub rule: String,
    /// The rule's title in the skill; empty when the skill has none.
    pub title: String,
    /// `erro` or `aviso`.
    pub severity: String,
    pub original: String,
    pub replacement: String,
    /// A decision model (Ollaya) names the rule but writes no replacement: no Aceitar.
    pub detect_only: bool,
}
