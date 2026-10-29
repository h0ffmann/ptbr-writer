//! The ptbr-writer app: the browser extension's check (`ptbr_ext::background::handle_check`) behind
//! five Tauri commands. The window (../ui, Rust compiled to wasm) only draws what they return. On
//! Android the same library also serves the overlay service (overlay.rs, through android.rs), and
//! both go to Claude only.

#[cfg(target_os = "android")]
mod android;
#[cfg(any(test, feature = "demo"))]
pub mod fake;
pub mod fetch;
pub mod overlay;
pub mod store;
pub mod view;

use ptbr_app_wire::{Boot, Prefs, ProviderInfo, View};
use ptbr_ext::background::handle_check;
use ptbr_ext::popup::provider_name;
use ptbr_ext::settings::models;
use ptbr_ext::types::{CheckError, CheckRequest, CheckResult, Level, ProviderId, Settings, Tone};
use std::path::PathBuf;
use std::sync::Mutex;
use store::Stored;
use tauri::{Manager, State};
use view::Session;

/// Android: the window offers Claude alone, and every check goes to Claude.
const ANDROID: bool = cfg!(target_os = "android");

struct App {
    dir: PathBuf,
    stored: Mutex<Stored>,
    session: Mutex<Session>,
}

pub fn message(e: CheckError) -> &'static str {
    match e {
        CheckError::NoKey => "Sem chave: cole a chave do provider em Configurações.",
        CheckError::Auth => "Chave inválida: o provider recusou a chave.",
        CheckError::RateLimit => "Limite do provider atingido: tente de novo em instantes.",
        CheckError::Network => "Sem resposta do provider: verifique a conexão.",
        CheckError::Local => "Ollama inacessível em 127.0.0.1:11434 (just ext-ollama).",
        CheckError::LocalOllaya => "Ollaya inacessível em 127.0.0.1:11435 (just ext-ollaya).",
        CheckError::Request => "O provider recusou o pedido: confira o modelo em Configurações.",
        CheckError::Busy => "O provider está fora do ar ou sobrecarregado: tente de novo em instantes.",
        CheckError::Truncated => "A resposta do provider veio cortada: revise um trecho menor.",
        CheckError::Provider | CheckError::Reload => "O provider devolveu uma resposta que não dá para ler.",
    }
}

/// One check, start to end, on a thread of its own: the extension's check holds `Rc`s (its abort
/// signal), so it cannot cross threads, and runs on a runtime of its own thread.
pub fn run_check(text: &str, level: Level, settings: &Settings) -> CheckResult {
    let req = CheckRequest { key: "app".into(), text: text.into(), level };
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("a tokio runtime");
    rt.block_on(handle_check(&req, &fetch::Http::default(), settings))
}

#[tauri::command]
fn boot(app: State<'_, App>) -> Boot {
    let demo = if cfg!(feature = "demo") { std::env::var("PTBR_APP_DEMO_TEXT").ok() } else { None };
    Boot {
        version: env!("CARGO_PKG_VERSION").into(),
        levels: Level::ALL.iter().map(|l| l.as_str().into()).collect(),
        tones: Tone::ALL.iter().map(|t| t.as_str().into()).collect(),
        providers: ProviderId::ALL
            .iter()
            .filter(|&&p| !ANDROID || p == ProviderId::Claude)
            .map(|&p| ProviderInfo {
                id: p.as_str().into(),
                name: provider_name(p).into(),
                local: p.is_local(),
                models: models(p).iter().map(|m| m.to_string()).collect(),
            })
            .collect(),
        prefs: app.stored.lock().expect("settings").prefs(),
        demo,
    }
}

/// The prefs as saved: a value the settings do not accept comes back as it was.
#[tauri::command]
fn save_prefs(app: State<'_, App>, prefs: Prefs) -> Result<Prefs, String> {
    let mut stored = app.stored.lock().expect("settings");
    let mut next = stored.with_prefs(&prefs);
    if ANDROID {
        next.settings = overlay::claude_only(next.settings);
    }
    next.save(&app.dir).map_err(|e| format!("Não salvou {}: {e}", app.dir.join(store::FILE).display()))?;
    *stored = next;
    Ok(stored.prefs())
}

