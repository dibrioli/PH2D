#!/bin/bash
# A×B «segunda rodada»: A = e81e56bb1f (bfe7c348e5^) · B = HEAD 57b4ddbac7; o MESMO roteiro (matriz CURTO=1),
# intercalados por bloco (ímpar AB, par BA). Cada variante corre o roteiro da SUA árvore (presets/papel da sua árvore).
SP=/tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/9f271829-f827-4805-a3f4-fb67e13ce887/scratchpad
LW=/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor
SA=$SP/wt-base/docs/Painter/ferramentas/mede_pen_up_4k.sh; BA=$SP/binA
SB=$LW/docs/Painter/ferramentas/mede_pen_up_4k.sh;        BB=$SP/binB
N=${N:-5}
CENAS=(${CENAS:-"3840x2160|Filbert:wet" "3840x2160|omissão:wet" "3840x2160|omissão:impasto" "1920x1080|Filbert:wet"})
for i in $(seq 1 "$N"); do
  [ $((i % 2)) = 1 ] && ORD="A B" || ORD="B A"
  for c in "${CENAS[@]}"; do
    TELA=${c%%|*}; PP=${c#*|}
    for v in $ORD; do
      [ $v = A ] && { S=$SA; BIN=$BA; } || { S=$SB; BIN=$BB; }
      echo "$(date +%T) bloco $i $v $TELA $PP load $(cut -d' ' -f1 /proc/loadavg)" >> $SP/carga.txt
      env CURTO=1 ROT="${v}${i}" KNOBS= TECLAS=0 RELEVO= PRE= SAIDA=$SP/mede bash "$S" matriz "$TELA" papel "$BIN" "$PP"
    done
  done
done
echo "[ab] feito"
