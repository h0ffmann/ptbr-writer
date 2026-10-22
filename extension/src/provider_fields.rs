//! Provider, model and key, shared by the popup and the options page. The model list and the key
//! follow the chosen provider; the other provider's key stays stored. The pure parts (what the
//! list offers, what a model's label says) here; the fields themselves on wasm only.

use crate::settings::models;
use crate::types::ProviderId;

pub const PROVIDERS: [(ProviderId, &str); 4] = [
    (ProviderId::Claude, "Claude"),
    (ProviderId::Deepseek, "DeepSeek"),
    (ProviderId::Ollama, "Ollama (local)"),
    (ProviderId::Ollaya, "Ollaya (local, só aponta)"),
];

/// The models the list offers: a local server's suggested models (the first one suggested) and the
/// ones it has installed; the fixed list for the others.
pub fn model_list(p: ProviderId, installed: &[String]) -> Vec<String> {
    let mut list: Vec<String> = models(p).iter().map(|m| (*m).to_string()).collect();
    if p.is_local() {
        for m in installed {
            if !list.contains(m) {
                list.push(m.clone());
            }
        }
    }
    list
}

/// A model as the list shows it; on a local server, whether it is the suggested one and whether
/// it is still to be downloaded.
pub fn model_label(p: ProviderId, m: &str, installed: &[String]) -> String {
    if !p.is_local() {
        return m.to_string();
    }
    let suggested = if m == models(p)[0] { " (sugerido)" } else { "" };
    let download = if installed.iter().any(|i| i == m) { "" } else { " (baixar)" };
    format!("{m}{suggested}{download}")
}

/// Baixar shows for a local model not installed yet.
pub fn pull_hidden(p: ProviderId, model: &str, installed: &[String]) -> bool {
    !p.is_local() || installed.iter().any(|i| i == model)
}

/// Ollama and Ollaya run on this machine and take no key.
pub fn key_placeholder(p: ProviderId) -> String {
    if p.is_local() { format!("sem chave: just ext-{p}") } else { String::new() }
}

pub fn pull_status(p: ProviderId, ok: bool) -> String {
    if ok {
        return "pronto".to_string();
    }
    let name = if p == ProviderId::Ollama { "Ollama" } else { "Ollaya" };
    format!("falhou: o {name} está no ar (just ext-{p})?")
}

