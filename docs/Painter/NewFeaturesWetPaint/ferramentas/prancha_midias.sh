#!/bin/bash
# A prancha 1:1 das MÍDIAS (prompt 17, Fase C): cada preset novo (a cena do portão `cada_preset_parece_a_sua_midia`,
# seca, à largura da foto do Rebelle) ao lado da foto do preset-oráculo, e o CONTROLO (o Wet Round à mesma largura).
# uso: prancha_midias.sh <dir com midia_<preset>.ppm (PH2D_SONDA_OUT do portão)> <saída.png> "<preset>|<foto>"…
# Recorte das duas primeiras linhas da bancada (recto + S, rampa + rápido): 1000×380 a partir de y 40.
set -eu
src=$1; out=$2; shift 2
F="$(cd "$(dirname "$0")/.." && pwd)/fixtures/rebelle"
tmp=$(mktemp -d); linhas=()
for par in "$@"; do
  p=${par%%|*}; f=${par##*|}
  magick "$src/midia_$p.ppm" -crop 1000x380+0+40 +repage -gravity northwest -pointsize 18 -annotate +8+4 "PH2D: $p" "$tmp/a.png"
  magick "$F/$f.png" -crop 1000x380+0+40 +repage -gravity northwest -pointsize 18 -annotate +8+4 "Rebelle: $f" "$tmp/b.png"
  magick "$src/midia_Wet Round.ppm" -crop 1000x380+0+40 +repage -gravity northwest -pointsize 18 -annotate +8+4 "controlo: Wet Round" "$tmp/c.png"
  magick "$tmp/a.png" "$tmp/b.png" "$tmp/c.png" +append "$tmp/l_$p.png"
  linhas+=("$tmp/l_$p.png")
done
magick "${linhas[@]}" -append "$out"
rm -rf "$tmp"
echo "prancha → $out"
