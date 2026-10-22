#!/usr/bin/env bash
# to-pdf — Markdown to PDF for the `.pdf` output: pandoc + xelatex, else picasso's Rust to-pdf (Typst); the flags of picasso's
# labs/ptbr-pdf. Uses pandoc and xelatex from PATH. Without them, it never downloads on its own:
# it says what the download is and exits 3; with --download (after the person agreed), it runs
# labs/ptbr-pdf at the pinned picasso revision (`just pins` checks it against forks/picasso).
#
#   to-pdf.sh [--download] texto.md [saida.pdf]
set -euo pipefail

PICASSO_REV=5da5237ba6f3fc51de0a10bc746a1ce0ad89e33e
SIZE="~14 MiB (72 MiB on disk)"  # labs/ptbr-pdf at PICASSO_REV, x86_64-linux: scripts/pdf-bench.sh
download=false
[ "${1-}" = --download ] && { download=true; shift; }
in=${1:?usage: to-pdf.sh [--download] texto.md [saida.pdf]}
out=${2:-${in%.*}.pdf}

if command -v pandoc >/dev/null && command -v xelatex >/dev/null; then
  pandoc "$in" --pdf-engine=xelatex -V lang=pt-BR -V geometry:margin=2.5cm \
    -V mainfont=DejaVuSans -V mainfontoptions=Extension=.ttf \
    -V monofont=DejaVuSansMono -V monofontoptions=Extension=.ttf -o "$out"
elif ! command -v nix >/dev/null; then
  echo "to-pdf: no pandoc+xelatex and no nix; deliver $in, and install pandoc and TeX Live (xelatex) for PDFs" >&2
  exit 1
elif ! $download; then
  echo "to-pdf: no pandoc+xelatex here; the PDF needs a one-time download of $SIZE through nix (the to-pdf binary, Typst with fonts embedded), plus compiling it from source: about 15 minutes on 4 cores, not in cache.nixos.org yet. Ask the person; if they agree, rerun with --download" >&2
  exit 3
else
  nix --extra-experimental-features 'nix-command flakes' run \
    "github:h0ffmann/picasso/$PICASSO_REV?dir=labs/ptbr-pdf" -- "$in" "$out" >/dev/null
fi
echo "$out"
