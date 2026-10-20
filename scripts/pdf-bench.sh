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
rev=$(sed -n 's/^PICASSO_REV=//p' "$root/plugins/ptbr-writter/scripts/to-pdf.sh")
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

python3 -I - "$tmp/cache.json" "$tmp/local.json" "$rev" "$secs" "$(nproc)" <<'PY'
import json, sys
cache, local, rev, secs, cores = sys.argv[1:]
text, i, dec, found = open(cache).read(), 0, json.JSONDecoder(), {}
while True:
    while i < len(text) and text[i].isspace():
        i += 1
    if i >= len(text):
        break
    o, i = dec.raw_decode(text, i)
    found.update(o)
local = json.load(open(local))
M = 2**20
served = {p: x for p, x in found.items() if x}
built = [p for p in local if p not in served]
print(f"picasso {rev[:7]}, labs/ptbr-pdf")
print(f"  on disk            {sum(v['narSize'] for v in local.values())/M:6.0f} MiB  {len(local)} store paths")
print(f"  download (zstd)    {sum(x.get('downloadSize', 0) for x in served.values())/M:6.0f} MiB  {len(served)} paths from cache.nixos.org")
print(f"  built locally      {sum(local[p]['narSize'] for p in built)/M:6.0f} MiB  {len(built)} paths (the TeX tree and formats)")
print(f"  first run          {secs}  ({cores} cores; depends on the network)")
PY
