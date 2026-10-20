#!/usr/bin/env bash
# plugin-smoke — install the plugin from this checkout the way a third party would, in an empty
# Claude Code config, and check what they get: the manifests validate, the three skills load from
# real files with a description, the ask-levels hook is there, and the always-on cost stays under
# a ceiling. No login, no tokens.
#
#   scripts/plugin-smoke.sh [max-always-on-tokens]   (default 300)
set -euo pipefail

max=${1:-300}
want="cientifico, ptbr, ptbr-writter"
root=$(git rev-parse --show-toplevel)
cfg=$(mktemp -d); trap 'rm -rf "$cfg"' EXIT
export CLAUDE_CONFIG_DIR=$cfg

claude plugin validate "$root" >/dev/null
claude plugin marketplace add "$root" >/dev/null
claude plugin install ptbr-writter@ptbr-writter >/dev/null
details=$(claude plugin details ptbr-writter@ptbr-writter)

got=$(sed -n 's/^ *Skills ([0-9]*) *//p' <<<"$details")
[ "$got" = "$want" ] || { echo "plugin-smoke: skills are \"$got\", want \"$want\"" >&2; exit 1; }
grep -q '^ *Hooks (1)' <<<"$details" || { echo "plugin-smoke: want 1 hook (ask-levels)" >&2; exit 1; }
cache=$(find "$cfg/plugins/cache" -mindepth 3 -maxdepth 3 -type d | head -1)
[ -z "$(find "$cache" -type l)" ] || { echo "plugin-smoke: links left in the installed copy" >&2; exit 1; }
for f in "$cache"/skills/*/SKILL.md; do
    grep -q '^description: .' "$f" || { echo "plugin-smoke: ${f#"$cache"/} has no description" >&2; exit 1; }
done
tok=$(sed -n 's/^ *Always-on: *~\{0,1\}\([0-9]*\).*/\1/p' <<<"$details")
[ "${tok:-0}" -le "$max" ] || { echo "plugin-smoke: always-on cost ~$tok tok, ceiling $max" >&2; exit 1; }
echo "plugin-smoke: ok ($got; always-on ~$tok tok)"
