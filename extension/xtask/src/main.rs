//! `cargo xtask build [--test] [--firefox]`: the extension into dist, dist-test or dist-firefox,
//! as build.mjs did: the wasm, wasm-bindgen's glue, wasm-opt, the static files and the manifest.

use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let flags: Vec<&str> = args.iter().map(String::as_str).collect();
    match flags.first() {
        Some(&"build") => match build(flags.contains(&"--test"), flags.contains(&"--firefox")) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("xtask: {e}");
                ExitCode::FAILURE
            }
        },
        _ => {
            eprintln!("usage: cargo xtask build [--test] [--firefox]");
            ExitCode::FAILURE
        }
    }
}

// The extension directory: the workspace root, where Cargo.toml and static/ live.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf()
}

fn run(cmd: &mut Command) -> Result<(), String> {
    let status = cmd.status().map_err(|e| format!("{:?}: {e}", cmd.get_program()))?;
    if status.success() { Ok(()) } else { Err(format!("{:?} failed: {status}", cmd.get_program())) }
}

fn size(p: &Path) -> Result<u64, String> {
    Ok(fs::metadata(p).map_err(|e| format!("{}: {e}", p.display()))?.len())
}

fn build(test: bool, firefox: bool) -> Result<(), String> {
    let root = root();
    let out = root.join(if test {
        "dist-test"
    } else if firefox {
        "dist-firefox"
    } else {
        "dist"
    });
    let _ = fs::remove_dir_all(&out);
    fs::create_dir_all(&out).map_err(|e| e.to_string())?;

    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let mut c = Command::new(cargo);
    c.current_dir(&root).args(["build", "--release", "--package", "ptbr-ext", "--target", "wasm32-unknown-unknown"]);
    if test {
        c.args(["--features", "test-build"]);
    }
    run(&mut c)?;

    let wasm = root.join("target/wasm32-unknown-unknown/release/ptbr_ext.wasm");
    run(Command::new("wasm-bindgen")
        .args(["--target", "web", "--out-name", "ptbr_ext", "--no-typescript", "--out-dir"])
        .arg(&out)
        .arg(&wasm))?;

    let bg = out.join("ptbr_ext_bg.wasm");
    let before = size(&bg)?;
    // The wasm features rustc's wasm32-unknown-unknown target emits by default; every browser the
    // manifest allows (Chrome 120, Firefox 128) runs them.
    let features = [
        "--enable-bulk-memory",
        "--enable-bulk-memory-opt",
        "--enable-mutable-globals",
        "--enable-nontrapping-float-to-int",
        "--enable-sign-ext",
        "--enable-reference-types",
        "--enable-multivalue",
    ];
    run(Command::new("wasm-opt").arg("-Oz").args(features).arg(&bg).arg("-o").arg(&bg))?;
    println!("ptbr_ext_bg.wasm: {before} bytes, {} after wasm-opt", size(&bg)?);

    copy_dir(&root.join("static"), &out)?;
    copy_dir(&root.join("loaders"), &out)?;
    let manifest = manifest(&root, test, firefox)?;
    fs::write(out.join("manifest.json"), serde_json::to_string_pretty(&manifest).unwrap() + "\n")
        .map_err(|e| e.to_string())
}

fn copy_dir(from: &Path, to: &Path) -> Result<(), String> {
    for entry in fs::read_dir(from).map_err(|e| format!("{}: {e}", from.display()))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let target = to.join(entry.file_name());
        if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            fs::create_dir_all(&target).map_err(|e| e.to_string())?;
            copy_dir(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target).map_err(|e| format!("{}: {e}", entry.path().display()))?;
        }
    }
    Ok(())
}

fn manifest(root: &Path, test: bool, firefox: bool) -> Result<Value, String> {
    let plugin_path = root.join("../plugins/ptbr-writer/.claude-plugin/plugin.json");
    let plugin: Value =
        serde_json::from_str(&fs::read_to_string(&plugin_path).map_err(|e| format!("{}: {e}", plugin_path.display()))?)
            .map_err(|e| e.to_string())?;
    let local: Vec<Value> = if test { vec!["http://127.0.0.1/*".into(), "http://localhost/*".into()] } else { vec![] };
    let mut host_permissions = vec![json!("https://api.anthropic.com/*"), json!("https://api.deepseek.com/*")];
    host_permissions.extend(local.iter().cloned());
    let mut matches =
        vec![json!("https://web.whatsapp.com/*"), json!("https://mail.google.com/*"), json!("https://github.com/*")];
    matches.extend(local.iter().cloned());
    let mut accessible = vec![json!("https://*/*")];
    accessible.extend(local);
    let mut m = json!({
        "manifest_version": 3,
        "name": "ptbr-writer",
        "version": plugin["version"],
        "description": plugin["description"],
        "permissions": ["storage", "scripting", "activeTab"],
        "host_permissions": host_permissions,
        "optional_host_permissions": ["http://127.0.0.1:11434/*", "http://127.0.0.1:11435/*", "<all_urls>"],
        "background": { "service_worker": "background.js", "type": "module" },
        "options_page": "options.html",
        "action": { "default_popup": "popup.html" },
        // The extension pages run wasm; the content script loads the glue and the wasm through
        // chrome.runtime.getURL, so they must be web accessible on every https site, as the user
        // enables sites beyond the default three (background: registerContentScripts). The dynamic
        // URL (Chrome only; Firefox has no such key) keeps a page from fingerprinting the extension by
        // its static id; the e2e suite runs the import through it.
        "content_security_policy": { "extension_pages": "script-src 'self' 'wasm-unsafe-eval'; object-src 'self'" },
        "web_accessible_resources": [{ "resources": ["ptbr_ext.js", "ptbr_ext_bg.wasm"], "matches": accessible, "use_dynamic_url": !firefox }],
        "content_scripts": [{ "matches": matches, "js": ["content.js"], "css": ["content.css"] }],
    });
    // Firefox runs no background service worker: an event page (scripts) and an add-on id instead.
    if firefox {
        m["background"] = json!({ "scripts": ["background.js"], "type": "module" });
        m["browser_specific_settings"] =
            json!({ "gecko": { "id": "ptbr-writer@h0ffmann", "strict_min_version": "128.0" } });
    }
    Ok(m)
}
