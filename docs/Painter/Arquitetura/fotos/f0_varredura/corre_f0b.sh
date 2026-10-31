#!/usr/bin/env bash
# corre.sh <out> <tsv> <pinceis> <pigs> <aguas> <pressoes> <ns> <assenta> <papel> <ordem>
set -u
W=/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor
BIN=$W/target/smoke/deps/ph2d_tool_painter-836625cefa6758a8
mkdir -p "$1"
PH2D_SONDA_OUT="$1" PH2D_SONDA_TSV="$2" PH2D_SONDA_PINCEIS="$3" PH2D_SONDA_PIGMENTOS="$4" PH2D_SONDA_AGUAS="$5" \
 PH2D_SONDA_PRESSOES="$6" PH2D_SONDA_NS="$7" PH2D_SONDA_ASSENTA="$8" PH2D_SONDA_PAPEL="$9" PH2D_SONDA_ORDEM="${10}" \
 timeout 1200 "$BIN" --ignored --nocapture --exact tool::paint::wetpaint_preset::sonda_varredura_oleo::sonda_varredura_oleo
