#!/usr/bin/env bash
# pdf-bench — what the `.pdf` output costs a machine with Nix and no pandoc: the closure of picasso's
# labs/ptbr-pdf at the revision to-pdf.sh pins, what cache.nixos.org serves (compressed download)
# and what the machine builds itself, and how long the build took. The time means something only on
# a machine that did not have it (plugin.yml's pdf_bench runs this on a fresh GitHub runner); with
# the closure already in the store, it says so.
#
#   scripts/pdf-bench.sh
set -euo pipefail

root=$(git rev-parse --show-toplevel)
rev=$(sed -n 's/^PICASSO_REV=//p' "$root/plugins/ptbr-writer/scripts/to-pdf.sh")
ref="github:h0ffmann/picasso/$rev?dir=labs/ptbr-pdf#to-pdf"
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT

drv_out=$(nix eval --raw "$ref.outPath")
if nix path-info "$drv_out" >/dev/null 2>&1; then secs="already in the store"
else
  start=$(date +%s)
  nix build --no-link "$ref"
  secs="$(( $(date +%s) - start )) s"
fi
top=$drv_out
nix path-info -r "$top" > "$tmp/paths"
xargs -a "$tmp/paths" -P 16 -I{} sh -c \
  'nix path-info --json --json-format 1 --store https://cache.nixos.org "{}" 2>/dev/null || echo "{\"{}\": null}"' > "$tmp/cache.json"
nix path-info --json --json-format 1 $(cat "$tmp/paths") > "$tmp/local.json"

M=1048576
jq -s 'add' "$tmp/cache.json" > "$tmp/found.json"
jq -r --arg rev "$rev" --arg secs "$secs" --arg cores "$(nproc)" --argjson M "$M" --slurpfile f "$tmp/found.json" '
  ($f[0] | with_entries(select(.value != null))) as $served
  | to_entries as $all
  | ($all | map(select(.key as $k | $served | has($k) | not))) as $built
  | def mib: . / $M | floor;
    "picasso \($rev[:7]), labs/ptbr-pdf",
    "  on disk            \($all | map(.value.narSize) | add | mib | tostring | (" " * (6 - length)) + .) MiB  \($all | length) store paths",
    "  download (zstd)    \($served | to_entries | map(.value.downloadSize // 0) | add // 0 | mib | tostring | (" " * (6 - length)) + .) MiB  \($served | length) paths from cache.nixos.org",
    "  built locally      \($built | map(.value.narSize) | add // 0 | mib | tostring | (" " * (6 - length)) + .) MiB  \($built | length) paths (the to-pdf binary)",
    "  first run          \($secs)  (\($cores) cores; depends on the network)"
' "$tmp/local.json"
