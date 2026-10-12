#!/bin/bash
# Prancha por pincel: PH2D | Rebelle (1000×640 cada, grelha A–J × 1–6 igual nos dois) + dois detalhes ampliados.
# uso: prancha_comparar.sh <dir dos midia_*.ppm> <dir de saída> "<preset>|<foto>|<título>"…
set -eu
src=$1; out=$2; shift 2
F="$(cd "$(dirname "$0")/.." && pwd)/fixtures/rebelle"
tmp=$(mktemp -d)
grelha() { # $1 entrada, $2 saída, $3 rótulo
  local d=()
  for i in $(seq 1 9); do d+=(-draw "line $((i*100)),0 $((i*100)),639"); done
  for j in 1 2 3 4 5; do d+=(-draw "line 0,$((j*107)) 999,$((j*107))"); done
  local t=()
  local L=(A B C D E F G H I J)
  for i in $(seq 0 9); do for j in $(seq 0 5); do t+=(-annotate "+$((i*100+4))+$((j*107+14))" "${L[$i]}$((j+1))"); done; done
  magick "$1" -stroke '#00000022' -strokewidth 1 "${d[@]}" -stroke none -fill '#00000055' -pointsize 12 "${t[@]}" \
    -fill black -pointsize 22 -gravity southeast -annotate +10+8 "$3" -gravity northwest \
    -bordercolor '#888' -border 2 "$2"
}
zoom() { # $1 entrada, $2 geometria, $3 escala, $4 saída, $5 rótulo
  magick "$1" -crop "$2" +repage -filter point -resize "$3" -fill black -pointsize 20 -gravity northwest \
    -annotate +6+4 "$5" -bordercolor '#888' -border 2 "$4"
}
for par in "$@"; do
  IFS='|' read -r p f t <<<"$par"
  a="$src/midia_$p.ppm"; b="$F/$f.png"
  grelha "$a" "$tmp/a.png" "PH2D — $p"
  grelha "$b" "$tmp/b.png" "Rebelle — $f"
  zoom "$a" 330x80+20+70 300% "$tmp/za.png" "PH2D: ponta do traço (A1–D2) ×3"
  zoom "$b" 330x80+20+70 300% "$tmp/zb.png" "Rebelle: ponta do traço (A1–D2) ×3"
  zoom "$a" 200x240+200+360 200% "$tmp/xa.png" "PH2D: cruzamento ×2"
  zoom "$b" 200x240+200+360 200% "$tmp/xb.png" "Rebelle: cruzamento ×2"
  magick "$tmp/a.png" "$tmp/b.png" -background white -splice 0x0 +append "$tmp/topo.png"
  magick "$tmp/za.png" "$tmp/zb.png" +append "$tmp/z.png"
  magick "$tmp/xa.png" "$tmp/xb.png" +append "$tmp/x.png"
  magick -size 2008x60 xc:white -fill black -pointsize 32 -gravity west -annotate +16+0 "$t   (esquerda: PH2D · direita: Rebelle)" "$tmp/tit.png"
  magick "$tmp/z.png" "$tmp/x.png" -background white -gravity center -append "$tmp/baixo.png"
  slug=$(echo "$p" | tr ' A-Z' '_a-z')
  magick "$tmp/tit.png" "$tmp/topo.png" "$tmp/baixo.png" -background white -gravity center -append "$out/comparar_$slug.png"
  echo "→ $out/comparar_$slug.png"
done
rm -rf "$tmp"
