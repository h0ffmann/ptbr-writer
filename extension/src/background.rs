//! The background: a check from a content script becomes the provider's request(s) and its reply
//! becomes suggestions; sites the user enables get their content script registered.

use crate::decisions::{read_decision, read_style};
use crate::providers::{
    Abort, Failed, Fetch, HttpRequest, HttpResponse, build_request, classify_error, decide_requests, is_truncated,
    read_text, style_request,
};
use crate::rules::{PromptOptions, build_system_prompt};
use crate::settings::level_for;
use crate::sites::is_default_host;
use crate::skills::{CIENTIFICO, SKILL};
use crate::suggestions::{filter_suggestions, parse_suggestions};
use crate::types::{CheckError, CheckRequest, CheckResult, ProviderId, Settings, Style, Suggestion};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;

thread_local! {
    // The check in flight per key: a new one for the same key aborts it.
    static INFLIGHT: RefCell<HashMap<String, Abort>> = RefCell::new(HashMap::new());
}

/// A check: `Err` when the provider needs a key it does not have, did not answer, refused, or
/// answered something that is not a complete reply. An aborted check is `Ok` with nothing.
pub async fn handle_check<F: Fetch>(req: &CheckRequest, fetch: &F, settings: &Settings) -> CheckResult {
    let provider = settings.provider;
    if !provider.is_local() && settings.keys[provider].is_empty() {
        return CheckResult::Err(CheckError::NoKey);
    }
    let abort = Abort::default();
    INFLIGHT.with(|m| {
        if let Some(previous) = m.borrow_mut().insert(req.key.clone(), abort.clone()) {
            previous.abort();
        }
    });
    let result = check(req, fetch, settings, &abort).await;
    INFLIGHT.with(|m| {
        let mut m = m.borrow_mut();
        if m.get(&req.key).is_some_and(|a| a.same(&abort)) {
            m.remove(&req.key);
        }
    });
    result
}

const NOTHING: CheckResult = CheckResult::Ok(vec![]);

/// The reply's JSON when it is a 2xx; else the error for its status.
async fn fetch_json<F: Fetch>(fetch: &F, req: HttpRequest, abort: &Abort, p: ProviderId) -> Result<Value, CheckError> {
    let res: HttpResponse = fetch.fetch(req, abort).await.map_err(|Failed| classify_error(None, Some(p)))?;
    if !res.ok() {
        return Err(classify_error(Some(res.status), Some(p)));
    }
    serde_json::from_str(&res.body).map_err(|_| CheckError::Provider)
}

async fn check<F: Fetch>(req: &CheckRequest, fetch: &F, settings: &Settings, abort: &Abort) -> CheckResult {
    let provider = settings.provider;
    if provider == ProviderId::Ollaya {
        return check_decisions(req, fetch, settings, abort).await;
    }
    let style = if provider == ProviderId::Ollama && settings.ollaya_style {
        pick_style(&req.text, fetch, settings, abort).await
    } else {
        None
    };
    if abort.aborted() {
        return NOTHING;
    }
    let opts = PromptOptions { tone: settings.tone, rewrite: settings.rewrite, never_fix: &settings.never_fix, style };
    let Ok(system) = build_system_prompt(req.level, SKILL, CIENTIFICO, &opts) else {
        return CheckResult::Err(CheckError::Provider);
    };
    let json = match fetch_json(fetch, build_request(provider, settings, &system, &req.text), abort, provider).await {
        Ok(json) => json,
        Err(_) if abort.aborted() => return NOTHING,
        Err(e) => return CheckResult::Err(e),
    };
    if abort.aborted() {
        return NOTHING;
    }
    if is_truncated(provider, &json) {
        return CheckResult::Err(CheckError::Provider);
    }
    let parsed = parse_suggestions(&read_text(provider, &json));
    let mut suggestions = filter_suggestions(&req.text, &parsed, settings.rewrite, &settings.never_fix);
    // The card says which style Ollaya chose, so the rewrite's tone has a visible reason.
    if let Some(style) = style {
        for s in &mut suggestions {
            s.rule = format!("{} · estilo {style}", s.rule);
        }
    }
    CheckResult::Ok(suggestions)
}

