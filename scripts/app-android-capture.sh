#!/usr/bin/env bash
# Installs the demo APK on the emulator adb sees, gives it the fake Claude (adb reverse) and a
# placeholder key, turns its overlay on, opens Messages with the demo text, waits for the overlay's
# check, and saves the pictures (the underlines with the badge, the cards, the text after Aceitar, the
# window) and a low-resolution video of it all (android-demo.mp4, and .gif when ffmpeg is here).
# Called by `scripts/app.sh android-screenshot`: app-android-capture.sh APK PORT TEXT OUTDIR
set -euo pipefail
# A step that fails says which one: adb often exits non-zero without a word.
trap 'echo "app-android-capture: line $LINENO failed: $BASH_COMMAND" >&2' ERR
apk="$1" port="$2" text="$3" out="$4"
pkg=io.github.h0ffmann.ptbr_writer
mkdir -p "$out"

shot() { adb exec-out screencap -p >"$out/$1"; test -s "$out/$1"; echo "app-android-capture: $out/$1"; }
# The overlay logs each step (OverlayService, tag ptbr): waits up to N half-seconds for a line.
wait_log() { # pattern, tries
    local i
    for i in $(seq "$2"); do
        adb logcat -d -s ptbr:I | grep -q "$1" && return 0
        sleep 0.5
    done
    echo "app-android-capture: no \"$1\" in the log" >&2
    adb logcat -d -s ptbr:I >&2 || true
    shot android-failed.png || true
    adb shell uiautomator dump /sdcard/ui.xml >/dev/null 2>&1 && adb shell cat /sdcard/ui.xml >"$out/android-failed.xml" || true
    return 1
}

adb wait-for-device
adb shell 'while [ "$(getprop sys.boot_completed)" != 1 ]; do sleep 1; done'
adb install -r -t "$apk" >/dev/null
adb reverse "tcp:$port" "tcp:$port"
# The settings the window would save, plus the demo build's apiBase: Claude at the fake.
printf '{"provider":"claude","keys":{"claude":"placeholder"},"apiBase":{"claude":"http://127.0.0.1:%s"},"level":"descritivo"}' "$port" |
    adb shell "run-as $pkg sh -c 'cat > settings.json'"
adb logcat -c

# Only the user turns an accessibility service on; on an emulator, adb may.
adb shell settings put secure enabled_accessibility_services "$pkg/$pkg.OverlayService"
adb shell settings put secure accessibility_enabled 1
wait_log "overlay: on" 40
adb shell input keyevent KEYCODE_HOME

# The video: half the screen's size, 1 Mbit/s, from Messages opening to Aceitar (screenrecord's
# limit is 3 minutes).
adb shell "screenrecord --size 540x1200 --bit-rate 1000000 /sdcard/demo.mp4" &
rec=$!
sleep 1
adb shell "am start -W -a android.intent.action.SENDTO -d sms:5550100 --es sms_body '$text'" >/dev/null
sleep 3
# The compose box got its text from the intent, but may not have the focus: a tap in it (its bounds
# from uiautomator), then a space typed in it, the change the overlay waits for.
adb shell uiautomator dump /sdcard/ui.xml >/dev/null
box="$(adb shell cat /sdcard/ui.xml | grep -o 'class="android.widget.EditText"[^>]*bounds="[^"]*"' | sed -n 's/.*bounds="\[\([0-9]*\),\([0-9]*\)\]\[\([0-9]*\),\([0-9]*\)\]".*/\1 \2 \3 \4/p' | head -1)"
[ -n "$box" ] || { echo "app-android-capture: no text field in Messages" >&2; shot android-failed.png; exit 1; }
read -r x1 y1 x2 y2 <<<"$box"
adb shell input tap $(((x1 + x2) / 2)) $(((y1 + y2) / 2))
sleep 1
# Ctrl+End: the end of the whole text (End alone stops at the line the tap hit).
adb shell input keycombination KEYCODE_CTRL_LEFT KEYCODE_MOVE_END
adb shell input text '%s'
# The fake answers two suggestions (fake.rs): one card fewer is a text the typing changed.
wait_log "overlay: 2 cards" 60
sleep 2
shot android-overlay.png

xy="$(adb logcat -d -s ptbr:I | sed -n 's/.*overlay: badge at \([0-9]*\),\([0-9]*\).*/\1 \2/p' | tail -1)"
[ -n "$xy" ] || { echo "app-android-capture: no badge" >&2; exit 1; }
# shellcheck disable=SC2086
adb shell input tap $xy
sleep 2
shot android-cards.png

# Aceitar on the first card: the field gets the new text, and its card closes.
acc="$(adb logcat -d -s ptbr:I | sed -n 's/.*overlay: accept at \([0-9]*\),\([0-9]*\).*/\1 \2/p' | tail -1)"
[ -n "$acc" ] || { echo "app-android-capture: no Aceitar" >&2; exit 1; }
# shellcheck disable=SC2086
adb shell input tap $acc
sleep 3
shot android-accepted.png
adb shell pkill -INT screenrecord || true
wait "$rec" 2>/dev/null || true
sleep 2
adb pull /sdcard/demo.mp4 "$out/android-demo.mp4" >/dev/null
# A GIF plays inline in a PR and a README: 8 frames a second, 270 px wide, one palette.
if command -v ffmpeg >/dev/null; then
    ffmpeg -loglevel error -y -i "$out/android-demo.mp4" \
        -vf "fps=8,scale=270:-1:flags=lanczos,split[a][b];[a]palettegen=max_colors=96[p];[b][p]paletteuse=dither=bayer" \
        "$out/android-demo.gif"
    echo "app-android-capture: $out/android-demo.gif ($(du -h "$out/android-demo.gif" | cut -f1))"
fi

# The window last, once the emulator has settled (a launcher still starting shows a dialog): its
# overlay section, now on; the window in front puts the overlay away.
adb shell am broadcast -a android.intent.action.CLOSE_SYSTEM_DIALOGS >/dev/null || true
adb shell am start -W -n "$pkg/.MainActivity" >/dev/null
sleep "${APP_CAPTURE_WAIT:-6}"
adb shell am broadcast -a android.intent.action.CLOSE_SYSTEM_DIALOGS >/dev/null || true
shot android-window.png
