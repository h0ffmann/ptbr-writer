#!/usr/bin/env bash
# Starts the app, waits for its window and its first check, saves the window as a PNG, stops it.
# Called by `scripts/app.sh screenshot`: app-capture.sh OS BINARY OUT.PNG
set -euo pipefail
os="$1" bin="$2" out="$3"

log="$(mktemp)"
"$bin" 2>"$log" &
pid=$!
# Git Bash numbers its own processes: PowerShell wants the Windows one.
winpid="$pid"
[ "$os" = windows ] && winpid="$(cat /proc/$pid/winpid)"
trap 'kill "$pid" 2>/dev/null || true; rm -f "$log"' EXIT

# The window's id on macOS: CoreGraphics' window list, by owner process (no permission to list).
mac_window() {
    osascript -l JavaScript -e "ObjC.import('CoreGraphics');
      const l = ObjC.deepUnwrap(ObjC.castRefToObject(\$.CGWindowListCopyWindowInfo(\$.kCGWindowListOptionOnScreenOnly, 0)));
      const w = l.find(w => w.kCGWindowOwnerPID === $pid && w.kCGWindowLayer === 0);
      w ? String(w.kCGWindowNumber) : ''"
}

for _ in $(seq 60); do
    kill -0 "$pid" 2>/dev/null || { echo "app-capture: the app exited" >&2; exit 1; }
    case "$os" in
        macos) [ -n "$(mac_window)" ] && break ;;
        linux) xwininfo -root -tree 2>/dev/null | grep -q '"ptbr-writer"' && break ;;
        windows) powershell -NoProfile -Command "if ((Get-Process -Id $winpid).MainWindowHandle -ne 0) { exit 0 } else { exit 1 }" && break ;;
    esac
    sleep 0.5
done
# The window loads its wasm, then checks the demo text against the fake provider: the Rust side
# logs the cards (lib.rs). No line in 30 s is a window that did not start.
for _ in $(seq 60); do grep -q "ptbr-app demo: " "$log" && break; sleep 0.5; done
grep "ptbr-app demo: " "$log" || { echo "app-capture: the window never checked the demo text" >&2; cat "$log" >&2; exit 1; }
# The cards render right after the reply; this covers a slow runner.
sleep "${APP_CAPTURE_WAIT:-2}"

case "$os" in
    macos) screencapture -x -o -l"$(mac_window)" "$out" ;;
    linux) import -window root "$out" ;;
    windows) powershell -NoProfile -ExecutionPolicy Bypass -File "$(dirname "$0")/app-capture.ps1" -ProcessId "$winpid" -Out "$(cygpath -w "$out")" ;;
esac
test -s "$out"
