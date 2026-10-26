//! The popup: the main switch, the current site (level, on/off), then provider, model and key.

use crate::background::host_of_url;
use crate::hosts::normalize_host;
use crate::types::ProviderId;

/// The host the extension can run on: an https page whose host the options page would accept
/// (no localhost, no IP address).
pub fn host_of(url: Option<&str>) -> Option<String> {
    let url = url?;
    let (scheme, _) = url.split_once("://")?;
    if !scheme.eq_ignore_ascii_case("https") {
        return None;
    }
    normalize_host(host_of_url(url)?)
}

pub fn provider_name(p: ProviderId) -> &'static str {
    match p {
        ProviderId::Claude => "Claude",
        ProviderId::Deepseek => "DeepSeek",
        ProviderId::Ollama => "Ollama",
        ProviderId::Ollaya => "Ollaya",
    }
}

#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use crate::chrome::{active_tab, open_options_page};
    use crate::dom::{Build, button, create, el, line, on, option, spawn};
    use crate::provider_fields::mount;
    use crate::settings::load_settings;
    use crate::sites::{
        LEVELS, ProviderFields as Saved, disable_site_flow, enable_site_flow, save_provider_fields, set_enabled,
        set_level,
    };
    use crate::text_fields::mount_text_fields;
    use crate::types::Level;
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen::JsValue;
    use web_sys::{HtmlElement, HtmlInputElement, HtmlSelectElement};

    fn section(root: &HtmlElement) -> HtmlElement {
        let div: HtmlElement = create("section");
        root.child(&div);
        div
    }

    /// Provider, model, level and the switch save at once; the key only on Salvar, so a half-pasted
    /// key is never stored. The full page holds the list of every site.
    pub async fn render(root: HtmlElement) -> Result<(), JsValue> {
        root.replace_children_with_node_0();
        let s = Rc::new(RefCell::new(load_settings().await?));

        let head = section(&root);
        let enabled: HtmlInputElement = create::<HtmlInputElement>("input").id("enabled");
        enabled.set_type("checkbox");
        enabled.set_checked(s.borrow().enabled);
        on(&enabled, "change", {
            let enabled = enabled.clone();
            move || {
                let on = enabled.checked();
                spawn(async move { set_enabled(on).await })
            }
        });
        head.append_child(&el("label").child(&enabled).text(" ptbr-writer ligado"))?;

        let site = section(&root);
        let url = active_tab().await?.and_then(|t| t.url);
        match (url.as_deref(), host_of(url.as_deref())) {
            (None, _) => line(&site, "Abra um site https para ativar a extensão."),
            (Some(_), None) => line(&site, "Site não suportado."),
            (Some(_), Some(host)) => {
                let cfg = s.borrow().sites.get(&host).cloned();
                let is_on = cfg.as_ref().is_some_and(|c| c.enabled);
                line(&site, &host);
                if let Some(cfg) = cfg {
                    let level: HtmlSelectElement = create::<HtmlSelectElement>("select").id("site-level");
                    for l in LEVELS {
                        level.append_child(&option(l.as_str(), l.as_str()))?;
                    }
                    level.set_value(cfg.level.as_str());
                    on(&level, "change", {
                        let (host, level) = (host.clone(), level.clone());
                        move || {
                            if let Some(l) = Level::parse(&level.value()) {
                                let host = host.clone();
                                spawn(async move { set_level(&host, l).await });
                            }
                        }
                    });
                    site.append_child(&el("label").text("Nível ").child(&level))?;
                }
                let text = if is_on { "Desativar neste site" } else { "Ativar neste site" };
                let root = root.clone();
                site.append_child(
                    &button(text, move || {
                        let (host, root) = (host.clone(), root.clone());
                        spawn(async move {
                            if is_on {
                                disable_site_flow(&host).await?;
                            } else {
                                enable_site_flow(&host).await?;
                            }
                            render(root).await
                        })
                    })
                    .id("toggle-site"),
                )?;
            }
        }

        mount_text_fields(&section(&root), &s.borrow());

        let provider = section(&root);
        {
            let s = s.borrow();
            if !s.provider.is_local() && s.keys[s.provider].is_empty() {
                line(&provider, &format!("Sem chave do {}.", provider_name(s.provider)));
            }
        }
        let fields = mount(&provider, s.clone(), {
            let enabled = enabled.clone();
            move |f| {
                let (p, model, enabled) = (f.chosen(), f.model.value(), enabled.checked());
                spawn(async move { save_provider_fields(Saved { provider: p, model: &model, key: "", enabled }).await })
            }
        });
        provider.append_child(
            &button("Salvar", {
                let root = root.clone();
                move || {
                    let (fields, enabled, root) = (fields.clone(), enabled.clone(), root.clone());
                    spawn(async move {
                        save_provider_fields(Saved {
                            provider: fields.chosen(),
                            model: &fields.model.value(),
                            key: &fields.key.value(),
                            enabled: enabled.checked(),
                        })
                        .await?;
                        render(root).await
                    })
                }
            })
            .id("save-key"),
        )?;

        root.append_child(&button("Todas as configurações", || drop(open_options_page())).id("open-options"))?;
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub use browser::render;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_https_page_with_an_acceptable_host() {
        assert_eq!(host_of(Some("https://example.com/x")).as_deref(), Some("example.com"));
        assert_eq!(host_of(Some("https://Sub.Example.com:8443/a?b#c")).as_deref(), Some("sub.example.com"));
    }

    #[test]
    fn http_localhost_ip_other_schemes_and_no_url_are_not_supported() {
        for url in [
            "http://example.com/",
            "https://localhost/",
            "https://192.168.0.1/",
            "http://127.0.0.1:8080/",
            "https://[::1]/",
            "chrome://extensions",
            "about:blank",
            "",
        ] {
            assert_eq!(host_of(Some(url)), None, "{url}");
        }
        assert_eq!(host_of(None), None);
    }

    #[test]
    fn names_the_provider_without_a_key() {
        assert_eq!(format!("Sem chave do {}.", provider_name(ProviderId::Claude)), "Sem chave do Claude.");
        assert_eq!(provider_name(ProviderId::Deepseek), "DeepSeek");
    }
}
