#!/bin/bash
# Os MESMOS gestos da sonda do PH2D (tests/it/sonda_gestos.rs) numa ferramenta do Rebelle.
# uso: tracos_rebelle.sh <x do ícone da ferramenta> <Size a escrever> <nome>   — tela 1280x720 a 100 %,
# canto da tela no ECRÃ = (340, 208). Fotografa em t0 e t8 e desfaz os 5 traços.
O="$HOME/Apps/rebelle/oraculo"; TX=$1; SZ=$2; NM=$3
S=$(python3 -c "
import math
pts=[]
for i in range(25):
    f=i/24; x=600+360*f; y=120+25*math.sin(f*2*math.pi*1.5); pts += [340+x, 208+y]
print(' '.join('%.1f'%v for v in pts))")
bash "$O/inj.sh" "click $TX 113; wait 600; click 215 421; wait 300; key ctrl+a; wait 150; type $SZ; wait 150; key enter; wait 600; \
 drag 400 328 900 328 167; wait 300; \
 path $S; wait 300; \
 drag 400 508 900 508 500; wait 300; \
 drag 940 508 1300 508 14; wait 300; \
 drag 400 688 900 688 167; wait 400" | tail -1
bash "$O/foto.sh" "r_${NM}_t0" | tail -1
sleep 8
bash "$O/foto.sh" "r_${NM}_t8" | tail -1
bash "$O/inj.sh" "key ctrl+z; wait 300; key ctrl+z; wait 300; key ctrl+z; wait 300; key ctrl+z; wait 300; key ctrl+z; wait 800" | tail -1
