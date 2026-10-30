#!/usr/bin/env bash
# opencode-smoke — install gramatico for OpenCode from this checkout into an empty directory, the
# way scripts/install-opencode.sh does for a third party, and check what they get: the three skills
# as real files with `name` (= folder) and `description`, no links, the binaries, and the plugin
# (called with a fake message: `ptbr ...` gets one instruction, anything else gets nothing).
# No login, no tokens, no OpenCode needed.
#
#   scripts/opencode-smoke.sh
set -euo pipefail

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
dest=$(mktemp -d); trap 'rm -rf "$dest"' EXIT
PTBR_SRC=$root OPENCODE_CONFIG_DIR=$dest bash "$root/scripts/install-opencode.sh" v0.0.0 >/dev/null

fail() { echo "opencode-smoke: $*" >&2; exit 1; }
[ -z "$(find "$dest" -type l)" ] || fail "links left in the installed copy"
for s in ptbr-writer ptbr cientifico; do
    f=$dest/skills/$s/SKILL.md
    [ -f "$f" ] || fail "skills/$s/SKILL.md is not a file"
    [ "$(sed -n 's/^name: *//p' "$f" | head -1)" = "$s" ] || fail "skills/$s/SKILL.md: name is not \"$s\""
    grep -q '^description: .' "$f" || fail "skills/$s/SKILL.md has no description"
done
[ -f "$dest/gramatico/bin/run.sh" ] || fail "bin/run.sh is missing"
for f in "$dest"/gramatico/bin/linux-*/ptbr; do [ -x "$f" ] || fail "${f#"$dest"/} is not executable"; done
[ -f "$dest/plugins/gramatico.js" ] || fail "plugins/gramatico.js is missing"

# The plugin, called as OpenCode calls it (the hook `chat.message`).
cat > "$dest/call.mjs" <<'JS'
const { PtbrWriter } = await import(process.argv[2])
const hook = (await PtbrWriter())["chat.message"]
const run = async (text) => {
  const output = { message: { id: "msg_1" }, parts: [{ type: "text", text }] }
  await hook({ sessionID: "ses_1" }, output)
  return output.parts.length
}
const [a, b] = [await run("ptbr escreva um runbook"), await run("escreva um runbook")]
if (a !== 2 || b !== 1) { console.error(`parts: ptbr=${a} (want 2), other=${b} (want 1)`); process.exit(1) }
JS
if command -v node >/dev/null; then node "$dest/call.mjs" "$dest/plugins/gramatico.js" || fail "the plugin did not add (or added too much)"
else echo "opencode-smoke: no node; plugin call skipped" >&2; fi
echo "opencode-smoke: ok (ptbr-writer, ptbr, cientifico; plugin ask-levels)"
