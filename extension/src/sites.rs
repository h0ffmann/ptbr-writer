//! The site list and the key field, shared by the options page and the popup: the pure parts
//! here, the flows (permissions, storage, a message to the background) on wasm only.

use crate::types::Level;

pub const DEFAULT_SITE_HOSTS: [&str; 3] = ["web.whatsapp.com", "mail.google.com", "github.com"];
pub const LEVELS: [Level; 4] = Level::ALL;
pub const NEW_SITE_LEVEL: Level = Level::Descritivo;
const MASK: &str = "••••";

pub fn is_default_host(host: &str) -> bool {
    DEFAULT_SITE_HOSTS.contains(&host)
}

/// The key as the field shows it: the mask and its last 4 characters; empty stays empty.
pub fn mask_key(key: &str) -> String {
    if key.is_empty() {
        return String::new();
    }
    let tail = key.char_indices().nth_back(3).map_or(key, |(i, _)| &key[i..]);
    format!("{MASK}{tail}")
}

/// What to store after a key field is saved: empty or still-masked input keeps the stored key.
pub fn resolve_key(stored: &str, typed: &str) -> String {
    let value = typed.trim();
    if value.is_empty() || value.starts_with(MASK) { stored.to_string() } else { value.to_string() }
}

#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use crate::chrome;
    use crate::settings::{known_model, load_settings, save_settings};
    use crate::types::{Message, ProviderId, Settings, SiteConfig};
    use wasm_bindgen::JsValue;

    fn origin(host: &str) -> String {
        format!("https://{host}/*")
    }

    async fn notify(msg: Message) -> Result<(), JsValue> {
        chrome::send_message::<_, serde_json::Value>(&msg).await.map(|_| ())
    }

    /// Enables `host`: its permission when it is not a default site, then the setting. `false` when
    /// the permission was refused.
    pub async fn enable_site_flow(host: &str) -> Result<bool, JsValue> {
        let is_default = is_default_host(host);
        if !is_default && !chrome::permissions_request(&origin(host)).await? {
            return Ok(false);
        }
        let mut s = load_settings().await?;
        let level = s.sites.get(host).map_or(NEW_SITE_LEVEL, |c| c.level);
        s.sites.insert(host.into(), SiteConfig { enabled: true, level });
        save_settings(&s).await?;
        if !is_default {
            notify(Message::EnableSite { host: host.into() }).await?;
        }
        Ok(true)
    }

    pub async fn disable_site_flow(host: &str) -> Result<(), JsValue> {
        let is_default = is_default_host(host);
        let mut s = load_settings().await?;
        let level = s.sites.get(host).map_or(NEW_SITE_LEVEL, |c| c.level);
        s.sites.insert(host.into(), SiteConfig { enabled: false, level });
        save_settings(&s).await?;
        if is_default {
            return Ok(());
        }
        // A permission the manifest requires cannot be removed; the site is off all the same.
        let _ = chrome::permissions_remove(&origin(host)).await;
        notify(Message::DisableSite { host: host.into() }).await
    }

    pub async fn remove_site_flow(host: &str) -> Result<(), JsValue> {
        if is_default_host(host) {
            return Ok(());
        }
        let mut s = load_settings().await?;
        s.sites.remove(host);
        save_settings(&s).await?;
        let _ = chrome::permissions_remove(&origin(host)).await;
        notify(Message::DisableSite { host: host.into() }).await
    }

    pub async fn set_level(host: &str, level: Level) -> Result<(), JsValue> {
        let mut s = load_settings().await?;
        let Some(site) = s.sites.get_mut(host) else {
            return Ok(());
        };
        site.level = level;
        save_settings(&s).await
    }

    pub async fn set_ollaya_style(ollaya_style: bool) -> Result<(), JsValue> {
        let mut s = load_settings().await?;
        s.ollaya_style = ollaya_style;
        save_settings(&s).await
    }

    pub async fn set_enabled(enabled: bool) -> Result<(), JsValue> {
        let mut s = load_settings().await?;
        s.enabled = enabled;
        save_settings(&s).await
    }

    /// Saves tone, rewrite, wait or the never-fix list, as `change` sets them on the settings stored
    /// now; every other setting stays.
    pub async fn save_text_settings(change: impl FnOnce(&mut Settings)) -> Result<(), JsValue> {
        let mut s = load_settings().await?;
        change(&mut s);
        save_settings(&s).await
    }

    /// The provider fields as saved by Salvar.
    pub struct ProviderFields<'a> {
        pub provider: ProviderId,
        pub key: &'a str,
        pub model: &'a str,
        pub enabled: bool,
    }

    /// Saves the chosen provider with its key and model; the other providers' keys and models stay.
    pub async fn save_provider_fields(f: ProviderFields<'_>) -> Result<(), JsValue> {
        let mut s = load_settings().await?;
        let p = f.provider;
        s.enabled = f.enabled;
        s.provider = p;
        s.keys[p] = resolve_key(&s.keys[p], f.key);
        if known_model(p, Some(f.model)) {
            s.models[p] = f.model.into();
        }
        save_settings(&s).await
    }
}

#[cfg(target_arch = "wasm32")]
pub use browser::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_and_keeps_on_empty_or_masked_input() {
        assert_eq!(mask_key("sk-test-1234"), "••••1234");
        assert_eq!(mask_key("ab"), "••••ab");
        assert_eq!(mask_key(""), "");
        assert_eq!(resolve_key("sk-old", ""), "sk-old");
        assert_eq!(resolve_key("sk-old", "••••1234"), "sk-old");
        assert_eq!(resolve_key("sk-old", "sk-new"), "sk-new");
    }
}
