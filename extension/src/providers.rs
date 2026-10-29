//! Each provider's HTTP: the request for a check, the text and the truncation in its reply, the
//! error for its status. Sending (`Fetch`) is the browser's, behind a trait so the background's
//! check runs on the host with a fake.

use crate::decisions::{Ask, decide_asks, style_body};
use crate::rules::SectionError;
use crate::skills::{CIENTIFICO, SKILL};
use crate::types::{CheckError, Level, ProviderId, Settings};
use serde_json::{Value, json};
use std::future::Future;

fn endpoint(p: ProviderId) -> &'static str {
    match p {
        ProviderId::Claude => "https://api.anthropic.com/v1/messages",
        ProviderId::Deepseek => "https://api.deepseek.com/chat/completions",
        // Ollama's own chat API, not /v1: only it takes think: false, and a thinking model would spend
        // max_tokens on its reasoning.
        ProviderId::Ollama => "http://127.0.0.1:11434/api/chat",
        // Ollaya serves decision models: /api/decide answers questions, /api/chat is a 404.
        ProviderId::Ollaya => "http://127.0.0.1:11435/api/decide",
    }
}

/// The two servers on this machine; their /api/tags and /api/pull speak the same JSON.
pub fn local_base(p: ProviderId) -> Option<&'static str> {
    match p {
        ProviderId::Ollama => Some("http://127.0.0.1:11434"),
        ProviderId::Ollaya => Some("http://127.0.0.1:11435"),
        _ => None,
    }
}

/// The host permission a local server needs: optional in the manifest, asked when the user picks it.
pub fn local_origin(p: ProviderId) -> Option<String> {
    local_base(p).map(|b| format!("{b}/*"))
}

/// Where a local server's /api/tags and /api/pull are; `apiBase` in a test build only.
pub fn local_base_for(p: ProviderId, s: &Settings) -> Option<String> {
    let base = if cfg!(feature = "test-build") { s.api_base.get(&p).map(|b| b.trim_end_matches('/')) } else { None };
    base.map(String::from).or_else(|| local_base(p).map(String::from))
}

fn path(p: ProviderId) -> &'static str {
    match p {
        ProviderId::Claude => "/v1/messages",
        ProviderId::Deepseek => "/chat/completions",
        ProviderId::Ollama => "/api/chat",
        ProviderId::Ollaya => "/api/decide",
    }
}

fn url_for(p: ProviderId, s: &Settings) -> String {
    let base = if cfg!(feature = "test-build") { s.api_base.get(&p) } else { None };
    match base {
        Some(base) => format!("{}{}", base.trim_end_matches('/'), path(p)),
        None => endpoint(p).to_string(),
    }
}

/// A POST with a JSON body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HttpRequest {
    pub url: String,
    pub headers: Vec<(&'static str, String)>,
    pub body: String,
}

