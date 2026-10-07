#!/bin/bash
# A MESMA régua nos dois lados: recortes iguais (tela 1000x640 a 1:1) — recto (linha 1), rápido (linha 2), rampa (linha 2).
S=/tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/7af01775-9131-400f-b9b5-d1d703d8f7c8/scratchpad
R="python3 -I $S/scripts/mede_traco.py"
for f in "$@"; do
  n=$(basename "$f" .png)
  $R "$f" --crop 20,60,580,120 --label "$n recto"
  $R "$f" --crop 580,240,420,120 --label "$n rapido"
  $R "$f" --crop 20,240,560,120 --label "$n rampa"
done
