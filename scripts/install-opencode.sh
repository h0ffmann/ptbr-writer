#!/usr/bin/env bash
# install-opencode — install ptbr-writer into OpenCode from a release tag: the skills as real files
# (links followed, `name:` added where the file has none, since OpenCode requires it), the pinned
# ptbr binaries, and the ask-levels plugin. Never main: the tag is required.
#
#   install-opencode.sh vX.Y.Z [--project]   (--project: ./.opencode instead of ~/.config/opencode)
#   PTBR_SRC=<checkout> install-opencode.sh vX.Y.Z [--project]   (no download; the smoke test uses it)
set -euo pipefail

tag=${1:?usage: install-opencode.sh vX.Y.Z [--project]}
[[ $tag =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "install-opencode: \"$tag\" is not a tag vX.Y.Z" >&2; exit 1; }
if [ "${2-}" = --project ]; then dest=$PWD/.opencode; else dest=${OPENCODE_CONFIG_DIR:-${XDG_CONFIG_HOME:-$HOME/.config}/opencode}; fi

tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
if [ -n "${PTBR_SRC-}" ]; then src=$PTBR_SRC
else
    curl -fsSL "https://github.com/h0ffmann/ptbr-writer/archive/refs/tags/$tag.tar.gz" | tar -xz -C "$tmp"
    src=$tmp/ptbr-writer-${tag#v}
fi

skill() { # skill <name> <source SKILL.md>
    mkdir -p "$dest/skills/$1"
    # awk, not sed -i: BSD sed (macOS) has neither GNU's `0,/re/` nor a bare -i.
    if grep -q '^name:' "$2"; then cp -L "$2" "$dest/skills/$1/SKILL.md"
    else awk -v n="$1" '!d && /^---$/ {print; print "name: " n; d=1; next} {print}' "$2" > "$dest/skills/$1/SKILL.md"
    fi
}
rm -rf "$dest/skills/ptbr-writer" "$dest/skills/ptbr" "$dest/skills/cientifico" "$dest/ptbr-writer"
skill ptbr-writer "$src/SKILL.md"
skill ptbr "$src/skills/ptbr/SKILL.md"
skill cientifico "$src/skills/cientifico/SKILL.md"

mkdir -p "$dest/plugins" "$dest/ptbr-writer/scripts"
cp -rL "$src/plugins/ptbr-writer/bin" "$dest/ptbr-writer/bin"
cp -L "$src/plugins/ptbr-writer/scripts/to-pdf.sh" "$dest/ptbr-writer/scripts/"
cp -L "$src/plugins/ptbr-writer/ptbr-writer.default.toml" "$dest/ptbr-writer/"
cp -L "$src/opencode/ptbr-writer.js" "$dest/plugins/ptbr-writer.js"
echo "ptbr-writer $tag installed in $dest (skills: ptbr-writer, ptbr, cientifico; plugin: ask-levels)"
