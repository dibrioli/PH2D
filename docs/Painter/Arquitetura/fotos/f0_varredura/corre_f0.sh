#!/usr/bin/env bash
# Corre a varredura F0 em 8 processos paralelos (um grupo de pincéis cada) sobre o binário de teste já compilado.
set -u
W=/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor
OUT=${1:?pasta de saída}
BIN=$(ls -t $W/target/smoke/deps/ph2d_tool_painter-* | grep -v '\.d$' | head -1)
mkdir -p "$OUT"
grupos=(
 "Oil/Flat/Thick Impasto,Oil/Knife/Knife Stain"
 "Oil/Flat/Filbert,Oil/Knife/Knife Dry"
 "Oil/Flat/Filbert Dry,Oil/Knife/Knife Textured"
 "Oil/Flat/Flat Bristle,Oil/Metallic/Metallic Flat"
 "Oil/Flat/Tilted Shade,ExpressOil/Round/Express Oil"
 "Oil/Dabber/Dabber"
 "Oil/Dabber/Dabber 4,Oil/Knife/Knife Smooth 3"
 "Oil/Knife/Knife Oily,Oil/Knife/Knife Grainy"
)
pids=()
i=0
for g in "${grupos[@]}"; do
  PH2D_SONDA_OUT="$OUT" PH2D_SONDA_TSV="g$i.tsv" PH2D_SONDA_PINCEIS="$g" \
    timeout 1500 "$BIN" --ignored --nocapture --exact tool::paint::wetpaint_preset::sonda_varredura_oleo::sonda_varredura_oleo > "$OUT/g$i.log" 2>&1 &
  pids+=($!)
  i=$((i+1))
done
for p in "${pids[@]}"; do wait "$p"; done
head -1 "$OUT/g0.tsv" > "$OUT/varredura.tsv"
for f in "$OUT"/g*.tsv; do tail -n +2 "$f" >> "$OUT/varredura.tsv"; done
wc -l "$OUT/varredura.tsv"
