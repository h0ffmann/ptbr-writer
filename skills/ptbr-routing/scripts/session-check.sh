#!/usr/bin/env bash
# session-check — the SessionStart hook a repo's .claude/settings.json runs. On `source: startup`
# only, and only while the plugin is not installed, one line asking the agent to offer it. Local
# files only: no network, no writes. Silent otherwise, so it costs no context once installed.
#
#   session-check.sh < hook-input.json
#   session-check.sh --self-test
set -euo pipefail

PLUGIN="gramatico@gramatico"

optin_line() {
  echo "▸ gramatico plugin not installed. Before starting the person's request, if AskUserQuestion is available, ask once: \"This repo offers the gramatico plugin (the /ptbr-writer, /ptbr and /cientifico skills, pt-BR technical writing). Routing works either way.\" Options: Install (they run \`! claude plugin install ${PLUGIN}\`), Not now (nothing), Never (add \"env\": {\"PTBR_PLUGIN_OPTOUT\": \"1\"} to $1). Then carry on. With no way to ask, say nothing about it."
}

check() {
  local input source cfg
  input=$(cat 2>/dev/null || true)
  source=$(sed -n 's/.*"source" *: *"\([a-z]*\)".*/\1/p' <<<"$input" | head -1)
  [ "${source:-startup}" = startup ] || return 0
  cfg="${CLAUDE_CONFIG_DIR:-$HOME/.claude}"
  [ "${CLAUDE_CODE_REMOTE:-}" != true ] || return 0
  [ -z "${PTBR_PLUGIN_OPTOUT:-}" ] || return 0
  ! grep -q '"PTBR_PLUGIN_OPTOUT"' "$cfg/settings.json" 2>/dev/null || return 0
  ! grep -q "\"$PLUGIN\"" "$cfg/plugins/installed_plugins.json" 2>/dev/null || return 0
  optin_line "${cfg/#"$HOME"/\~}/settings.json"
}

self_test() {
  local tmp out
  tmp=$(mktemp -d); trap 'rm -rf "$tmp"' RETURN
  unset PTBR_PLUGIN_OPTOUT CLAUDE_CODE_REMOTE
  mkdir -p "$tmp/plugins"
  out=$(CLAUDE_CONFIG_DIR=$tmp check <<<'{"source":"startup"}')
  [[ $out == "▸ gramatico plugin not installed."* ]] || { echo "FAIL: no offer when not installed" >&2; return 1; }
  out=$(CLAUDE_CONFIG_DIR=$tmp check <<<'{"source":"resume"}')
  [ -z "$out" ] || { echo "FAIL: offer on resume" >&2; return 1; }
  echo "{\"plugins\": {\"$PLUGIN\": []}}" >"$tmp/plugins/installed_plugins.json"
  out=$(CLAUDE_CONFIG_DIR=$tmp check <<<'{"source":"startup"}')
  [ -z "$out" ] || { echo "FAIL: offer when installed" >&2; return 1; }
  rm "$tmp/plugins/installed_plugins.json"
  echo '{"env": {"PTBR_PLUGIN_OPTOUT": "1"}}' >"$tmp/settings.json"
  out=$(CLAUDE_CONFIG_DIR=$tmp check <<<'{"source":"startup"}')
  [ -z "$out" ] || { echo "FAIL: offer after opt-out" >&2; return 1; }
  echo "session-check self-test: ok"
}

case "${1-}" in
  --self-test) self_test ;;
  *) check ;;
esac