/// Ollaya + Ollama: Ollaya picks the style of the rewrite. A feature flag, so any failure (Ollaya
/// down, a 403, an odd reply) gives `None` and Ollama writes as it does alone.
async fn pick_style<F: Fetch>(text: &str, fetch: &F, settings: &Settings, abort: &Abort) -> Option<Style> {
    let json = fetch_json(fetch, style_request(settings, text), abort, ProviderId::Ollaya).await.ok()?;
    read_style(&json)
}

/// Ollaya: one /api/decide per sentence. Any failure fails the check, as one failed reply does
/// for the other providers: a sentence left unread is not "no problems".
async fn check_decisions<F: Fetch>(req: &CheckRequest, fetch: &F, settings: &Settings, abort: &Abort) -> CheckResult {
    let Ok(requests) = decide_requests(settings, req.level, &req.text) else {
        return CheckResult::Err(CheckError::Provider);
    };
    let asks = requests.into_iter().map(|(ask, http)| async move {
        let json = fetch_json(fetch, http, abort, ProviderId::Ollaya).await?;
        if is_truncated(ProviderId::Ollaya, &json) {
            return Err(CheckError::Provider);
        }
        Ok(read_decision(&json, &ask))
    });
    let results: Vec<Result<Option<Suggestion>, CheckError>> = join_all(asks).await;
    if abort.aborted() {
        return NOTHING;
    }
    let mut found = vec![];
    for r in results {
        match r {
            Err(e) => return CheckResult::Err(e),
            Ok(Some(s)) => found.push(s),
            Ok(None) => {}
        }
    }
    CheckResult::Ok(filter_suggestions(&req.text, &found, settings.rewrite, &settings.never_fix))
}

/// Every future at once, their outputs in order; a Promise.all without a runtime.
async fn join_all<T, Fut: Future<Output = T>>(futures: impl IntoIterator<Item = Fut>) -> Vec<T> {
    use std::pin::pin;
    use std::task::Poll;
    let mut pending: Vec<(std::pin::Pin<Box<Fut>>, Option<T>)> =
        futures.into_iter().map(|f| (Box::pin(f), None)).collect();
    let mut all = pin!(std::future::poll_fn(|cx| {
        let mut done = true;
        for (f, out) in pending.iter_mut().filter(|(_, out)| out.is_none()) {
            match f.as_mut().poll(cx) {
                Poll::Ready(v) => *out = Some(v),
                Poll::Pending => done = false,
            }
        }
        if done { Poll::Ready(()) } else { Poll::Pending }
    }));
    all.as_mut().await;
    pending.into_iter().map(|(_, out)| out.unwrap()).collect()
}

/// The part of chrome.runtime.MessageSender the gate reads.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sender {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tab: Option<SenderTab>,
    #[serde(rename = "frameId", default, skip_serializing_if = "Option::is_none")]
    pub frame_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SenderTab {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// The hostname of a URL: after the scheme, before the path, without userinfo or port.
pub(crate) fn host_of_url(url: &str) -> Option<&str> {
    let (_, rest) = url.split_once("://")?;
    let authority = rest.split(['/', '?', '#']).next()?;
    let host = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
    let host = if host.starts_with('[') {
        host.split_once(']').map_or(host, |(h, _)| &h[1..])
    } else {
        host.split(':').next().unwrap_or(host)
    };
    (!host.is_empty()).then_some(host)
}

fn sender_host(sender: &Sender) -> Option<&str> {
    sender.tab.as_ref().and_then(|t| t.url.as_deref()).or(sender.url.as_deref()).and_then(host_of_url)
}

/// A check from a content script: the sender's site must be enabled now (an open tab may have read
/// older settings), and its level comes from the settings, not from the message. Keys are per tab
/// and frame, so two tabs never cancel each other.
pub async fn check_from_sender<F: Fetch>(
    msg: &CheckRequest,
    sender: &Sender,
    fetch: &F,
    settings: &Settings,
) -> CheckResult {
    let Some(level) = sender_host(sender).and_then(|host| level_for(host, settings)) else {
        return NOTHING;
    };
    let opt = |v: Option<i64>| v.map_or("undefined".to_string(), |v| v.to_string());
    let key = format!("{}:{}:{}", opt(sender.tab.as_ref().and_then(|t| t.id)), opt(sender.frame_id), msg.key);
    handle_check(&CheckRequest { key, text: msg.text.clone(), level }, fetch, settings).await
}

// --- Content script registration for sites the user enables ---

/// A chrome.scripting.RegisteredContentScript.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentScript {
    pub id: String,
    pub matches: Vec<String>,
    pub js: Vec<String>,
    pub css: Vec<String>,
    #[serde(rename = "runAt")]
    pub run_at: String,
    #[serde(rename = "persistAcrossSessions")]
    pub persist_across_sessions: bool,
}

