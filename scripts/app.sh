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
#
# Android (gen/android; macOS or Linux, the Java 17+ JDK, the Android SDK in ANDROID_HOME):
#
#   android-bootstrap   the desktop tools, the Android Rust targets, and the pinned SDK packages
#   android-test        clippy for aarch64-linux-android (android.rs builds only there)
#   android-dev         the app on a running emulator or a phone (adb), with Tauri's dev loop
#   android-build       a debug APK (installable with adb, no keystore) for the phones (arm64)
#   android-screenshot  the demo APK on a running emulator: its overlay over Messages, checked against
#                       the fake Claude, to docs/img/app/android-*.png (scripts/app-android-capture.sh)
set -euo pipefail

TAURI_CLI=2.12.2
# The extension's wasm-bindgen (extension/Cargo.toml): the glue and the crate must match.
WASM_BINDGEN=0.2.129
# The Android SDK packages (sdkmanager): the NDK that builds the Rust, the platform compileSdk names
# (gen/android/app/build.gradle.kts).
ANDROID_NDK=27.3.13750724
ANDROID_PLATFORM=android-37.0

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
    # Globals, not locals: the EXIT trap runs after this function returns.
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

# ANDROID_HOME, NDK_HOME and a JDK 17 or newer for Gradle; the SDK's default folder when unset.
android_env() {
    if [ -z "${ANDROID_HOME:-}" ]; then
        case "$os" in
            macos) ANDROID_HOME="$HOME/Library/Android/sdk" ;;
            *) ANDROID_HOME="$HOME/Android/Sdk" ;;
        esac
    fi
    [ -d "$ANDROID_HOME" ] || die "no Android SDK in $ANDROID_HOME: install Android Studio or the command line tools, or set ANDROID_HOME"
    export ANDROID_HOME NDK_HOME="$ANDROID_HOME/ndk/$ANDROID_NDK"
    if [ -n "${JAVA_HOME_17_X64:-}" ] && [ -z "${JAVA_HOME:-}" ]; then
        export JAVA_HOME="$JAVA_HOME_17_X64"
    elif [ "$os" = macos ] && [ -z "${JAVA_HOME:-}" ]; then
        JAVA_HOME="$(/usr/libexec/java_home -v 17+ 2>/dev/null)" || die "no JDK 17 or newer: brew install --cask temurin@21"
        export JAVA_HOME
    fi
    PATH="$ANDROID_HOME/platform-tools:$PATH"
}

sdkmanager_bin() {
    local s
    for s in "$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager" "$(command -v sdkmanager 2>/dev/null)"; do
        [ -x "$s" ] && { echo "$s"; return; }
    done
    die "no sdkmanager: install the Android command line tools (cmdline-tools;latest)"
}

android_bootstrap() {
    bootstrap
    android_env
    # Phones (arm64) and the emulator (x86_64); Tauri's other two, 32-bit, are not built.
    rustup target add --toolchain "$channel" aarch64-linux-android x86_64-linux-android
    if [ ! -d "$NDK_HOME" ] || [ ! -d "$ANDROID_HOME/platforms/$ANDROID_PLATFORM" ]; then
        yes | "$(sdkmanager_bin)" --sdk_root="$ANDROID_HOME" --licenses >/dev/null || true
        "$(sdkmanager_bin)" --sdk_root="$ANDROID_HOME" "ndk;$ANDROID_NDK" "platforms;$ANDROID_PLATFORM" "platform-tools"
    fi
    "$JAVA_HOME/bin/java" -version 2>&1 | head -1
    echo "app-android-bootstrap: ok (NDK $ANDROID_NDK, $ANDROID_PLATFORM, $ANDROID_HOME)"
}

# The phones' debug APK, copied to app/target/android (android-screenshot builds over Gradle's output).
android_build() {
    android_env
    need_tools
    ui
    (cd "$app/src-tauri" && cargo-tauri android build --debug --apk --target aarch64 "$@")
    local version
    version="$(sed -n 's/^version = "\(.*\)"/\1/p' "$app/Cargo.toml" | head -1)"
    mkdir -p "$app/target/android"
    cp "$app/src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk" "$app/target/android/ptbr-writer-$version-arm64-debug.apk"
    echo "app-android-build: app/target/android/ptbr-writer-$version-arm64-debug.apk"
}

# clippy and the unit tests' build for Android: android.rs and the cfg(target_os = "android") code
# compile only there. The NDK's clang builds ring's C.
android_test() {
    android_env
    ui
    local host bin
    case "$os" in macos) host=darwin-x86_64 ;; *) host=linux-x86_64 ;; esac
    bin="$NDK_HOME/toolchains/llvm/prebuilt/$host/bin"
    export CC_aarch64_linux_android="$bin/aarch64-linux-android24-clang" AR_aarch64_linux_android="$bin/llvm-ar"
    export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$bin/aarch64-linux-android24-clang"
    cd "$app"
    cargo clippy -p ptbr-app --target aarch64-linux-android --all-targets -- -D warnings
    cargo clippy -p ptbr-app --target aarch64-linux-android --all-targets --features demo -- -D warnings
    echo "app-android-test: ok"
}

# The demo APK on the emulator adb sees, against the fake Claude on this machine (adb reverse).
android_screenshot() {
    android_env
    need_tools
    ui
    local abi="${ANDROID_SCREENSHOT_TARGET:-x86_64}" port=11999
    (cd "$app/src-tauri" && cargo-tauri android build --debug --apk --target "$abi" --features demo)
    (cd "$app" && cargo build -p ptbr-app --features demo --bin fake-provider)
    "$app/target/debug/fake-provider" "$port" 2>/dev/null &
    fake=$!
    trap 'kill "$fake" 2>/dev/null || true' EXIT
    local text
    text="$(sed -n 's/^pub const DEMO_TEXT: &str = "\(.*\)";$/\1/p' "$app/src-tauri/src/fake.rs")"
    bash "$root/scripts/app-android-capture.sh" "$app/src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk" "$port" "$text" "$root/docs/img/app"
}

icons() {
    need_tools
    cd "$app/src-tauri"
    cargo-tauri tauri icon icons/icon.svg
    # Android's sizes go to gen/android's res (Tauri writes them there); no iOS or Store build, so
    # their sizes would be files nothing reads.
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
    android-bootstrap) android_bootstrap ;;
    android-dev) android_env && need_tools && ui && cd "$app/src-tauri" && cargo-tauri android dev "$@" ;;
    android-test) android_test ;;
    android-build) android_build "$@" ;;
    android-screenshot) android_screenshot ;;
    *) die "usage: scripts/app.sh bootstrap|ui|dev|build|test|icons|screenshot|android-bootstrap|android-dev|android-test|android-build|android-screenshot" ;;
esac