#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use crate::chrome::permissions_request;
    use crate::dom::{Build, create, el, on, option, spawn};
    use crate::providers::{local_models, local_origin, local_pull};
    use crate::sites::{mask_key, set_ollaya_style};
    use crate::types::Settings;
    use std::cell::RefCell;
    use std::rc::Rc;
    use web_sys::{Element, HtmlButtonElement, HtmlElement, HtmlInputElement, HtmlLabelElement, HtmlSelectElement};

    /// The three fields, over the settings they edit in place, so a provider switched back shows
    /// what was just chosen.
    #[derive(Clone)]
    pub struct ProviderFields {
        pub provider: HtmlSelectElement,
        pub model: HtmlSelectElement,
        pub key: HtmlInputElement,
        pub settings: Rc<RefCell<Settings>>,
    }

    impl ProviderFields {
        /// The provider the select shows.
        pub fn chosen(&self) -> ProviderId {
            ProviderId::parse(&self.provider.value()).unwrap_or(self.settings.borrow().provider)
        }
    }

    struct Mount {
        fields: ProviderFields,
        pull: HtmlButtonElement,
        status: HtmlElement,
        style_row: HtmlLabelElement,
        installed: RefCell<Vec<String>>,
        on_change: Box<dyn Fn(&ProviderFields)>,
    }

    impl Mount {
        fn fill(&self, p: ProviderId) {
            let installed = self.installed.borrow();
            let model = &self.fields.model;
            model.replace_children_with_node_0();
            for m in model_list(p, &installed) {
                model.append_child(&option(&m, &model_label(p, &m, &installed))).expect("appendChild");
            }
            model.set_value(&self.fields.settings.borrow().models[p]);
            self.pull.set_hidden(pull_hidden(p, &model.value(), &installed));
        }

        async fn refresh(&self) {
            let p = self.fields.chosen();
            if !p.is_local() {
                return;
            }
            let got = local_models(p).await;
            // A switch while the list was loading: this list belongs to the other server.
            if self.fields.chosen() != p {
                return;
            }
            *self.installed.borrow_mut() = got;
            self.fill(p);
        }

        fn show(self: &Rc<Self>, p: ProviderId) {
            self.style_row.set_hidden(p != ProviderId::Ollama);
            self.fields.key.set_value(&mask_key(&self.fields.settings.borrow().keys[p]));
            // The other server's models are not this one's.
            self.installed.borrow_mut().clear();
            self.fill(p);
            if p.is_local() {
                let m = self.clone();
                wasm_bindgen_futures::spawn_local(async move { m.refresh().await });
            }
            self.fields.key.set_disabled(p.is_local());
            self.fields.key.set_placeholder(&key_placeholder(p));
        }
    }

    fn labeled(text: &str, input: &impl AsRef<web_sys::Node>) -> HtmlLabelElement {
        create::<HtmlLabelElement>("label").text(&format!("{text} ")).child(input)
    }

    /// Mounts the fields into `container`; `on_change` runs when the provider or the model changes.
    pub fn mount(
        container: &Element,
        settings: Rc<RefCell<Settings>>,
        on_change: impl Fn(&ProviderFields) + 'static,
    ) -> ProviderFields {
        let provider: HtmlSelectElement = create::<HtmlSelectElement>("select").id("provider");
        for (id, name) in PROVIDERS {
            provider.append_child(&option(id.as_str(), name)).expect("appendChild");
        }
        let model: HtmlSelectElement = create::<HtmlSelectElement>("select").id("model");
        let key: HtmlInputElement = create::<HtmlInputElement>("input").id("key");
        key.set_type("password");
        key.set_autocomplete("off");
        let fields = ProviderFields { provider, model, key, settings };

        // A local server's list (Ollama, Ollaya): the suggested models and the ones already
        // installed; one not installed is downloaded with Baixar.
        let pull: HtmlButtonElement = create::<HtmlButtonElement>("button").id("pull-model");
        pull.set_type("button");
        pull.set_text_content(Some("Baixar"));
        let status: HtmlElement = create::<HtmlElement>("span").id("pull-status");
        status.set_attribute("role", "status").expect("setAttribute");

        // Ollaya + Ollama: a feature flag beside Ollama. On, Ollaya picks the style and Ollama writes
        // in it; Ollaya down, Ollama writes alone. Saved on the click, like the on/off switch.
        let style: HtmlInputElement = create::<HtmlInputElement>("input").id("ollaya-style");
        style.set_type("checkbox");
        style.set_checked(fields.settings.borrow().ollaya_style);
        let style_row = labeled("Ollaya escolhe o estilo (conciso, socrático, amigável)", &style);

        let m = Rc::new(Mount {
            fields,
            pull,
            status,
            style_row,
            installed: RefCell::new(vec![]),
            on_change: Box::new(on_change),
        });

        on(&style, "change", {
            let (m, style) = (m.clone(), style.clone());
            move || {
                let (m, style) = (m.clone(), style.clone());
                spawn(async move {
                    // The permission for Ollaya, asked in this click as for the provider itself.
                    let origin = local_origin(ProviderId::Ollaya).expect("a local provider");
                    if style.checked() && !permissions_request(&origin).await? {
                        style.set_checked(false);
                    }
                    m.fields.settings.borrow_mut().ollaya_style = style.checked();
                    set_ollaya_style(style.checked()).await
                })
            }
        });

        let current = m.fields.settings.borrow().provider;
        m.fields.provider.set_value(current.as_str());
        m.show(current);

        on(&m.fields.provider, "change", {
            let m = m.clone();
            move || {
                let m = m.clone();
                spawn(async move {
                    let p = m.fields.chosen();
                    // Asked in the click that picks a local server: the browser shows its prompt
                    // only during a user gesture.
                    if let Some(origin) = local_origin(p)
                        && !permissions_request(&origin).await?
                    {
                        m.fields.provider.set_value(m.fields.settings.borrow().provider.as_str());
                        return Ok(());
                    }
                    m.fields.settings.borrow_mut().provider = p;
                    m.show(p);
                    (m.on_change)(&m.fields);
                    Ok(())
                })
            }
        });
        on(&m.fields.model, "change", {
            let m = m.clone();
            move || {
                let p = m.fields.chosen();
                let model = m.fields.model.value();
                m.fields.settings.borrow_mut().models[p] = model.clone();
                m.pull.set_hidden(pull_hidden(p, &model, &m.installed.borrow()));
                (m.on_change)(&m.fields);
            }
        });
        on(&m.pull, "click", {
            let m = m.clone();
            move || {
                let m = m.clone();
                spawn(async move {
                    m.pull.set_disabled(true);
                    let model = m.fields.model.value();
                    m.status.set_text_content(Some(&format!("baixando {model}…")));
                    let p = m.fields.chosen();
                    let ok = local_pull(p, &model).await;
                    if ok {
                        m.refresh().await;
                    }
                    m.status.set_text_content(Some(&pull_status(p, ok)));
                    m.pull.set_disabled(false);
                    Ok(())
                })
            }
        });

        let pull_row = el("p").child(&m.pull).text(" ").child(&m.status);
        container
            .child(&labeled("Provedor", &m.fields.provider))
            .child(&labeled("Modelo", &m.fields.model))
            .child(&pull_row)
            .child(&m.style_row)
            .child(&labeled("Chave", &m.fields.key));
        m.fields.clone()
    }
}

