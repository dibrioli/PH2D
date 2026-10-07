#!/bin/bash
# A MESMA régua nos dois lados: recortes iguais (tela 1000x640 a 1:1) — recto (linha 1), rápido (linha 2), rampa (linha 2).
R="python3 -I $(dirname "$(readlink -f "$0")")/mede_traco.py"
for f in "$@"; do
  n=$(basename "$f" .png)
  $R "$f" --crop 20,60,580,120 --label "$n recto"
  $R "$f" --crop 580,240,420,120 --label "$n rapido"
  $R "$f" --crop 20,240,560,120 --label "$n rampa"
done
