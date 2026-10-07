#!/bin/bash
# Para cada ferramenta: envia os traços + simulação (A), zoom 100 %, foto; envia DRY (B), foto; corta a tela (1000x640 em 480,220).
O="$HOME/Apps/rebelle/oraculo"; cd "$O"
for t in oil express wc ink pencil pastel marker airbrush wcpreset wcwet penciltilt oildirty; do
  timeout 120 node ws_eventos.mjs "ev_${t}_A.json" 6000 2 | grep -v "SAVE and LOAD" | tail -2
  timeout 30 bash inj.sh "click 1853 273; wait 300; key ctrl+a; wait 200; type 100; wait 200; key enter; wait 1500" >/dev/null
  timeout 60 bash foto.sh "ws_${t}_A" | tail -1
  timeout 60 node ws_eventos.mjs "ev_${t}_B.json" 4000 2 | tail -1
  timeout 60 bash foto.sh "ws_${t}_B" | tail -1
  for s in A B; do magick "ws_${t}_${s}.png" -crop 1000x640+480+220 +repage "c_ws_${t}_${s}.png"; done
done
echo TODAS_FIM
