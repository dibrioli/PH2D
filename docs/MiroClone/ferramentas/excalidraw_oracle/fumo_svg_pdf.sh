#!/usr/bin/env bash
# W8 — fumo dos oráculos SVG/PDF do SISTEMA (triagem: ver README §W8). Corre cada ferramenta sobre um SVG e
# um PDF mínimos feitos aqui à mão, e sobre um SVG do Excalidraw (o recorte da moldura, as fontes).
# Uso: bash fumo_svg_pdf.sh <pasta_de_trabalho> [svg_excalidraw]
set -uo pipefail
W="${1:?uso: fumo_svg_pdf.sh <pasta> [svg]}"; HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
X="${2:-$HERE/saidas/w8_tipos.svg}"
mkdir -p "$W" && cd "$W" || exit 2
cat > m.svg <<'SVG'
<svg xmlns="http://www.w3.org/2000/svg" width="120" height="80" viewBox="0 0 120 80"><rect width="120" height="80" fill="#ffffff"/><rect x="10" y="10" width="60" height="40" rx="8" fill="#a5d8ff" stroke="#1e1e1e" stroke-width="2"/><circle cx="90" cy="50" r="20" fill="#ff0000"/><text x="10" y="72" font-family="DejaVu Sans" font-size="10" fill="#000">Olá</text></svg>
SVG
python3 -I - <<'PY'
objs=[b"<< /Type /Catalog /Pages 2 0 R >>", b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
 b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 120 80] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>"]
c=b"1 1 1 rg 0 0 120 80 re f 0.65 0.85 1 rg 10 30 60 40 re f 1 0 0 rg 70 10 30 20 re f BT /F1 10 Tf 0 0 0 rg 10 8 Td (Ola) Tj ET"
objs += [b"<< /Length %d >>\nstream\n" % len(c) + c + b"\nendstream", b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>"]
out=b"%PDF-1.7\n%\xe2\xe3\xcf\xd3\n"; offs=[]
for i,o in enumerate(objs,1): offs.append(len(out)); out+=b"%d 0 obj\n"%i+o+b"\nendobj\n"
x=len(out); out+=b"xref\n0 %d\n0000000000 65535 f \n"%(len(objs)+1)+b"".join(b"%010d 00000 n \n"%o for o in offs)
out+=b"trailer\n<< /Size %d /Root 1 0 R >>\nstartxref\n%d\n%%%%EOF\n"%(len(objs)+1,x); open("m.pdf","wb").write(out)
PY
# o recorte da moldura com a largura 60: o filho g1b (x 130..210, y 430..490 do SVG) tem de sumir
python3 -I -c "
import sys; s=open(sys.argv[1]).read(); old='<clipPath id=\"fr1\"><rect transform=\"translate(10 390) rotate(0 200 100)\" width=\"400\"'
assert s.count(old)==1; open('clip.svg','w').write(s.replace(old, old.replace('width=\"400\"','width=\"60\"')))" "$X"
r() { local n=$1; shift; if timeout 120 "$@" >"log_$n.txt" 2>&1; then echo "ok    $n"; else echo "FALHA $n (exit $?)"; fi; }
export HOME_INK="$PWD/inkhome"
r rsvg rsvg-convert -o rsvg.png m.svg;            r resvg resvg m.svg resvg.png;     r usvg usvg m.svg usvg.svg
r inkscape env HOME="$HOME_INK" inkscape m.svg --export-type=png --export-filename=inkscape.png
r inkscape_pdf env HOME="$HOME_INK" inkscape m.svg --export-type=pdf --export-filename=inkscape.pdf
r magick_svg magick m.svg magick_svg.png;         r xmllint xmllint --noout m.svg "$X"
r pdfinfo pdfinfo m.pdf;                           r pdftoppm pdftoppm -png -r 72 -singlefile m.pdf pdftoppm
r pdftocairo_png pdftocairo -png -r 72 -singlefile m.pdf pdftocairo; r pdftocairo_svg pdftocairo -svg m.pdf pdftocairo.svg
r qpdf_check qpdf --check m.pdf;                   r qpdf_json qpdf --json m.pdf qpdf.json
r gs gs -q -dSAFER -dBATCH -dNOPAUSE -sDEVICE=png16m -r72 -sOutputFile=gs.png m.pdf
r magick_pdf magick -density 72 m.pdf magick_pdf.png
r clip_rsvg rsvg-convert -o c_rsvg.png clip.svg;   r clip_resvg resvg clip.svg c_resvg.png
r clip_inkscape env HOME="$HOME_INK" inkscape clip.svg --export-type=png --export-filename=c_ink.png
python3 -I - <<'PY'
from PIL import Image
import numpy as np, os
def L(p):
    a=np.asarray(Image.open(p).convert("RGBA")).astype(int); return a[...,:3]*a[...,3:4]//255+255*(255-a[...,3:4])//255
def cmp(a,b):
    if not (os.path.exists(a) and os.path.exists(b)): return f"{a} vs {b}: em falta"
    A,B=L(a),L(b); d=np.abs(A-B).max(axis=2)
    return f"{a} vs {b}: max {d.max()} · px>8 {(d>8).mean()*100:.2f}%"
for x in ["resvg.png","inkscape.png","magick_svg.png"]: print(cmp("rsvg.png",x))
for x in ["pdftocairo.png","gs.png","magick_pdf.png"]: print(cmp("pdftoppm.png",x))
for f in ["c_rsvg.png","c_resvg.png","c_ink.png"]:
    if os.path.exists(f):
        a=np.asarray(Image.open(f).convert("RGBA")).astype(int)[430:490,130:210]
        print(f, "recorta a moldura" if ((a[...,:3].sum(axis=2)<300)&(a[...,3]>128)).sum()==0 else "NÃO recorta (ignora ns1:clip-path)")
PY
