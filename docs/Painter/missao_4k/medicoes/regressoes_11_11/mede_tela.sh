#!/bin/bash
# uso: N=<blocos> OUT=<dir> mede.sh "<sha>|<pincel:painel>" ...  — ímpar na ordem dada, par invertida
SP=/tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/fb53f5ce-2f42-4773-b6f1-219f6399f510/scratchpad
N=${N:-2}; OUT=${OUT:-$SP/mede}; CEL=("$@"); mkdir -p $OUT
for i in $(seq 1 $N); do
  if [ $((i%2)) = 1 ]; then ORD=("${CEL[@]}"); else ORD=(); for ((k=${#CEL[@]}-1;k>=0;k--)); do ORD+=("${CEL[$k]}"); done; fi
  for c in "${ORD[@]}"; do s=${c%%|*}; pp=${c#*|}
    echo "$(date +%T) $i $s $pp load $(cut -d' ' -f1 /proc/loadavg)" >> $OUT/carga.txt
    env CURTO=1 ROT="${s}_${i}" KNOBS= TECLAS=0 RELEVO= PRE= SAIDA=$OUT bash $SP/arv_$s/docs/Painter/ferramentas/mede_pen_up_4k.sh matriz ${TELA:-3840x2160} papel $SP/bin_$s "$pp"
  done
done
echo "[mede] feito"
