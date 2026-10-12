#!/usr/bin/env bash
# W15 — o PRETO E BRANCO de uma imagem do quadro nos rasterizadores do SISTEMA: o SVG exportado (os bytes
# originais + `feColorMatrix saturate 0` em linearRGB) e o PDF (os píxeis já cinzentos), lidos no meio da
# imagem cinzenta (20,20 da página + PAD) e da colorida (80,20). O ecrã dá `gray_of` (impresso pelo teste).
# Uso: bash fumo_w15_cinza.sh <pasta_de_trabalho>   (corre o teste #[ignore] que grava w15.svg / w15.pdf)
set -uo pipefail
W="${1:?uso: fumo_w15_cinza.sh <pasta>}"; ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
mkdir -p "$W" && W="$(cd "$W" && pwd)"
( cd "$ROOT" && PH2D_W15_OUT="$W" bash scripts/ph2d-run.sh cargo test -p ph2d-board-export --lib \
    grava_svg_e_pdf_cinzentos -- --ignored --nocapture 2>&1 | grep -E 'PAD=|test result' )
cd "$W" || exit 2
r() { local n=$1; shift; if timeout 120 "$@" >"log_$n.txt" 2>&1; then echo "ok    $n"; else echo "FALHA $n (exit $?)"; fi; }
r rsvg rsvg-convert -o rsvg.png w15.svg;  r resvg resvg w15.svg resvg.png
r inkscape env HOME="$W/inkhome" inkscape w15.svg --export-type=png --export-filename=inkscape.png
r magick_svg magick w15.svg magick_svg.png
r pdftoppm pdftoppm -png -r 72 -singlefile w15.pdf pdftoppm; r gs gs -q -dSAFER -dBATCH -dNOPAUSE -sDEVICE=png16m -r72 -sOutputFile=gs.png w15.pdf
r mutool mutool draw -r 72 -o mutool.png w15.pdf
python3 -I - <<'PY'
from PIL import Image
import glob
for f in sorted(glob.glob("*.png")):
    im = Image.open(f).convert("RGBA")
    print(f"{f:16} cinza(30,30)={im.getpixel((30,30))}  cor(90,30)={im.getpixel((90,30))}  tamanho={im.size}")
PY