/// The part of chrome.scripting this module uses.
pub trait Scripting {
    /// The ids registered, all of them or those among `ids`.
    fn registered(&self, ids: Option<&[String]>) -> impl Future<Output = Result<Vec<String>, Failed>>;
    fn register(&self, scripts: Vec<ContentScript>) -> impl Future<Output = Result<(), Failed>>;
    fn unregister(&self, ids: &[String]) -> impl Future<Output = Result<(), Failed>>;
}

fn site_id(host: &str) -> String {
    format!("site:{host}")
}

fn site_script(host: &str) -> ContentScript {
    ContentScript {
        id: site_id(host),
        matches: vec![format!("https://{host}/*")],
        js: vec!["content.js".into()],
        css: vec!["content.css".into()],
        run_at: "document_idle".into(),
        persist_across_sessions: true,
    }
}

pub async fn enable_site<S: Scripting>(host: &str, scripting: &S) -> Result<(), Failed> {
    if is_default_host(host) {
        return Ok(());
    }
    if !scripting.registered(Some(&[site_id(host)])).await?.is_empty() {
        return Ok(());
    }
    scripting.register(vec![site_script(host)]).await
}

pub async fn disable_site<S: Scripting>(host: &str, scripting: &S) -> Result<(), Failed> {
    if is_default_host(host) {
        return Ok(());
    }
    let id = site_id(host);
    if scripting.registered(Some(std::slice::from_ref(&id))).await?.is_empty() {
        return Ok(());
    }
    scripting.unregister(&[id]).await
}

/// Registers every enabled site not registered yet; at startup and install.
pub async fn sync_sites<S: Scripting>(settings: &Settings, scripting: &S) -> Result<(), Failed> {
    let registered = scripting.registered(None).await?;
    let scripts: Vec<ContentScript> = settings
        .sites
        .iter()
        .filter(|(host, cfg)| cfg.enabled && !is_default_host(host) && !registered.contains(&site_id(host)))
        .map(|(host, _)| site_script(host))
        .collect();
    if scripts.is_empty() { Ok(()) } else { scripting.register(scripts).await }
}

