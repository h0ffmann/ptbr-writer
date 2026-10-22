#!/usr/bin/env bash
# Mozilla's Firefox MCP (firefox-devtools-mcp), pinned, on a throwaway profile: the agent drives a
# Firefox and installs extension/dist-firefox with install_extension (type "path"). Extra flags
# (--headless, --start-url URL) pass through.
set -euo pipefail
args=()
# Ubuntu's /usr/bin/firefox is a shell wrapper around the snap, and geckodriver cannot start it.
snap=/snap/firefox/current/usr/lib/firefox/firefox
[ -x "$snap" ] && args+=(--firefox-path "$snap")
exec npx -y @mozilla/firefox-devtools-mcp@0.10.4 "${args[@]}" "$@"