fn post(url: String, mut headers: Vec<(&'static str, String)>, body: &Value) -> HttpRequest {
    headers.insert(0, ("content-type", "application/json".into()));
    HttpRequest { url, headers, body: body.to_string() }
}

pub fn build_request(p: ProviderId, s: &Settings, system: &str, user: &str) -> HttpRequest {
    let url = url_for(p, s);
    match p {
        ProviderId::Ollama => post(
            url,
            vec![],
            &json!({
                "model": s.models.ollama,
                "stream": false,
                "think": false,
                "options": { "temperature": 0, "num_predict": 1024 },
                "messages": [{ "role": "system", "content": system }, { "role": "user", "content": user }],
            }),
        ),
        ProviderId::Claude => post(
            url,
            vec![
                ("x-api-key", s.keys.claude.clone()),
                ("anthropic-version", "2023-06-01".into()),
                ("anthropic-dangerous-direct-browser-access", "true".into()),
            ],
            // Thinking is on and cannot be turned off on these models, and it counts against max_tokens: low
            // effort and room for it. They refuse temperature with a 400.
            &json!({
                "model": s.models.claude,
                "max_tokens": 16000,
                "output_config": { "effort": "low" },
                "system": system,
                "messages": [{ "role": "user", "content": user }],
            }),
        ),
        // DeepSeek and, when asked for a chat, Ollaya: the OpenAI shape.
        _ => post(
            url,
            vec![("Authorization", format!("Bearer {}", s.keys.deepseek))],
            &json!({
                "model": s.models.deepseek,
                "max_tokens": 2048,
                "temperature": 0,
                // Both DeepSeek models think by default, and the thinking counts against max_tokens: it left the
                // answer empty. A correction needs no chain of thought.
                "thinking": { "type": "disabled" },
                "messages": [{ "role": "system", "content": system }, { "role": "user", "content": user }],
            }),
        ),
    }
}

/// Ollaya's requests: one per sentence, asking which of the level's rules it breaks (decisions.rs).
pub fn decide_requests(s: &Settings, level: Level, text: &str) -> Result<Vec<(Ask, HttpRequest)>, SectionError> {
    let url = url_for(ProviderId::Ollaya, s);
    Ok(decide_asks(&s.models.ollaya, level, text, SKILL, CIENTIFICO)?
        .into_iter()
        .map(|ask| {
            let req = post(url.clone(), vec![], &ask.body);
            (ask, req)
        })
        .collect())
}

/// Ollaya + Ollama: the one question that picks the style Ollama writes in.
pub fn style_request(s: &Settings, text: &str) -> HttpRequest {
    post(url_for(ProviderId::Ollaya, s), vec![], &style_body(&s.models.ollaya, text))
}

/// The model's text in a reply; empty when the shape is not the provider's.
pub fn read_text(p: ProviderId, json: &Value) -> String {
    let text = match p {
        ProviderId::Ollama => {
            json.get("message").and_then(|m| m.get("content")).and_then(Value::as_str).map(String::from)
        }
        ProviderId::Claude => json.get("content").and_then(Value::as_array).map(|blocks| {
            blocks
                .iter()
                .filter(|b| b.get("type").and_then(Value::as_str) == Some("text"))
                .filter_map(|b| b.get("text").and_then(Value::as_str))
                .collect()
        }),
        _ => json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(Value::as_str)
            .map(String::from),
    };
    text.unwrap_or_default()
}

/// True when the reply was cut at max_tokens: its JSON is incomplete, so "no suggestions" would be a lie.
pub fn is_truncated(p: ProviderId, json: &Value) -> bool {
    match p {
        ProviderId::Claude => json.get("stop_reason").and_then(Value::as_str) == Some("max_tokens"),
        ProviderId::Ollama => json.get("done_reason").and_then(Value::as_str) == Some("length"),
        // Ollaya drops part of a long state to fit the model's context: a sentence may have gone unread.
        ProviderId::Ollaya => json.get("state_truncated").and_then(Value::as_bool) == Some(true),
        ProviderId::Deepseek => {
            json.get("choices").and_then(|c| c.get(0)).and_then(|c| c.get("finish_reason")).and_then(Value::as_str)
                == Some("length")
        }
    }
}

/// The error for a status; `None` is a request that never got an answer. Ollama and Ollaya answer
/// 403 to an extension origin they were not started to accept (OLLAMA_ORIGINS, OLLAYA_ORIGINS).
pub fn classify_error(status: Option<u16>, p: Option<ProviderId>) -> CheckError {
    match (p, status) {
        (Some(ProviderId::Ollama), None | Some(403)) => CheckError::Local,
        (Some(ProviderId::Ollaya), None | Some(403)) => CheckError::LocalOllaya,
        (_, None) => CheckError::Network,
        (_, Some(401 | 403)) => CheckError::Auth,
        (_, Some(429)) => CheckError::RateLimit,
        // The request itself: a model the key cannot use, or a parameter the model refuses.
        (_, Some(400 | 404 | 422)) => CheckError::Request,
        (_, Some(500..=599)) => CheckError::Busy,
        _ => CheckError::Provider,
    }
}

/// A request that got no answer: refused, offline, or aborted (then `Abort::aborted` says so).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Failed;

pub struct HttpResponse {
    pub status: u16,
    pub body: String,
}

impl HttpResponse {
    /// 2xx.
    pub fn ok(&self) -> bool {
        (200..300).contains(&self.status)
    }
}

/// An AbortSignal that works without the browser: aborting runs what was registered, once.
#[derive(Clone, Default)]
pub struct Abort(std::rc::Rc<std::cell::RefCell<AbortState>>);

#[derive(Default)]
struct AbortState {
    aborted: bool,
    on_abort: Vec<Box<dyn FnOnce()>>,
}

impl Abort {
    pub fn aborted(&self) -> bool {
        self.0.borrow().aborted
    }

    pub fn abort(&self) {
        let listeners = {
            let mut s = self.0.borrow_mut();
            if s.aborted {
                return;
            }
            s.aborted = true;
            std::mem::take(&mut s.on_abort)
        };
        listeners.into_iter().for_each(|f| f());
    }

    /// Runs `f` on abort, at once when already aborted.
    pub fn on_abort(&self, f: impl FnOnce() + 'static) {
        if self.aborted() { f() } else { self.0.borrow_mut().on_abort.push(Box::new(f)) }
    }

    pub fn same(&self, other: &Abort) -> bool {
        std::rc::Rc::ptr_eq(&self.0, &other.0)
    }
}

/// Sends a request; the browser's fetch on wasm, a fake in tests.
pub trait Fetch {
    fn fetch(&self, req: HttpRequest, abort: &Abort) -> impl Future<Output = Result<HttpResponse, Failed>>;
}

#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use wasm_bindgen::JsCast;
    use wasm_bindgen::prelude::wasm_bindgen;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{AbortController, Headers, RequestInit, Response};

    #[wasm_bindgen]
    extern "C" {
        // The global fetch: a window's or a service worker's.
        #[wasm_bindgen(js_name = fetch)]
        fn fetch_with_init(url: &str, init: &RequestInit) -> js_sys::Promise;
    }

    async fn response(url: &str, init: &RequestInit) -> Result<HttpResponse, Failed> {
        let res: Response = JsFuture::from(fetch_with_init(url, init)).await.map_err(|_| Failed)?.unchecked_into();
        let text = JsFuture::from(res.text().map_err(|_| Failed)?).await.map_err(|_| Failed)?;
        Ok(HttpResponse { status: res.status(), body: text.as_string().unwrap_or_default() })
    }

    /// The browser's fetch, the Abort mapped to an AbortController.
    pub struct BrowserFetch;

    impl Fetch for BrowserFetch {
        async fn fetch(&self, req: HttpRequest, abort: &Abort) -> Result<HttpResponse, Failed> {
            let controller = AbortController::new().map_err(|_| Failed)?;
            let c = controller.clone();
            abort.on_abort(move || c.abort());
            let headers = Headers::new().map_err(|_| Failed)?;
            for (k, v) in &req.headers {
                headers.append(k, v).map_err(|_| Failed)?;
            }
            let init = RequestInit::new();
            init.set_method("POST");
            init.set_headers(&headers);
            init.set_body(&req.body.as_str().into());
            init.set_signal(Some(&controller.signal()));
            response(&req.url, &init).await
        }
    }

    async fn local_json(url: &str, body: Option<&Value>) -> Result<Value, Failed> {
        let init = RequestInit::new();
        if let Some(body) = body {
            init.set_method("POST");
            init.set_body(&body.to_string().as_str().into());
        }
        let res = response(url, &init).await?;
        if !res.ok() {
            return Err(Failed);
        }
        serde_json::from_str(&res.body).map_err(|_| Failed)
    }

    /// The models a local server at `base` has; none when it is down or refuses the extension.
    pub async fn local_models(base: &str) -> Vec<String> {
        let Ok(j) = local_json(&format!("{base}/api/tags"), None).await else {
            return vec![];
        };
        j.get("models")
            .and_then(Value::as_array)
            .map(|models| {
                models.iter().filter_map(|m| m.get("name").and_then(Value::as_str).map(String::from)).collect()
            })
            .unwrap_or_default()
    }

    /// Downloads a model into the server; both resume a pull cut short, so an interrupted one can be asked again.
    pub async fn local_pull(base: &str, model: &str) -> bool {
        match local_json(&format!("{base}/api/pull"), Some(&json!({ "model": model, "stream": false }))).await {
            Ok(j) => j.get("status").and_then(Value::as_str) == Some("success"),
            Err(Failed) => false,
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use browser::{BrowserFetch, local_models, local_pull};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PerProvider;

    fn base() -> Settings {
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

    fn body(r: &HttpRequest) -> Value {
        serde_json::from_str(&r.body).unwrap()
    }

    fn header<'a>(r: &'a HttpRequest, name: &str) -> Option<&'a str> {
        r.headers.iter().find(|(k, _)| *k == name).map(|(_, v)| v.as_str())
    }

    #[test]
    fn claude_url_exact_headers_and_body() {
        let r = build_request(ProviderId::Claude, &base(), "SYS", "USER");
        assert_eq!(r.url, "https://api.anthropic.com/v1/messages");
        let headers: Vec<(&str, &str)> = r.headers.iter().map(|(k, v)| (*k, v.as_str())).collect();
        assert_eq!(
            headers,
            [
                ("content-type", "application/json"),
                ("x-api-key", "sk-test"),
                ("anthropic-version", "2023-06-01"),
                ("anthropic-dangerous-direct-browser-access", "true"),
            ]
        );
        assert_eq!(
            body(&r),
            json!({ "model": base().models.claude, "max_tokens": 16000, "output_config": { "effort": "low" }, "system": "SYS", "messages": [{ "role": "user", "content": "USER" }] })
        );
    }

    #[test]
    fn claude_models_get_no_sampling_parameters() {
        // Every model the settings offer answers temperature, top_p or top_k with a 400.
        for model in crate::settings::models(ProviderId::Claude) {
            let mut s = base();
            s.models.claude = (*model).into();
            let b = body(&build_request(ProviderId::Claude, &s, "SYS", "USER"));
            assert_eq!(b["model"], *model);
            for p in ["temperature", "top_p", "top_k", "thinking"] {
                assert!(b.get(p).is_none(), "{model}: {p}");
            }
        }
    }

    #[test]
    fn deepseek_url_bearer_and_messages() {
        let r = build_request(ProviderId::Deepseek, &base(), "SYS", "USER");
        assert_eq!(r.url, "https://api.deepseek.com/chat/completions");
        assert_eq!(header(&r, "Authorization"), Some("Bearer sk-test"));
        assert_eq!(header(&r, "content-type"), Some("application/json"));
        let b = body(&r);
        assert_eq!(b["model"], base().models.deepseek);
        assert_eq!(b["temperature"], 0);
        assert_eq!(b["max_tokens"], 2048);
        // deepseek-flash and deepseek-v4-pro think by default, and the thinking counts against max_tokens.
        assert_eq!(b["thinking"], json!({ "type": "disabled" }));
        assert_eq!(b["messages"][0], json!({ "role": "system", "content": "SYS" }));
        assert_eq!(b["messages"][1], json!({ "role": "user", "content": "USER" }));
    }

    #[test]
    fn ollama_local_chat_api_no_key_thinking_off_reply_and_truncation() {
        let r = build_request(ProviderId::Ollama, &base(), "SYS", "USER");
        assert_eq!(r.url, "http://127.0.0.1:11434/api/chat");
        assert_eq!(r.headers, [("content-type", "application/json".to_string())]);
        assert_eq!(
            body(&r),
            json!({
                "model": "qwen3.6:27b",
                "stream": false,
                "think": false,
                "options": { "temperature": 0, "num_predict": 1024 },
                "messages": [{ "role": "system", "content": "SYS" }, { "role": "user", "content": "USER" }],
            })
        );
        assert_eq!(
            read_text(ProviderId::Ollama, &json!({ "message": { "content": "[]" }, "done_reason": "stop" })),
            "[]"
        );
        assert!(is_truncated(ProviderId::Ollama, &json!({ "done_reason": "length" })));
        assert!(!is_truncated(ProviderId::Ollama, &json!({ "done_reason": "stop" })));
    }

    #[cfg(feature = "test-build")]
    #[test]
    fn api_base_overrides_the_url_test_builds_only() {
        let mut s = base();
        s.api_base.insert(ProviderId::Claude, "http://127.0.0.1:9".into());
        s.api_base.insert(ProviderId::Deepseek, "http://127.0.0.1:8/".into());
        assert_eq!(build_request(ProviderId::Claude, &s, "a", "b").url, "http://127.0.0.1:9/v1/messages");
        assert_eq!(build_request(ProviderId::Deepseek, &s, "a", "b").url, "http://127.0.0.1:8/chat/completions");
        s.api_base.insert(ProviderId::Ollama, "http://127.0.0.1:7/".into());
        assert_eq!(local_base_for(ProviderId::Ollama, &s).as_deref(), Some("http://127.0.0.1:7"));
        assert_eq!(local_base_for(ProviderId::Ollaya, &s).as_deref(), Some("http://127.0.0.1:11435"));
    }

    #[cfg(not(feature = "test-build"))]
    #[test]
    fn api_base_is_ignored_outside_a_test_build() {
        let mut s = base();
        s.api_base.insert(ProviderId::Claude, "http://127.0.0.1:9".into());
        assert_eq!(build_request(ProviderId::Claude, &s, "a", "b").url, "https://api.anthropic.com/v1/messages");
        s.api_base.insert(ProviderId::Ollama, "http://127.0.0.1:7".into());
        assert_eq!(local_base_for(ProviderId::Ollama, &s).as_deref(), Some("http://127.0.0.1:11434"));
        assert_eq!(local_base_for(ProviderId::Claude, &s), None);
    }

    #[test]
    fn without_api_base_the_url_is_the_real_endpoint() {
        assert_eq!(build_request(ProviderId::Claude, &base(), "a", "b").url, "https://api.anthropic.com/v1/messages");
        assert_eq!(
            build_request(ProviderId::Deepseek, &base(), "a", "b").url,
            "https://api.deepseek.com/chat/completions"
        );
    }

    #[test]
    fn claude_concatenates_text_blocks() {
        let j = json!({ "content": [{ "type": "text", "text": "a" }, { "type": "tool_use" }, { "type": "text", "text": "b" }] });
        assert_eq!(read_text(ProviderId::Claude, &j), "ab");
    }

    #[test]
    fn deepseek_first_choice_content() {
        assert_eq!(read_text(ProviderId::Deepseek, &json!({ "choices": [{ "message": { "content": "oi" } }] })), "oi");
    }

    #[test]
    fn unexpected_shapes_give_empty_string() {
        assert_eq!(read_text(ProviderId::Claude, &json!({})), "");
        assert_eq!(read_text(ProviderId::Deepseek, &json!({})), "");
        assert_eq!(read_text(ProviderId::Claude, &Value::Null), "");
        assert_eq!(read_text(ProviderId::Deepseek, &json!("x")), "");
    }

    #[test]
    fn claude_stop_reason_max_tokens() {
        assert!(is_truncated(ProviderId::Claude, &json!({ "stop_reason": "max_tokens", "content": [] })));
        assert!(!is_truncated(ProviderId::Claude, &json!({ "stop_reason": "end_turn", "content": [] })));
    }

    #[test]
    fn deepseek_finish_reason_length() {
        assert!(is_truncated(
            ProviderId::Deepseek,
            &json!({ "choices": [{ "finish_reason": "length", "message": { "content": "[" } }] })
        ));
        assert!(!is_truncated(
            ProviderId::Deepseek,
            &json!({ "choices": [{ "finish_reason": "stop", "message": { "content": "[]" } }] })
        ));
    }

    #[test]
    fn unexpected_shapes_are_not_truncated() {
        assert!(!is_truncated(ProviderId::Claude, &Value::Null));
        assert!(!is_truncated(ProviderId::Deepseek, &json!({})));
    }

    #[test]
    fn maps_statuses() {
        assert_eq!(classify_error(Some(401), None), CheckError::Auth);
        assert_eq!(classify_error(Some(403), None), CheckError::Auth);
        assert_eq!(classify_error(Some(429), None), CheckError::RateLimit);
        assert_eq!(classify_error(Some(400), None), CheckError::Request);
        assert_eq!(classify_error(Some(404), None), CheckError::Request);
        assert_eq!(classify_error(Some(500), None), CheckError::Busy);
        assert_eq!(classify_error(Some(529), None), CheckError::Busy);
        assert_eq!(classify_error(Some(302), None), CheckError::Provider);
        assert_eq!(classify_error(None, None), CheckError::Network);
    }

    #[test]
    fn ollama_down_or_refusing_the_extension_origin_is_local() {
        assert_eq!(classify_error(None, Some(ProviderId::Ollama)), CheckError::Local);
        assert_eq!(classify_error(Some(403), Some(ProviderId::Ollama)), CheckError::Local);
        assert_eq!(classify_error(Some(500), Some(ProviderId::Ollama)), CheckError::Busy);
        assert_eq!(classify_error(Some(403), Some(ProviderId::Claude)), CheckError::Auth);
    }

    #[test]
    fn ollaya_down_or_refusing_the_extension_origin_is_local_ollaya() {
        assert_eq!(classify_error(None, Some(ProviderId::Ollaya)), CheckError::LocalOllaya);
        assert_eq!(classify_error(Some(403), Some(ProviderId::Ollaya)), CheckError::LocalOllaya);
        assert_eq!(classify_error(Some(404), Some(ProviderId::Ollaya)), CheckError::Request);
    }

    #[test]
    fn abort_runs_its_listeners_once() {
        let a = Abort::default();
        let hits = std::rc::Rc::new(std::cell::Cell::new(0));
        let h = hits.clone();
        a.on_abort(move || h.set(h.get() + 1));
        assert!(!a.aborted());
        a.abort();
        a.abort();
        assert!(a.aborted());
        assert_eq!(hits.get(), 1);
        let h = hits.clone();
        a.on_abort(move || h.set(h.get() + 1));
        assert_eq!(hits.get(), 2);
    }
}
