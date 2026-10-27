#!/usr/bin/env bash
# The desktop app (app/, Tauri 2): bootstrap, ui, dev, build, test, icons, screenshot. The `just app-*`
# recipes and .github/workflows/app.yml call it, on Linux, macOS and Windows (Git Bash).
#
#   bootstrap    the pinned Rust (app/rust-toolchain.toml), tauri-cli and wasm-bindgen-cli in
#                app/.tools, and a check of the system's webview libraries
#   ui           the window, Rust to wasm, into app/ui/dist
#   dev          the app with Tauri's dev loop
#   build        the installers of this system (dmg/app, deb/rpm/AppImage, msi/nsis)
#   test         fmt, clippy (host and wasm32), unit tests, the version check
#   icons        app/src-tauri/icons from icons/icon.svg (the site's favicon on paper), desktop sizes only
#   screenshot   the demo build against the fake provider, its window to docs/img/app/<os>.png
set -euo pipefail

TAURI_CLI=2.12.2
# The extension's wasm-bindgen (extension/Cargo.toml): the glue and the crate must match.
WASM_BINDGEN=0.2.129

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
app="$root/app"
tools="$app/.tools"
channel="$(sed -n 's/^channel = "\(.*\)"/\1/p' "$app/rust-toolchain.toml")"

case "$(uname -s)" in
    Darwin) os=macos ;;
    Linux) os=linux ;;
    MINGW* | MSYS* | CYGWIN*) os=windows ;;
    *) os=unknown ;;
esac

# rustup's cargo of the pinned channel when rustup is here (a Homebrew cargo may come first on PATH);
# else the PATH's (the nix dev shell).
if command -v rustup >/dev/null 2>&1; then
    if cargo_bin="$(rustup which --toolchain "$channel" cargo 2>/dev/null)"; then
        PATH="$(dirname "$cargo_bin"):$PATH"
    fi
fi
export PATH="$tools/bin:$PATH"

die() {
    echo "app: $*" >&2
    exit 1
}

has_version() { # binary, version
    command -v "$1" >/dev/null 2>&1 && "$1" --version 2>/dev/null | grep -qF "$2"
}

bootstrap() {
    if command -v rustup >/dev/null 2>&1; then
        rustup toolchain install "$channel" --profile minimal --component clippy,rustfmt --target wasm32-unknown-unknown
        PATH="$(dirname "$(rustup which --toolchain "$channel" cargo)"):$PATH"
    else
        echo "app: no rustup; using $(command -v cargo || echo 'no cargo') (needs Rust $channel or newer and wasm32-unknown-unknown)"
    fi
    command -v cargo >/dev/null || die "no cargo: install rustup (https://rustup.rs), or enter the dev shell (nix develop)"
    local have
    have="$(rustc --version | cut -d' ' -f2)"
    [ "$(printf '%s\n%s\n' "$channel" "$have" | sort -V | head -1)" = "$channel" ] || die "rustc $have is older than $channel"
    has_version cargo-tauri "$TAURI_CLI" || cargo install --locked --root "$tools" tauri-cli --version "$TAURI_CLI"
    has_version wasm-bindgen "$WASM_BINDGEN" || cargo install --locked --root "$tools" wasm-bindgen-cli --version "$WASM_BINDGEN"
    case "$os" in
        linux)
            local missing=()
            for lib in webkit2gtk-4.1 javascriptcoregtk-4.1 libsoup-3.0 gtk+-3.0 librsvg-2.0; do
                pkg-config --exists "$lib" 2>/dev/null || missing+=("$lib")
            done
            if [ ${#missing[@]} -gt 0 ]; then
                echo "app: missing ${missing[*]}. On Debian or Ubuntu:" >&2
                echo "  sudo apt-get install libwebkit2gtk-4.1-dev libgtk-3-dev libsoup-3.0-dev librsvg2-dev libayatana-appindicator3-dev patchelf" >&2
                echo "  (or enter the dev shell: nix develop)" >&2
                exit 1
            fi
            ;;
        macos) xcode-select -p >/dev/null 2>&1 || die "no Xcode command line tools: xcode-select --install" ;;
        windows) echo "app: Windows needs the MSVC build tools and WebView2 (in Windows 10 and 11 already)" ;;
        *) die "unsupported system $(uname -s)" ;;
    esac
    echo "app-bootstrap: ok (rust $(rustc --version | cut -d' ' -f2), $(cargo-tauri --version), $(wasm-bindgen --version))"
}

