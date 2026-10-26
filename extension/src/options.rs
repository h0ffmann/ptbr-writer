//! The options page: the main switch, provider, model and key, tone, rewrite and wait, the
//! never-fix list, and the list of every site.

use crate::dom::{Build, button, create, el, on, option, query, spawn};
use crate::hosts::normalize_host;
use crate::never_fix::{NEVER_FIX_MAX_WORDS, add_never_fix, word_count};
use crate::provider_fields::mount;
use crate::settings::load_settings;
use crate::sites::{
    LEVELS, ProviderFields as Saved, disable_site_flow, enable_site_flow, is_default_host, remove_site_flow,
    save_provider_fields, save_text_settings, set_level,
};
use crate::text_fields::mount_text_fields;
use crate::types::Level;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    HtmlButtonElement, HtmlElement, HtmlInputElement, HtmlSelectElement, HtmlTableRowElement, HtmlTableSectionElement,
};

// The page's fixed markup, as the TypeScript's innerHTML.
const PAGE: &str = r#"
    <h1>ptbr-writer</h1>
    <p><label><input id="enabled" type="checkbox"> Ativado</label></p>
    <div id="provider-fields"></div>
    <p><button id="save" type="button">Salvar</button> <span id="status" role="status"></span></p>
    <div id="text-fields"></div>
    <h2>Nunca corrigir</h2>
    <ul id="never-fix"></ul>
    <p><input id="new-never-fix" type="text" placeholder="até 3 palavras"> <button id="add-never-fix" type="button">Adicionar</button>
    <span id="never-fix-error" role="alert"></span></p>
    <h2>Sites</h2>
    <table id="sites"><thead><tr><th>Site</th><th>Nível</th><th>Ativo</th><th></th></tr></thead><tbody></tbody></table>
    <p><input id="new-host" type="text" placeholder="exemplo.com"> <button id="add-site" type="button">Adicionar site</button>
    <span id="site-error" role="alert"></span></p>"#;

pub async fn render(root: HtmlElement) -> Result<(), JsValue> {
    let s = Rc::new(RefCell::new(load_settings().await?));
    root.set_inner_html(PAGE);
    query::<HtmlInputElement>(&root, "#enabled").set_checked(s.borrow().enabled);
    let fields = mount(&query(&root, "#provider-fields"), s.clone(), |_| {});
    mount_text_fields(&query(&root, "#text-fields"), &s.borrow());
    let list: HtmlElement = query(&root, "#never-fix");
    for entry in &s.borrow().never_fix {
        let li = el("li").text(&format!("{entry} "));
        let (entry, root) = (entry.clone(), root.clone());
        li.append_child(&button("Remover", move || {
            let (entry, root) = (entry.clone(), root.clone());
            spawn(async move {
                // Reloaded in save_text_settings: another tab may have changed the list since.
                save_text_settings(|s| s.never_fix.retain(|e| *e != entry)).await?;
                render(root).await
            })
        }))?;
        list.append_child(&li)?;
    }
    on(&query::<HtmlButtonElement>(&root, "#add-never-fix"), "click", {
        let root = root.clone();
        move || {
            let root = root.clone();
            spawn(async move {
                let typed = query::<HtmlInputElement>(&root, "#new-never-fix").value();
                let typed = typed.trim();
                if typed.is_empty() || word_count(typed) > NEVER_FIX_MAX_WORDS {
                    query::<HtmlElement>(&root, "#never-fix-error").set_text_content(Some("Use de 1 a 3 palavras."));
                    return Ok(());
                }
                save_text_settings(|s| s.never_fix = add_never_fix(&s.never_fix, typed)).await?;
                render(root).await
            })
        }
    });
    let body: HtmlTableSectionElement = query(&root, "#sites tbody");
    for (host, cfg) in &s.borrow().sites {
        let row: HtmlTableRowElement = body.insert_row()?.unchecked_into();
        row.set_attribute("data-host", host)?;
        row.insert_cell()?.set_text_content(Some(host));
        let select: HtmlSelectElement = create("select");
        for l in LEVELS {
            select.append_child(&option(l.as_str(), l.as_str()))?;
        }
        select.set_value(cfg.level.as_str());
        on(&select, "change", {
            let (host, select) = (host.clone(), select.clone());
            move || {
                if let Some(level) = Level::parse(&select.value()) {
                    let host = host.clone();
                    spawn(async move { set_level(&host, level).await });
                }
            }
        });
        row.insert_cell()?.append_child(&select)?;
        let toggle: HtmlInputElement = create("input");
        toggle.set_type("checkbox");
        toggle.set_checked(cfg.enabled);
        on(&toggle, "change", {
            let (host, toggle, root) = (host.clone(), toggle.clone(), root.clone());
            move || {
                let (host, on, root) = (host.clone(), toggle.checked(), root.clone());
                spawn(async move {
                    if on {
                        enable_site_flow(&host).await?;
                    } else {
                        disable_site_flow(&host).await?;
                    }
                    render(root).await
                })
            }
        });
        row.insert_cell()?.append_child(&toggle)?;
        let last = row.insert_cell()?;
        if !is_default_host(host) {
            let (host, root) = (host.clone(), root.clone());
            last.append_child(&button("Remover", move || {
                let (host, root) = (host.clone(), root.clone());
                spawn(async move {
                    remove_site_flow(&host).await?;
                    render(root).await
                })
            }))?;
        }
    }
    on(&query::<HtmlButtonElement>(&root, "#save"), "click", {
        let root = root.clone();
        move || {
            let (fields, root) = (fields.clone(), root.clone());
            spawn(async move {
                save_provider_fields(Saved {
                    provider: fields.chosen(),
                    enabled: query::<HtmlInputElement>(&root, "#enabled").checked(),
                    key: &fields.key.value(),
                    model: &fields.model.value(),
                })
                .await?;
                render(root.clone()).await?;
                query::<HtmlElement>(&root, "#status").set_text_content(Some("Salvo."));
                Ok(())
            })
        }
    });
    on(&query::<HtmlButtonElement>(&root, "#add-site"), "click", move || {
        let root = root.clone();
        spawn(async move {
            let err: HtmlElement = query(&root, "#site-error");
            let Some(host) = normalize_host(&query::<HtmlInputElement>(&root, "#new-host").value()) else {
                err.set_text_content(Some("Site inválido. Use um domínio como exemplo.com."));
                return Ok(());
            };
            if !enable_site_flow(&host).await? {
                err.set_text_content(Some("Permissão negada."));
                return Ok(());
            }
            render(root).await
        })
    });
    Ok(())
}