#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use crate::chrome;
    use wasm_bindgen::JsValue;

    /// chrome.scripting.
    pub struct ChromeScripting;

    impl Scripting for ChromeScripting {
        async fn registered(&self, ids: Option<&[String]>) -> Result<Vec<String>, Failed> {
            chrome::get_registered_content_scripts(ids).await.map_err(|_| Failed)
        }

        async fn register(&self, scripts: Vec<ContentScript>) -> Result<(), Failed> {
            chrome::register_content_scripts(&scripts).await.map_err(|_| Failed)
        }

        async fn unregister(&self, ids: &[String]) -> Result<(), Failed> {
            chrome::unregister_content_scripts(ids).await.map_err(|_| Failed)
        }
    }

    /// A runtime message with its sender; the reply. background.js registers the listener itself,
    /// synchronously, before the wasm is loaded: a service worker loses the event that woke it
    /// when the listener comes later. `Err` on a message that is not one of ours: the loader
    /// answers it `{ok: false, error: "provider"}`.
    pub async fn handle_message(msg: JsValue, sender: JsValue) -> Result<JsValue, JsValue> {
        use crate::providers::BrowserFetch;
        use crate::settings::load_settings;
        use crate::types::Message;
        let (host, enable) = match chrome::from_js::<Message>(msg)? {
            Message::Check(req) => {
                let sender: Sender = chrome::from_js(sender).unwrap_or_default();
                let result = match load_settings().await {
                    Ok(settings) => check_from_sender(&req, &sender, &BrowserFetch, &settings).await,
                    Err(_) => CheckResult::Err(CheckError::Provider),
                };
                return chrome::to_js(&result);
            }
            Message::EnableSite { host } => (host, true),
            Message::DisableSite { host } => (host, false),
        };
        let done = if enable {
            enable_site(&host, &ChromeScripting).await
        } else {
            disable_site(&host, &ChromeScripting).await
        };
        chrome::to_js(&serde_json::json!({ "ok": done.is_ok() }))
    }

    /// Registers every enabled site; at startup and install. A failed sync is swallowed, as the
    /// TypeScript's `.catch(() => {})`.
    pub async fn sync_sites_now() {
        if let Ok(s) = crate::settings::load_settings().await {
            let _ = sync_sites(&s, &ChromeScripting).await;
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use browser::{ChromeScripting, handle_message, sync_sites_now};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{block_on, poll_once};
    use crate::types::{Level, PerProvider, Severity, SiteConfig, Tone};
    use serde_json::json;
    use std::cell::RefCell;
    use std::rc::Rc;

    fn settings() -> Settings {
        Settings {
            keys: PerProvider {
                claude: "sk-test".into(),
                deepseek: "sk-test".into(),
                ollama: "".into(),
                ollaya: "".into(),
            },
            ..Settings::default()
        }
    }

    fn prompt_options(s: &Settings) -> PromptOptions<'_> {
        PromptOptions { tone: s.tone, rewrite: s.rewrite, never_fix: &s.never_fix, style: None }
    }

    fn req() -> CheckRequest {
        CheckRequest { key: "k1".into(), text: "texto a nível de teste aqui".into(), level: Level::Estrito }
    }

    fn suggestion() -> Value {
        json!({ "original": "a nível de", "replacement": "em", "rule": "PTW-1", "severity": "aviso", "occurrence": 1 })
    }

    fn claude_body() -> Value {
        json!({ "content": [{ "type": "text", "text": json!({ "suggestions": [suggestion()] }).to_string() }] })
    }

    /// A fetch answering from a closure over the request; it records every request and its Abort.
    struct Fake<R: Fn(&HttpRequest) -> Result<HttpResponse, Failed>> {
        reply: R,
        calls: RefCell<Vec<(HttpRequest, Abort)>>,
    }

    impl<R: Fn(&HttpRequest) -> Result<HttpResponse, Failed>> Fetch for Fake<R> {
        async fn fetch(&self, req: HttpRequest, abort: &Abort) -> Result<HttpResponse, Failed> {
            self.calls.borrow_mut().push((req.clone(), abort.clone()));
            (self.reply)(&req)
        }
    }

    fn fake<R: Fn(&HttpRequest) -> Result<HttpResponse, Failed>>(reply: R) -> Fake<R> {
        Fake { reply, calls: RefCell::new(vec![]) }
    }

    fn ok_fetch(body: Value, status: u16) -> Fake<impl Fn(&HttpRequest) -> Result<HttpResponse, Failed>> {
        fake(move |_| Ok(HttpResponse { status, body: body.to_string() }))
    }

    fn down() -> Fake<impl Fn(&HttpRequest) -> Result<HttpResponse, Failed>> {
        fake(|_| Err(Failed))
    }

    fn ok(r: &CheckResult) -> &Vec<Suggestion> {
        match r {
            CheckResult::Ok(s) => s,
            CheckResult::Err(e) => panic!("{e:?}"),
        }
    }

    #[test]
    fn no_key_does_not_call_fetch() {
        let f = ok_fetch(claude_body(), 200);
        assert_eq!(block_on(handle_check(&req(), &f, &Settings::default())), CheckResult::Err(CheckError::NoKey));
        assert!(f.calls.borrow().is_empty());
    }

    #[test]
    fn status_200_returns_suggestions() {
        let r = block_on(handle_check(&req(), &ok_fetch(claude_body(), 200), &settings()));
        assert_eq!(ok(&r).len(), 1);
    }

    #[test]
    fn aviso_with_an_erro_inside_it_returns_1_suggestion() {
        let erro = json!({ "original": "nível", "replacement": "nivel", "rule": "PTW-2", "severity": "erro", "occurrence": 1 });
        let body = json!({ "content": [{ "type": "text", "text": json!({ "suggestions": [suggestion(), erro] }).to_string() }] });
        let r = block_on(handle_check(&req(), &ok_fetch(body, 200), &settings()));
        assert_eq!(ok(&r).iter().map(|s| s.original.as_str()).collect::<Vec<_>>(), ["a nível de"]);
    }

    #[test]
    fn ollama_and_ollaya_replies_pass_through_filter_suggestions() {
        let ollama = Settings { provider: ProviderId::Ollama, never_fix: vec!["nível".into()], ..Settings::default() };
        let chat = json!({ "message": { "content": json!([suggestion()]).to_string() }, "done_reason": "stop" });
        assert_eq!(block_on(handle_check(&req(), &ok_fetch(chat, 200), &ollama)), CheckResult::Ok(vec![]));
        let mut gone = suggestion();
        gone["original"] = json!("não está no texto");
        let missing = json!({ "message": { "content": json!([gone]).to_string() }, "done_reason": "stop" });
        let plain = Settings { provider: ProviderId::Ollama, ..Settings::default() };
        assert_eq!(block_on(handle_check(&req(), &ok_fetch(missing, 200), &plain)), CheckResult::Ok(vec![]));
        // A detection is not a rewrite: "só erros" and the never-fix list keep it.
        let ollaya = Settings {
            provider: ProviderId::Ollaya,
            rewrite: false,
            never_fix: vec!["deploy".into()],
            ..Settings::default()
        };
        let text = "Foi decidido que o deploy será feito.";
        let body = json!({ "answers": { "regra": { "type": "choice", "choice": "PTW-1", "probabilities": { "PTW-1": 0.9 } } }, "state_truncated": false });
        let r = block_on(handle_check(&CheckRequest { text: text.into(), ..req() }, &ok_fetch(body, 200), &ollaya));
        assert_eq!(ok(&r).iter().map(|s| s.original.as_str()).collect::<Vec<_>>(), [text]);
    }

    #[test]
    fn status_401_auth_429_rate_limit() {
        assert_eq!(
            block_on(handle_check(&req(), &ok_fetch(json!({}), 401), &settings())),
            CheckResult::Err(CheckError::Auth)
        );
        assert_eq!(
            block_on(handle_check(&req(), &ok_fetch(json!({}), 429), &settings())),
            CheckResult::Err(CheckError::RateLimit)
        );
    }

    #[test]
    fn rejecting_fetch_network() {
        assert_eq!(block_on(handle_check(&req(), &down(), &settings())), CheckResult::Err(CheckError::Network));
    }

    #[test]
    fn ollama_checks_without_a_key_down_or_403_local() {
        let ollama = Settings { provider: ProviderId::Ollama, ..Settings::default() };
        let body = json!({ "message": { "content": json!({ "suggestions": [suggestion()] }).to_string() }, "done_reason": "stop" });
        let r = block_on(handle_check(&req(), &ok_fetch(body, 200), &ollama));
        assert_eq!(ok(&r).len(), 1);
        assert_eq!(
            block_on(handle_check(&req(), &ok_fetch(json!({}), 403), &ollama)),
            CheckResult::Err(CheckError::Local)
        );
        assert_eq!(block_on(handle_check(&req(), &down(), &ollama)), CheckResult::Err(CheckError::Local));
    }

    #[test]
    fn ollaya_one_decide_request_per_sentence_detections_back_down_or_403_local_ollaya_truncated_provider() {
        let ollaya = Settings { provider: ProviderId::Ollaya, ..Settings::default() };
        let text = "Foi decidido que o deploy será feito. A equipe valida o build hoje.";
        let first = "Foi decidido que o deploy será feito.";
        let body = json!({ "answers": { "regra": { "type": "choice", "choice": "PTW-1", "probabilities": { "PTW-1": 0.9 } } }, "state_truncated": false });
        let f = ok_fetch(body.clone(), 200);
        let r = block_on(handle_check(&CheckRequest { text: text.into(), ..req() }, &f, &ollaya));
        assert_eq!(
            ok(&r)[0],
            Suggestion {
                original: first.into(),
                replacement: String::new(),
                detect_only: true,
                rule: "PTW-1 (90%)".into(),
                severity: Severity::Aviso,
                occurrence: 1
            }
        );
        assert_eq!(
            ok(&r).iter().map(|s| s.original.as_str()).collect::<Vec<_>>(),
            [first, "A equipe valida o build hoje."]
        );
        let calls = f.calls.borrow();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].0.url, "http://127.0.0.1:11435/api/decide");
        let sent: Value = serde_json::from_str(&calls[0].0.body).unwrap();
        assert_eq!(sent["model"], "decima");
        assert_eq!(sent["state"], first);
        assert_eq!(
            block_on(handle_check(&req(), &ok_fetch(json!({}), 403), &ollaya)),
            CheckResult::Err(CheckError::LocalOllaya)
        );
        assert_eq!(block_on(handle_check(&req(), &down(), &ollaya)), CheckResult::Err(CheckError::LocalOllaya));
        let mut cut = body;
        cut["state_truncated"] = json!(true);
        assert_eq!(
            block_on(handle_check(&CheckRequest { text: text.into(), ..req() }, &ok_fetch(cut, 200), &ollaya)),
            CheckResult::Err(CheckError::Provider)
        );
    }

    #[test]
    fn ollaya_plus_ollama_ollaya_picks_the_style_ollama_writes_in_it_ollaya_down_ollama_writes_alone() {
        let both = Settings { provider: ProviderId::Ollama, ollaya_style: true, ..Settings::default() };
        let chat = json!({ "message": { "content": json!([suggestion()]).to_string() }, "done_reason": "stop" });
        let c = chat.clone();
        let f = fake(move |r| {
            let body = if r.url.ends_with("/api/decide") {
                json!({ "answers": { "estilo": { "choice": "conciso" } } })
            } else {
                c.clone()
            };
            Ok(HttpResponse { status: 200, body: body.to_string() })
        });
        let r = block_on(handle_check(&req(), &f, &both));
        assert_eq!(ok(&r)[0].rule, "PTW-1 · estilo conciso");
        let calls = f.calls.borrow();
        assert_eq!(calls[0].0.url, "http://127.0.0.1:11435/api/decide");
        assert_eq!(calls[1].0.url, "http://127.0.0.1:11434/api/chat");
        let sent: Value = serde_json::from_str(&calls[1].0.body).unwrap();
        assert!(sent["messages"][0]["content"].as_str().unwrap().contains("Estilo da reescrita: `conciso`"));
        let down = fake(move |r| {
            if r.url.ends_with("/api/decide") {
                Err(Failed)
            } else {
                Ok(HttpResponse { status: 200, body: chat.to_string() })
            }
        });
        let alone = block_on(handle_check(&req(), &down, &both));
        assert_eq!(ok(&alone)[0].rule, "PTW-1");
    }

    #[test]
    fn a_truncated_reply_provider_not_no_problems() {
        let mut cut = claude_body();
        cut["stop_reason"] = json!("max_tokens");
        assert_eq!(
            block_on(handle_check(&req(), &ok_fetch(cut, 200), &settings())),
            CheckResult::Err(CheckError::Provider)
        );
        let ds = json!({ "choices": [{ "finish_reason": "length", "message": { "content": "[{\"original\":" } }] });
        let ds_settings = Settings { provider: ProviderId::Deepseek, ..settings() };
        assert_eq!(
            block_on(handle_check(&req(), &ok_fetch(ds, 200), &ds_settings)),
            CheckResult::Err(CheckError::Provider)
        );
    }

    #[test]
    fn status_2xx_with_non_json_body_provider() {
        let f = fake(|_| Ok(HttpResponse { status: 200, body: "<html>".into() }));
        assert_eq!(block_on(handle_check(&req(), &f, &settings())), CheckResult::Err(CheckError::Provider));
    }

    #[test]
    fn a_missing_skill_section_is_a_provider_error_not_a_panic() {
        let e =
            build_system_prompt(Level::Cientifico, SKILL, "# x", &prompt_options(&Settings::default())).unwrap_err();
        assert_eq!(e.to_string(), "cientifico/SKILL.md: seção Regras não encontrada");
    }

    /// A fetch pending until its Abort fires, or until `release` is set; then it answers `body`.
    fn pending_fetch(body: Value, release: Rc<std::cell::Cell<bool>>) -> impl Fetch {
        struct Pending {
            body: Value,
            release: Rc<std::cell::Cell<bool>>,
            signals: RefCell<Vec<Abort>>,
        }
        impl Fetch for Pending {
            async fn fetch(&self, _: HttpRequest, abort: &Abort) -> Result<HttpResponse, Failed> {
                self.signals.borrow_mut().push(abort.clone());
                let abort = abort.clone();
                std::future::poll_fn(|_| {
                    if abort.aborted() {
                        std::task::Poll::Ready(Err(Failed))
                    } else if self.release.get() {
                        std::task::Poll::Ready(Ok(HttpResponse { status: 200, body: self.body.to_string() }))
                    } else {
                        std::task::Poll::Pending
                    }
                })
                .await
            }
        }
        Pending { body, release, signals: RefCell::new(vec![]) }
    }

    #[test]
    fn second_call_with_same_key_aborts_the_first_which_returns_nothing() {
        let release = Rc::new(std::cell::Cell::new(false));
        let f = pending_fetch(claude_body(), release.clone());
        let (s, r) = (settings(), req());
        let mut first = Box::pin(handle_check(&r, &f, &s));
        assert!(poll_once(&mut first).is_none());
        let mut second = Box::pin(handle_check(&r, &f, &s));
        assert!(poll_once(&mut second).is_none());
        // The first was aborted by the second and gives up; the second waits for the server.
        assert_eq!(poll_once(&mut first), Some(CheckResult::Ok(vec![])));
        release.set(true);
        let r2 = block_on(second);
        assert_eq!(ok(&r2).len(), 1);
    }

    fn scripting_stub(registered: &[&str]) -> Stub {
        Stub {
            ids: RefCell::new(registered.iter().map(|s| s.to_string()).collect()),
            registers: RefCell::new(vec![]),
            unregisters: RefCell::new(vec![]),
        }
    }

    struct Stub {
        ids: RefCell<std::collections::BTreeSet<String>>,
        registers: RefCell<Vec<Vec<ContentScript>>>,
        unregisters: RefCell<Vec<Vec<String>>>,
    }

    impl Scripting for Stub {
        async fn registered(&self, ids: Option<&[String]>) -> Result<Vec<String>, Failed> {
            Ok(self.ids.borrow().iter().filter(|id| ids.is_none_or(|ids| ids.contains(id))).cloned().collect())
        }
        async fn register(&self, scripts: Vec<ContentScript>) -> Result<(), Failed> {
            self.ids.borrow_mut().extend(scripts.iter().map(|s| s.id.clone()));
            self.registers.borrow_mut().push(scripts);
            Ok(())
        }
        async fn unregister(&self, ids: &[String]) -> Result<(), Failed> {
            for id in ids {
                self.ids.borrow_mut().remove(id);
            }
            self.unregisters.borrow_mut().push(ids.to_vec());
            Ok(())
        }
    }

    #[test]
    fn enable_site_registers_site_host_for_https_host() {
        let s = scripting_stub(&[]);
        block_on(enable_site("example.com", &s)).unwrap();
        let registers = s.registers.borrow();
        assert_eq!(registers.len(), 1);
        let script = &registers[0][0];
        assert_eq!(script.id, "site:example.com");
        assert_eq!(script.matches, ["https://example.com/*"]);
        assert_eq!(script.js, ["content.js"]);
        assert_eq!(script.css, ["content.css"]);
        assert_eq!(script.run_at, "document_idle");
        assert!(script.persist_across_sessions);
        assert_eq!(serde_json::to_value(script).unwrap()["runAt"], json!("document_idle"));
    }

    #[test]
    fn never_registers_default_hosts_and_is_idempotent() {
        let s = scripting_stub(&[]);
        block_on(enable_site("github.com", &s)).unwrap();
        block_on(enable_site("example.com", &s)).unwrap();
        block_on(enable_site("example.com", &s)).unwrap();
        assert_eq!(s.registers.borrow().len(), 1);
    }

    #[test]
    fn disable_site_unregisters() {
        let s = scripting_stub(&["site:example.com"]);
        block_on(disable_site("example.com", &s)).unwrap();
        assert_eq!(*s.unregisters.borrow(), [vec!["site:example.com".to_string()]]);
    }

    #[test]
    fn startup_skips_registered_ids_and_default_hosts() {
        let s = scripting_stub(&["site:a.com"]);
        let mut st = settings();
        st.sites.insert("a.com".into(), SiteConfig { enabled: true, level: Level::Descritivo });
        st.sites.insert("b.com".into(), SiteConfig { enabled: true, level: Level::Descritivo });
        st.sites.insert("c.com".into(), SiteConfig { enabled: false, level: Level::Descritivo });
        block_on(sync_sites(&st, &s)).unwrap();
        let registers = s.registers.borrow();
        assert_eq!(registers.len(), 1);
        assert_eq!(registers[0].iter().map(|x| x.id.as_str()).collect::<Vec<_>>(), ["site:b.com"]);
    }

    fn site(enabled: bool) -> Settings {
        let mut s = settings();
        s.sites.insert("example.com".into(), SiteConfig { enabled, level: Level::Leve });
        s
    }

    fn tab(id: i64) -> Sender {
        Sender {
            tab: Some(SenderTab { id: Some(id), url: Some("https://example.com/x".into()) }),
            frame_id: Some(0),
            url: None,
        }
    }

    #[test]
    fn a_sender_whose_host_is_not_enabled_gets_nothing_and_no_fetch() {
        let f = ok_fetch(claude_body(), 200);
        assert_eq!(block_on(check_from_sender(&req(), &tab(1), &f, &site(false))), CheckResult::Ok(vec![]));
        assert!(f.calls.borrow().is_empty());
    }

    #[test]
    fn the_extension_switched_off_no_fetch() {
        let f = ok_fetch(claude_body(), 200);
        assert_eq!(
            block_on(check_from_sender(&req(), &tab(1), &f, &Settings { enabled: false, ..site(true) })),
            CheckResult::Ok(vec![])
        );
        assert!(f.calls.borrow().is_empty());
    }

    #[test]
    fn a_sender_with_no_url_no_fetch() {
        let f = ok_fetch(claude_body(), 200);
        assert_eq!(block_on(check_from_sender(&req(), &Sender::default(), &f, &site(true))), CheckResult::Ok(vec![]));
        assert!(f.calls.borrow().is_empty());
    }

    #[test]
    fn takes_the_level_from_the_settings_not_from_the_message() {
        let f = ok_fetch(claude_body(), 200);
        block_on(check_from_sender(&CheckRequest { level: Level::Estrito, ..req() }, &tab(1), &f, &site(true)));
        let sent: Value = serde_json::from_str(&f.calls.borrow()[0].0.body).unwrap();
        assert_eq!(
            sent["system"],
            build_system_prompt(Level::Leve, SKILL, CIENTIFICO, &prompt_options(&site(true))).unwrap()
        );
    }

    #[test]
    fn passes_the_tone_rewrite_and_never_fix_settings_into_the_prompt() {
        let f = ok_fetch(claude_body(), 200);
        let s = Settings { tone: Tone::Informal, rewrite: false, never_fix: vec!["Kafka".into()], ..site(true) };
        block_on(check_from_sender(&req(), &tab(1), &f, &s));
        let sent: Value = serde_json::from_str(&f.calls.borrow()[0].0.body).unwrap();
        let system = sent["system"].as_str().unwrap();
        assert!(system.contains("mantenha a voz e as gírias"));
        assert!(system.contains("não devolva nenhum aviso"));
        assert!(system.contains("nunca troque estas palavras, nem dentro de uma frase reescrita: Kafka"));
    }

    #[test]
    fn falls_back_to_sender_url() {
        let f = ok_fetch(claude_body(), 200);
        let sender = Sender { url: Some("https://user@example.com:443/y?q#f".into()), ..Sender::default() };
        let r = block_on(check_from_sender(&req(), &sender, &f, &site(true)));
        assert_eq!(ok(&r).len(), 1);
        assert_eq!(host_of_url("https://[::1]:8080/x"), Some("::1"));
        assert_eq!(host_of_url("nope"), None);
    }

    #[test]
    fn the_same_message_key_from_two_tabs_does_not_abort_either() {
        let release = Rc::new(std::cell::Cell::new(false));
        let f = pending_fetch(claude_body(), release.clone());
        let (s, r, t1, t2) = (site(true), req(), tab(1), tab(2));
        let mut a = Box::pin(check_from_sender(&r, &t1, &f, &s));
        let mut b = Box::pin(check_from_sender(&r, &t2, &f, &s));
        assert!(poll_once(&mut a).is_none());
        assert!(poll_once(&mut b).is_none());
        let inflight = INFLIGHT.with(|m| m.borrow().values().cloned().collect::<Vec<_>>());
        assert_eq!(inflight.len(), 2);
        assert!(inflight.iter().all(|x| !x.aborted()));
        release.set(true);
        for r in [block_on(a), block_on(b)] {
            assert_eq!(ok(&r).len(), 1);
        }
    }
}
