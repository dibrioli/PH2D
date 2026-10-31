#!/bin/bash
S=${S:?pasta de saída}
B="WetPainter/Oil"
corre() { d=$1; shift; mkdir -p $S/md/$d; echo "== $d $(date +%T) load $(cut -d' ' -f1 /proc/loadavg)"; bash "$(dirname "$0")"/corre.sh $S/md/$d 300 "$@" 2>&1 | grep -E "armado|FIM|PRAZO|fim T" ; }
corre ti_r32 PH2D_OLEO_APP_PRESET=$B/Flat/Thick_Impasto.ph2d-brush.png PH2D_OLEO_APP_RAIO=32
corre filbert_r32 PH2D_OLEO_APP_PRESET=$B/Flat/Filbert.ph2d-brush.png PH2D_OLEO_APP_RAIO=32
corre bristle_r32 PH2D_OLEO_APP_PRESET=$B/Flat/Flat_Bristle.ph2d-brush.png PH2D_OLEO_APP_RAIO=32
corre knifeoily_r32 PH2D_OLEO_APP_PRESET=$B/Knife/Knife_Oily.ph2d-brush.png PH2D_OLEO_APP_RAIO=32 PH2D_OLEO_APP_N_AZUL=12 PH2D_OLEO_APP_N_AMARELO=4
corre dabber_r32 PH2D_OLEO_APP_PRESET=$B/Dabber/Dabber.ph2d-brush.png PH2D_OLEO_APP_RAIO=32 PH2D_OLEO_APP_N_AZUL=12 PH2D_OLEO_APP_N_AMARELO=4
echo LOTE_FIM $(date +%T)
