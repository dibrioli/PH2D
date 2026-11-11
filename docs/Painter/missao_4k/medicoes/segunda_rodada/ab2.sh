#!/bin/bash
SP=/tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/9f271829-f827-4805-a3f4-fb67e13ce887/scratchpad
for i in 1 2 3; do [ $((i%2)) = 1 ] && ORD="P Q" || ORD="Q P"; for v in $ORD; do
  [ $v = P ] && S=$SP/wt-base || S=$SP/wt-mid
  echo "$(date +%T) $i $v load $(cut -d' ' -f1 /proc/loadavg)" >> $SP/carga2.txt
  env CURTO=1 ROT="${v}${i}" KNOBS= TECLAS=0 RELEVO= PRE= SAIDA=$SP/mede2 bash $S/docs/Painter/ferramentas/mede_pen_up_4k.sh matriz 3840x2160 papel $SP/bin$v "omissão:impasto"
done; done
