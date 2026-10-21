#!/usr/bin/env bash
# run.sh — run the skill's static binary for this OS and CPU: run.sh [route's args].
# ptbr finds routes.tsv from its argv[0], so it is started as <skill>/scripts/ptbr-route.
# No binary for the platform: one line on stderr, exit 1; the skill then reads routes.tsv by hand.
set -euo pipefail

here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
plat="$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m)"
bin="$here/$plat/ptbr"
if [ -x "$bin" ]; then exec -a "$(dirname "$here")/scripts/ptbr-route" "$bin" "$@"; fi
echo "ptbr-routing: no binary for $plat; read routes.tsv by hand" >&2
exit 1