need_tools() {
    has_version cargo-tauri "$TAURI_CLI" && has_version wasm-bindgen "$WASM_BINDGEN" || die "run just app-bootstrap first"
}

ui() {
    need_tools
    (cd "$app" && cargo build -p ptbr-app-ui --target wasm32-unknown-unknown --release)
    rm -rf "$app/ui/dist"
    wasm-bindgen --target web --no-typescript --out-dir "$app/ui/dist" "$app/target/wasm32-unknown-unknown/release/ptbr_app_ui.wasm"
    cp "$app"/ui/static/* "$app/ui/dist/"
    echo "app-ui: $(du -sh "$app/ui/dist" | cut -f1) in app/ui/dist"
}

check_version() {
    local plugin crate
    plugin="$(sed -n 's/.*"version": *"\([^"]*\)".*/\1/p' "$root/plugins/ptbr-writer/.claude-plugin/plugin.json" | head -1)"
    crate="$(sed -n 's/^version = "\(.*\)"/\1/p' "$app/Cargo.toml" | head -1)"
    [ "$plugin" = "$crate" ] || die "app/Cargo.toml has $crate, plugin.json has $plugin: one version for both"
}

test_all() {
    check_version
    # generate_context! embeds app/ui/dist: clippy and the tests need it built.
    ui
    cd "$app"
    cargo fmt --all -- --check
    cargo clippy -p ptbr-app -p ptbr-app-wire --all-targets -- -D warnings
    cargo clippy -p ptbr-app --all-targets --features demo -- -D warnings
    cargo clippy -p ptbr-app-ui --target wasm32-unknown-unknown -- -D warnings
    cargo test -p ptbr-app -p ptbr-app-wire
    cargo test -p ptbr-app --features demo
    echo "app-test: ok"
}

# The window's picture on this system: the demo build checks fake::DEMO_TEXT against the fake
# provider (no key, no network), then the window is captured.
screenshot() {
    ui
    (cd "$app" && cargo build -p ptbr-app --features demo --bins)
    local bin="$app/target/debug" exe="" port=11999 out="$root/docs/img/app/$os.png"
    [ "$os" = windows ] && exe=.exe
    local cfg
    cfg="$(mktemp -d)"
    printf '{"provider":"ollama","apiBase":{"ollama":"http://127.0.0.1:%s"},"level":"descritivo"}\n' "$port" >"$cfg/settings.json"
    "$bin/fake-provider$exe" "$port" 2>/dev/null &
    fake=$!
    trap 'kill "$fake" 2>/dev/null || true; rm -rf "$cfg"' EXIT
    local text
    text="$(sed -n 's/^pub const DEMO_TEXT: &str = "\(.*\)";$/\1/p' "$app/src-tauri/src/fake.rs")"
    mkdir -p "$(dirname "$out")"
    export PTBR_APP_CONFIG_DIR="$cfg" PTBR_APP_DEMO_TEXT="$text"
    local capture=(bash "$root/scripts/app-capture.sh" "$os" "$bin/ptbr-app$exe" "$out")
    # Linux without a display: Xvfb, one screen the window's size.
    if [ "$os" = linux ] && [ -z "${DISPLAY:-}" ]; then
        xvfb-run -a -s "-screen 0 1100x760x24" "${capture[@]}"
    else
        "${capture[@]}"
    fi
    echo "app-screenshot: $out"
}

icons() {
    need_tools
    cd "$app/src-tauri"
    cargo-tauri tauri icon icons/icon.svg
    # No mobile or Store build: their sizes would be files nothing reads.
    rm -rf icons/android icons/ios icons/Square*Logo.png icons/StoreLogo.png
}

cmd="${1:-}"
shift || true
case "$cmd" in
    bootstrap) bootstrap ;;
    ui) ui ;;
    dev) ui && cd "$app/src-tauri" && cargo-tauri tauri dev "$@" ;;
    build) ui && cd "$app/src-tauri" && cargo-tauri tauri build "$@" ;;
    test) test_all ;;
    icons) icons ;;
    screenshot) screenshot ;;
    *) die "usage: scripts/app.sh bootstrap|ui|dev|build|test|icons|screenshot" ;;
esac