#[cfg(target_arch = "wasm32")]
pub use browser::{ProviderFields, mount};

#[cfg(test)]
mod tests {
    use super::*;

    fn installed(names: &[&str]) -> Vec<String> {
        names.iter().map(|n| n.to_string()).collect()
    }

    #[test]
    fn one_model_list_for_the_chosen_provider() {
        assert_eq!(model_list(ProviderId::Claude, &installed(&["llama3.2:1b"])), models(ProviderId::Claude));
        assert_eq!(model_list(ProviderId::Deepseek, &[]), ["deepseek-flash", "deepseek-v4-pro"]);
        assert_eq!(model_label(ProviderId::Claude, "claude-haiku-5-5", &[]), "claude-haiku-5-5");
        assert!(pull_hidden(ProviderId::Claude, "claude-haiku-5-5", &[]));
        assert_eq!(key_placeholder(ProviderId::Claude), "");
    }

    #[test]
    fn a_local_server_lists_the_suggested_and_the_installed_models() {
        let have = installed(&["llama3.2:1b"]);
        assert_eq!(
            model_list(ProviderId::Ollama, &have),
            ["qwen3.6:27b", "qwen3:14b", "qwen3:8b", "qwen3:4b", "llama3.2:1b"]
        );
        assert_eq!(model_label(ProviderId::Ollama, "qwen3.6:27b", &have), "qwen3.6:27b (sugerido) (baixar)");
        assert_eq!(model_label(ProviderId::Ollama, "llama3.2:1b", &have), "llama3.2:1b");
        assert!(!pull_hidden(ProviderId::Ollama, "qwen3.6:27b", &have));
        assert_eq!(key_placeholder(ProviderId::Ollama), "sem chave: just ext-ollama");
        assert_eq!(
            model_list(ProviderId::Ollaya, &have),
            ["decima", "laya:multilingual", "decima:small", "pplx-decider", "winnow:e4b", "llama3.2:1b"]
        );
    }

    #[test]
    fn baixar_lists_the_pulled_model_as_installed() {
        let have = installed(&["llama3.2:1b", "qwen3.6:27b"]);
        assert_eq!(model_label(ProviderId::Ollama, "qwen3.6:27b", &have), "qwen3.6:27b (sugerido)");
        assert!(pull_hidden(ProviderId::Ollama, "qwen3.6:27b", &have));
        assert_eq!(model_list(ProviderId::Ollama, &have).iter().filter(|m| *m == "qwen3.6:27b").count(), 1);
        assert_eq!(pull_status(ProviderId::Ollama, true), "pronto");
        assert_eq!(pull_status(ProviderId::Ollama, false), "falhou: o Ollama está no ar (just ext-ollama)?");
        assert_eq!(pull_status(ProviderId::Ollaya, false), "falhou: o Ollaya está no ar (just ext-ollaya)?");
    }
}
