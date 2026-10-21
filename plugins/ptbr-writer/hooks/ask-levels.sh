#!/usr/bin/env bash
# ask-levels — UserPromptSubmit hook: a request that starts with `ptbr` gets one instruction (the
# logic and the config file's rules are in the ptbr-ask-levels binary; bin/run.sh picks it for
# this OS and warns when there is none).
#
#   ask-levels.sh < hook-input.json
#   ask-levels.sh --self-test
set -euo pipefail

root="${CLAUDE_PLUGIN_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
if [ "${1-}" = --self-test ]; then exec bash "$root/bin/run.sh" ask-levels --self-test; fi
exec bash "$root/bin/run.sh" ask-levels "$root/skills/ptbr-writer/SKILL.md" "$root/scripts/to-pdf.sh" "$root/ptbr-writer.default.toml" "bash $root/bin/run.sh reading-time"
