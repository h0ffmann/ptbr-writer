#!/usr/bin/env bash
# ask-levels — UserPromptSubmit hook: a request that starts with `ptbr` gets one instruction (the
# logic and the config file's rules are in ask_levels.py). Needs python3; without it, silent.
#
#   ask-levels.sh < hook-input.json
#   ask-levels.sh --self-test
set -euo pipefail

root="${CLAUDE_PLUGIN_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
command -v python3 >/dev/null || exit 0
if [ "${1-}" = --self-test ]; then exec python3 -I "$root/hooks/ask_levels.py" --self-test; fi
exec python3 -I "$root/hooks/ask_levels.py" "$root/skills/ptbr-writer/SKILL.md" "$root/scripts/to-pdf.sh" "$root/ptbr-writer.default.toml"
