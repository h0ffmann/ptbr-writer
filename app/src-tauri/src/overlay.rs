//! The Android overlay's session: the text of the field the user is typing in, checked with Claude,
//! and its cards. The Kotlin service (gen/android, `OverlayService`) only draws the `View`s these
//! return, through JNI (android.rs); everything here builds and tests on the host.

use crate::store::Stored;
use crate::view::Session;
use crate::{message, run_check};
use ptbr_ext::types::{CheckResult, ProviderId, Settings};
use serde_json::json;
use std::path::Path;
use std::sync::Mutex;

/// One field at a time: the last one checked. A panic in a check (android.rs catches it) leaves the
/// lock poisoned; the session in it is still whole, so the lock is taken back.
static SESSION: Mutex<Session> = Mutex::new(Session { text: String::new(), suggestions: Vec::new() });

/// On Android the check goes to Claude only, from the phone, with the user's key.
pub fn claude_only(mut s: Settings) -> Settings {
    s.provider = ProviderId::Claude;
    s
}

/// `{"view": View}`, or `{"error": message}` with the session left as it was.
pub fn check(dir: &Path, text: &str) -> String {
    let stored = Stored::load(dir);
    match run_check(text, stored.level, &claude_only(stored.settings)) {
        CheckResult::Ok(suggestions) => {
            let mut session = SESSION.lock().unwrap_or_else(|e| e.into_inner());
            *session = Session::new(text.into(), suggestions);
            json!({ "view": session.view() }).to_string()
        }
        CheckResult::Err(e) => json!({ "error": message(e) }).to_string(),
    }
}

pub fn accept(id: usize) -> String {
    let mut session = SESSION.lock().unwrap_or_else(|e| e.into_inner());
    session.accept(id);
    json!({ "view": session.view() }).to_string()
}

pub fn ignore(id: usize) -> String {
    let mut session = SESSION.lock().unwrap_or_else(|e| e.into_inner());
    session.ignore(id);
    json!({ "view": session.view() }).to_string()
}

/// The field changed or lost focus: no card points at it any more.
pub fn clear() {
    *SESSION.lock().unwrap_or_else(|e| e.into_inner()) = Session::default();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::FILE;
    use serde_json::Value;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("ptbr-app-overlay-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn any_provider_becomes_claude() {
        let s = Settings { provider: ProviderId::Ollama, ..Settings::default() };
        assert_eq!(claude_only(s).provider, ProviderId::Claude);
    }

    /// Ollama in the file, no Claude key: Claude is asked, and refuses before sending.
    #[test]
    fn without_a_claude_key_the_check_is_an_error_and_sends_nothing() {
        let dir = temp_dir("nokey");
        std::fs::write(dir.join(FILE), r#"{"provider":"ollama"}"#).unwrap();
        let out: Value = serde_json::from_str(&check(&dir, "Um texto qualquer aqui.")).unwrap();
        assert_eq!(out["error"], message(ptbr_ext::types::CheckError::NoKey));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}

/// The overlay over real HTTP, against the fake Claude (the demo build's apiBase).
#[cfg(all(test, feature = "demo"))]
mod http_tests {
    use super::*;
    use crate::fake;
    use crate::store::FILE;
    use ptbr_app_wire::View;
    use serde_json::Value;
    use std::net::TcpListener;

    #[test]
    fn check_accept_ignore_through_the_session() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        std::thread::spawn(move || fake::serve(listener));
        let dir = std::env::temp_dir().join(format!("ptbr-app-overlay-http-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let settings =
            json!({ "provider": "claude", "keys": { "claude": "placeholder" }, "apiBase": { "claude": base } });
        std::fs::write(dir.join(FILE), settings.to_string()).unwrap();

        let out: Value = serde_json::from_str(&check(&dir, fake::DEMO_TEXT)).unwrap();
        let view: View = serde_json::from_value(out["view"].clone()).expect("a view");
        assert_eq!(view.cards.len(), 2);
        assert_eq!(view.cards[0].rule, "PTW-1");

        let out: Value = serde_json::from_str(&accept(0)).unwrap();
        let view: View = serde_json::from_value(out["view"].clone()).unwrap();
        assert!(view.text.starts_with("A equipe decidiu que o deploy"));
        assert_eq!(view.cards.len(), 1);

        let out: Value = serde_json::from_str(&ignore(0)).unwrap();
        assert_eq!(out["view"]["cards"].as_array().map(Vec::len), Some(0));
        clear();
        let out: Value = serde_json::from_str(&accept(0)).unwrap();
        assert_eq!(out["view"]["text"], "");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
