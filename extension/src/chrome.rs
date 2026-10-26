//! The `chrome.*` APIs the extension uses, as typed imports, and the serde boundary over them: a
//! Rust value goes out as a plain JS object (`to_js`) and a JS value comes in through serde
//! (`from_js`), so the shapes stay the TypeScript version's.

use js_sys::{Function, Promise};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen]
extern "C" {
    /// A `chrome.*.onSomething` event.
    pub type ChromeEvent;
    #[wasm_bindgen(method, js_name = addListener)]
    pub fn add_listener(this: &ChromeEvent, listener: &Function);
    #[wasm_bindgen(method, js_name = removeListener)]
    pub fn remove_listener(this: &ChromeEvent, listener: &Function);

    #[wasm_bindgen(js_namespace = ["chrome", "storage", "local"], js_name = get)]
    fn storage_local_get(key: &str) -> Promise;
    #[wasm_bindgen(js_namespace = ["chrome", "storage", "local"], js_name = set)]
    fn storage_local_set(items: &JsValue) -> Promise;
    /// `chrome.storage.onChanged`; absent where the API is stubbed.
    #[wasm_bindgen(thread_local_v2, js_namespace = ["chrome", "storage"], js_name = onChanged)]
    pub static STORAGE_ON_CHANGED: Option<ChromeEvent>;

    // `catch`: after the extension reloads, a script left in an open tab throws here, synchronously.
    #[wasm_bindgen(catch, js_namespace = ["chrome", "runtime"], js_name = sendMessage)]
    fn runtime_send_message(message: &JsValue) -> Result<Promise, JsValue>;
    #[wasm_bindgen(js_namespace = ["chrome", "runtime"], js_name = getURL)]
    pub fn get_url(path: &str) -> String;
    #[wasm_bindgen(js_namespace = ["chrome", "runtime"], js_name = openOptionsPage)]
    pub fn open_options_page() -> Promise;

    #[wasm_bindgen(js_namespace = ["chrome", "scripting"], js_name = registerContentScripts)]
    fn scripting_register(scripts: &JsValue) -> Promise;
    #[wasm_bindgen(js_namespace = ["chrome", "scripting"], js_name = unregisterContentScripts)]
    fn scripting_unregister(filter: &JsValue) -> Promise;
    #[wasm_bindgen(js_namespace = ["chrome", "scripting"], js_name = getRegisteredContentScripts)]
    fn scripting_get_registered(filter: &JsValue) -> Promise;

    #[wasm_bindgen(js_namespace = ["chrome", "permissions"], js_name = request)]
    fn permissions_request_raw(permissions: &JsValue) -> Promise;
    #[wasm_bindgen(js_namespace = ["chrome", "permissions"], js_name = remove)]
    fn permissions_remove_raw(permissions: &JsValue) -> Promise;

    #[wasm_bindgen(js_namespace = ["chrome", "tabs"], js_name = query)]
    fn tabs_query_raw(query: &JsValue) -> Promise;
}

/// A Rust value as a plain JS object: maps become objects, not `Map`s.
pub fn to_js<T: Serialize>(v: &T) -> Result<JsValue, JsValue> {
    v.serialize(&serde_wasm_bindgen::Serializer::json_compatible()).map_err(JsValue::from)
}

pub fn from_js<T: DeserializeOwned>(v: JsValue) -> Result<T, JsValue> {
    serde_wasm_bindgen::from_value(v).map_err(JsValue::from)
}

async fn await_js(p: Promise) -> Result<JsValue, JsValue> {
    JsFuture::from(p).await
}

/// `chrome.storage.local.get(key)[key]`, as JSON; `None` when nothing is stored under it.
pub async fn storage_get(key: &str) -> Result<Option<Value>, JsValue> {
    let got: serde_json::Map<String, Value> = from_js(await_js(storage_local_get(key)).await?)?;
    Ok(got.get(key).cloned())
}

pub async fn storage_set<T: Serialize>(key: &str, value: &T) -> Result<(), JsValue> {
    let mut items = serde_json::Map::new();
    items.insert(key.into(), serde_json::to_value(value).map_err(|e| JsValue::from_str(&e.to_string()))?);
    await_js(storage_local_set(&to_js(&items)?)).await.map(|_| ())
}

/// `chrome.runtime.sendMessage(msg)`, the reply as it came: `Err` when nothing answered.
pub async fn send_message_raw<M: Serialize>(msg: &M) -> Result<JsValue, JsValue> {
    await_js(runtime_send_message(&to_js(msg)?)?).await
}

/// `chrome.runtime.sendMessage(msg)`, the reply read as `R`.
pub async fn send_message<M: Serialize, R: DeserializeOwned>(msg: &M) -> Result<R, JsValue> {
    from_js(send_message_raw(msg).await?)
}

/// The ids of the registered content scripts, all or those among `ids`.
pub async fn get_registered_content_scripts(ids: Option<&[String]>) -> Result<Vec<String>, JsValue> {
    #[derive(serde::Deserialize)]
    struct Registered {
        id: String,
    }
    let filter = match ids {
        Some(ids) => to_js(&serde_json::json!({ "ids": ids }))?,
        None => JsValue::UNDEFINED,
    };
    let scripts: Vec<Registered> = from_js(await_js(scripting_get_registered(&filter)).await?)?;
    Ok(scripts.into_iter().map(|s| s.id).collect())
}

pub async fn register_content_scripts<T: Serialize>(scripts: &[T]) -> Result<(), JsValue> {
    await_js(scripting_register(&to_js(&scripts)?)).await.map(|_| ())
}

pub async fn unregister_content_scripts(ids: &[String]) -> Result<(), JsValue> {
    await_js(scripting_unregister(&to_js(&serde_json::json!({ "ids": ids }))?)).await.map(|_| ())
}

fn origins(origin: &str) -> Result<JsValue, JsValue> {
    to_js(&serde_json::json!({ "origins": [origin] }))
}

/// Asks the user for a host permission (`https://host/*`); only during a user gesture.
pub async fn permissions_request(origin: &str) -> Result<bool, JsValue> {
    Ok(await_js(permissions_request_raw(&origins(origin)?)).await?.is_truthy())
}

pub async fn permissions_remove(origin: &str) -> Result<bool, JsValue> {
    Ok(await_js(permissions_remove_raw(&origins(origin)?)).await?.is_truthy())
}

/// What the popup reads of a tab.
#[derive(Clone, Debug, Default, serde::Deserialize)]
pub struct Tab {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub url: Option<String>,
}

/// The active tab of the current window, when there is one.
pub async fn active_tab() -> Result<Option<Tab>, JsValue> {
    let tabs: Vec<Tab> = from_js(
        await_js(tabs_query_raw(&to_js(&serde_json::json!({ "active": true, "currentWindow": true }))?)).await?,
    )?;
    Ok(tabs.into_iter().next())
}
