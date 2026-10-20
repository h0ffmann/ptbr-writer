#!/usr/bin/env bash
# to-pdf — Markdown to PDF for the `.pdf` output: pandoc + xelatex, the flags of picasso's
# labs/ptbr-pdf. Uses pandoc and xelatex from PATH. Without them, it never downloads on its own:
# it says what the download is and exits 3; with --download (after the person agreed), it runs
# labs/ptbr-pdf at the pinned picasso revision (`just pins` checks it against forks/picasso).
#
#   to-pdf.sh [--download] texto.md [saida.pdf]
set -euo pipefail

PICASSO_REV=dcacdde558475d9c403d670b69a4cb8bfcf01b66
SIZE="~730 MiB"  # labs/ptbr-pdf's closure on disk, x86_64-linux (nix path-info -Sh)
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
  echo "to-pdf: no pandoc+xelatex here; the PDF needs a one-time download of $SIZE through nix (pandoc and a minimal TeX). Ask the person; if they agree, rerun with --download" >&2
  exit 3
else
  nix --extra-experimental-features 'nix-command flakes' run \
    "github:h0ffmann/picasso/$PICASSO_REV?dir=labs/ptbr-pdf" -- "$in" "$out" >/dev/null
fi
echo "$out"