#[tauri::command]
async fn check(app: State<'_, App>, text: String, level: String) -> Result<View, String> {
    let level = Level::parse(&level).ok_or("Nível desconhecido.")?;
    let mut settings = app.stored.lock().expect("settings").settings.clone();
    if ANDROID {
        settings = overlay::claude_only(settings);
    }
    let sent = text.clone();
    let result = tauri::async_runtime::spawn_blocking(move || run_check(&sent, level, &settings))
        .await
        .map_err(|e| e.to_string())?;
    match result {
        CheckResult::Ok(suggestions) => {
            let mut session = app.session.lock().expect("session");
            *session = Session::new(text, suggestions);
            let view = session.view();
            // scripts/app-capture.sh waits for this line: the window ran its check end to end.
            if cfg!(feature = "demo") {
                eprintln!("ptbr-app demo: {} cards", view.cards.len());
            }
            Ok(view)
        }
        CheckResult::Err(e) => Err(message(e).into()),
    }
}

#[tauri::command]
fn accept(app: State<'_, App>, id: usize) -> View {
    let mut session = app.session.lock().expect("session");
    session.accept(id);
    session.view()
}

#[tauri::command]
fn ignore(app: State<'_, App>, id: usize) -> View {
    let mut session = app.session.lock().expect("session");
    session.ignore(id);
    session.view()
}

/// The settings folder: the OS's config folder for the app, or PTBR_APP_CONFIG_DIR in the demo build.
fn config_dir(app: &tauri::App) -> PathBuf {
    let demo = if cfg!(feature = "demo") { std::env::var_os("PTBR_APP_CONFIG_DIR") } else { None };
    demo.map(PathBuf::from).unwrap_or_else(|| app.path().app_config_dir().expect("a config folder"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let dir = config_dir(app);
            let stored = Mutex::new(Stored::load(&dir));
            app.manage(App { dir, stored, session: Mutex::new(Session::default()) });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![boot, save_prefs, check, accept, ignore])
        .run(tauri::generate_context!())
        .expect("the ptbr-writer window");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cloud_provider_without_a_key_never_sends() {
        let r = run_check("Um texto qualquer aqui.", Level::Leve, &Settings::default());
        assert_eq!(r, CheckResult::Err(CheckError::NoKey));
    }
}

/// The check over real HTTP. The local URL is the extension's apiBase, honored in the demo build only.
#[cfg(all(test, feature = "demo"))]
mod http_tests {
    use super::*;
    use ptbr_ext::types::Severity;
    use std::net::TcpListener;

    /// reqwest to the fake Ollama, its reply to suggestions and cards.
    #[test]
    fn a_check_over_http_returns_the_fake_providers_suggestions() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        std::thread::spawn(move || fake::serve(listener));
        let mut settings = Settings { provider: ProviderId::Ollama, ..Settings::default() };
        settings.api_base.insert(ProviderId::Ollama, base);
        let CheckResult::Ok(s) = run_check(fake::DEMO_TEXT, Level::Descritivo, &settings) else {
            panic!("the check failed")
        };
        assert_eq!(s.len(), 2);
        assert_eq!((s[0].rule.as_str(), s[0].severity), ("PTW-1", Severity::Erro));
        assert_eq!((s[1].rule.as_str(), s[1].severity), ("PTW-17", Severity::Aviso));
        let v = Session::new(fake::DEMO_TEXT.into(), s).view();
        assert_eq!(v.cards.len(), 2);
    }

    #[test]
    fn nothing_listening_is_a_local_error() {
        let port = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        let mut settings = Settings { provider: ProviderId::Ollama, ..Settings::default() };
        settings.api_base.insert(ProviderId::Ollama, format!("http://127.0.0.1:{port}"));
        assert_eq!(run_check("Um texto qualquer aqui.", Level::Leve, &settings), CheckResult::Err(CheckError::Local));
    }
}
