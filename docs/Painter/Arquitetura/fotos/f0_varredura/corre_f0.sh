#!/usr/bin/env bash
# Corre a varredura F0/E9 em 8 processos paralelos (um grupo de pincéis cada) sobre o binário de teste já compilado.
# Eixos por omissão (E9): as duas leis × os dois papéis (o PADRÃO decide). Sobrepõe com PH2D_SONDA_LEI/PAPEIS.
# PH2D_F0_TIMEOUT = segundos por grupo (omissão 14400; a grelha E9 inteira mede ~2,5–3 h, ver E9_portao_mae.md).
set -u
W=/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor
OUT=${1:?pasta de saída}
export PH2D_SONDA_LEI=${PH2D_SONDA_LEI:-velha,pigmento}
export PH2D_SONDA_PAPEIS=${PH2D_SONDA_PAPEIS:-padrao,liso}
mkdir -p "$OUT"
# O binário congela-se numa cópia em $OUT: recompilar durante as ~3 h não troca a régua a meio.
cp "$(ls -t $W/target/smoke/deps/ph2d_tool_painter-* | grep -v '\.d$' | head -1)" "$OUT/sonda.bin"
BIN="$OUT/sonda.bin"
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
    timeout "${PH2D_F0_TIMEOUT:-14400}" "$BIN" --ignored --nocapture --exact tool::paint::wetpaint_preset::sonda_varredura_oleo::sonda_varredura_oleo > "$OUT/g$i.log" 2>&1 &
  pids+=($!)
  i=$((i+1))
done
for p in "${pids[@]}"; do wait "$p"; done
head -1 "$OUT/g0.tsv" > "$OUT/varredura.tsv"
for f in "$OUT"/g*.tsv; do tail -n +2 "$f" >> "$OUT/varredura.tsv"; done
wc -l "$OUT/varredura.tsv"
