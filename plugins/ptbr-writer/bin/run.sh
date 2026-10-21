#!/usr/bin/env bash
# run.sh — run the plugin's static binary for this OS and CPU: run.sh <ask-levels|reading-time> [args].
# No binary for the platform: one line on stderr and exit 0; for the hook, also a hook JSON so
# Claude tells the person (a request that starts with `ptbr`), never silence.
set -euo pipefail

here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
cmd=${1:?usage: run.sh <ask-levels|reading-time> [args]}; shift
plat="$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m)"
bin="$here/$plat/ptbr"
if [ -x "$bin" ]; then exec "$bin" "$cmd" "$@"; fi

msg="ptbr-writer: no binary for $plat (bin/$plat/ptbr); the plugin ships binaries for linux-x86_64 and linux-aarch64 only. Reading time and the ptbr question are off."
echo "$msg" >&2
if [ "$cmd" = ask-levels ]; then
    input=$(cat 2>/dev/null || true)
    if grep -qiE '"prompt" *: *" *ptbr' <<<"$input"; then
        printf '{"hookSpecificOutput":{"hookEventName":"UserPromptSubmit","additionalContext":"%s Tell the person this once, then answer the request normally."}}\n' "$msg"
    fi
fi
exit 0
