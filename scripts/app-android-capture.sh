#!/usr/bin/env bash
# Installs the demo APK on the emulator adb sees, gives it the fake Claude (adb reverse) and a
# placeholder key, turns its overlay on, opens Messages with the demo text, waits for the overlay's
# check, and saves three pictures: the underlines with the badge, the cards, the window.
# Called by `scripts/app.sh android-screenshot`: app-android-capture.sh APK PORT TEXT OUTDIR
set -euo pipefail
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
adb shell input keyevent KEYCODE_MOVE_END
adb shell input text '%s'
wait_log "overlay: [1-9][0-9]* cards" 60
sleep 2
shot android-overlay.png

xy="$(adb logcat -d -s ptbr:I | sed -n 's/.*overlay: badge at \([0-9]*\),\([0-9]*\).*/\1 \2/p' | tail -1)"
[ -n "$xy" ] || { echo "app-android-capture: no badge" >&2; exit 1; }
# shellcheck disable=SC2086
adb shell input tap $xy
sleep 2
shot android-cards.png

# The window last, once the emulator has settled (a launcher still starting shows a dialog): its
# overlay section, now on; the window in front puts the overlay away.
adb shell am broadcast -a android.intent.action.CLOSE_SYSTEM_DIALOGS >/dev/null || true
adb shell am start -W -n "$pkg/.MainActivity" >/dev/null
sleep "${APP_CAPTURE_WAIT:-6}"
adb shell am broadcast -a android.intent.action.CLOSE_SYSTEM_DIALOGS >/dev/null || true
shot android-window.png
